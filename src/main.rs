mod cars;
mod simulate;
mod util;

use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use util::{hue_to_rgb, linspace};
use cars::{random_breaking, update_position, update_velocity, AllCars, Car};
use piston_window::{Context, G2d};
use rand::Rng;
use simulate::{start_simulation, Simulation, WINDOW_HEIGHT, WINDOW_WIDTH};


#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
struct RunConfig {
    road_length: f64,
    num_lanes: i32,
    car_density: f64,
    acceleration_rate: f64,
    break_rate: f64,
    max_movement: f64,
    view_width: i32,
    random_stop_rate: f64,
    current_lane_bias: f64,
    steps_to_run: usize,
}

#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
struct Config {
    road_length: f64,
    car_length: f64,
    num_cars: usize,
    num_lanes: i32,
    max_velocity: f64,
    max_acceleration: f64,
    max_deceleration: f64,
    view_width: i32,
    random_stop_rate: f64,
    dt: f64,
    current_lane_bias: f64,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SimulationResult {
    flow_rate: f64,
    max_flow_rate: f64,
    collisions: f64,
}

#[derive(Serialize, Deserialize)]
struct SimulationSave {
    config: RunConfig,
    result: SimulationResult,
}

#[derive(Debug, Clone, Copy)]
struct StepLog {
    flow_rate: f64,
    collisions: i32,
}

impl StepLog {
    fn new() -> StepLog {
        return StepLog {
            flow_rate: 0.,
            collisions: 0,
        };
    }
}

struct CarsSimulation {
    cars: AllCars,
    config: Config,
    logs: Vec<StepLog>,
    current_log: StepLog,
}

impl CarsSimulation {
    fn new(config: Config) -> CarsSimulation {
        let mut cars: AllCars = Vec::with_capacity(config.num_cars);

        let mut rng = rand::thread_rng();
        for _ in 0..config.num_cars {
            let new_car = Car {
                position: rng.gen_range(0.0..config.road_length - config.car_length),
                velocity: rng.gen_range(0.0..config.max_velocity),
                lane: rng.gen_range(0..config.num_lanes),
            };
            cars.push(new_car);
        }

        return CarsSimulation {
            cars: cars,
            config: config,
            logs: Vec::new(),
            current_log: StepLog::new(),
        };
    }
}

impl Simulation for CarsSimulation {
    fn update(&mut self) {
        self.current_log = StepLog::new();
        update_position(&mut self.cars, &self.config);
        random_breaking(
            &mut self.cars,
            (self.config.num_cars as f64) * self.config.random_stop_rate,
        );
        update_velocity(&mut self.cars, &self.config, &mut self.current_log);

        let mut flow_rate = 0.;
        for (_, car) in self.cars.iter().enumerate() {
            flow_rate += car.velocity;
        }
        self.current_log.flow_rate += flow_rate / self.config.road_length;

        self.logs.push(self.current_log);
    }

    fn log(&self) {
        let mut sum = 0.;
        for log in self.logs.iter() {
            sum += log.flow_rate;
        }
        let average_flow_rate = sum / self.logs.len() as f64;
        println!("{}", average_flow_rate);
    }

    fn render(&self, context: &Context, graphics: &mut G2d) {
        let x_mul = WINDOW_WIDTH as f64 / self.config.road_length;
        let y_mul = (WINDOW_HEIGHT / self.config.num_lanes as u32) as f64;

        let mut draw_car = |pos: f64, lane: i32, vel: f64| {
            piston_window::rectangle(
                hue_to_rgb(120. - 120. * vel / self.config.max_velocity),
                [
                    pos * x_mul,
                    lane as f64 * y_mul,
                    self.config.car_length * x_mul,
                    y_mul,
                ],
                context.transform,
                graphics,
            );
        };

        for car in self.cars.iter() {
            draw_car(car.position, car.lane, car.velocity);
            if car.position + self.config.car_length > self.config.road_length {
                draw_car(
                    car.position - self.config.road_length,
                    car.lane,
                    car.velocity,
                );
            }
        }
    }
}

fn bake_config (ic: RunConfig) -> Config {
    return Config {
        road_length: ic.road_length,
        num_lanes: ic.num_lanes,
        car_length: 1.,
        num_cars: (ic.car_density * ic.road_length * (ic.num_lanes as f64)) as usize,
        max_velocity: ic.max_movement,
        max_acceleration: ic.max_movement * ic.acceleration_rate,
        max_deceleration: ic.max_movement * ic.break_rate,
        view_width: ic.view_width,
        random_stop_rate: ic.random_stop_rate,
        current_lane_bias: ic.current_lane_bias,
        dt: 1.,
    };
}

fn run_simulation(input_config: RunConfig) -> Option<SimulationResult> {
    let config = bake_config(input_config);

    let max_flow_rate = (
        config.max_velocity * (config.num_cars as f64) 
        * (1. - config.random_stop_rate * config.max_velocity / config.max_acceleration / config.dt / 2.)
    ) / (config.road_length as f64);

    let mut simulation = CarsSimulation::new(config);
    
    for _ in 0..input_config.steps_to_run {
        simulation.update();
    }

    //let relevant_logs = &simulation.logs[simulation.logs.len() - steps_since_collision..];

    let mut average_collisions = 0.;
    let mut average_flow_rate = 0.;
    for log in simulation.logs.iter() {
        average_collisions += log.collisions as f64;
        average_flow_rate += log.flow_rate;
    }
    average_flow_rate /= simulation.logs.len() as f64;
    average_collisions /= simulation.logs.len() as f64;

    return Some(SimulationResult {
        flow_rate: average_flow_rate,
        max_flow_rate: max_flow_rate,
        collisions: average_collisions,
    })
}

fn run_batch(configs: Vec<RunConfig>) ->  Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./result");
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir(path)?;

    let num_done = AtomicUsize::new(0);

    let best = configs.par_iter().enumerate().filter_map(|(i, &config)| {
        let result_option = run_simulation(config);

        num_done.fetch_add(1, Ordering::SeqCst);
        println!("{}/{}", num_done.load(Ordering::SeqCst), configs.len());

        if let Some(result) = result_option {
            let save = SimulationSave {
                config: config,
                result: result, 
            };
        
            let serialized = serde_json::to_string(&save).expect("Serialization failed");
            let mut file = File::create(format!("./result/sim-{i}.json")).expect("File creation failed");
            file.write_all(serialized.as_bytes()).expect("Write to file failed");
            
            return Some(save);
        }

        return None;
    }).max_by(|a, b| {
        a.result.flow_rate.partial_cmp(&b.result.flow_rate).unwrap()
    }).unwrap();


    let mut sim = CarsSimulation::new(bake_config(best.config));
    for _ in 0..1000 {
        sim.update();
    }
    start_simulation(sim);

    Ok(())
}

fn main() ->  Result<(), Box<dyn std::error::Error>> {
    let car_densities = vec![0.1];
    let biases = linspace(0., 1., 50);
    let num_lanes = vec![10];//linspace(0, 30, 2);
    let stop_rates = vec![0.0001, 0.0002, 0.0005, 0.001, 0.002, 0.005];

    let mut configs = Vec::<RunConfig>::new();

    for &stop_rate in stop_rates.iter() {
        for &bias in biases.iter() {
            for &num_lanes in num_lanes.iter() {
                for &car_density in car_densities.iter() {
                    let config = RunConfig {
                        road_length: 200.,
                        num_lanes: num_lanes,
                        car_density: car_density,
                        max_movement: 0.2,
                        acceleration_rate: 0.001,
                        break_rate: 0.01,
                        view_width: 1,
                        random_stop_rate: stop_rate,
                        current_lane_bias: bias,
                        steps_to_run: 5000,
                    };
                    
                    configs.push(config);
                }
            }
        }
    }

    run_batch(configs)?;

    Ok(())
}