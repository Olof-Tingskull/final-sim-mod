use rand::Rng;
use std::cmp::Ordering;

use crate::Config;
use crate::StepLog;

pub struct Car {
    pub position: f64,
    pub velocity: f64,
    pub lane: i32,
}

pub struct LaneParameters {
    forward: usize,
    backward: usize,
}

pub type AllCars = Vec<Car>;

#[derive(Debug, Clone, Copy)]
struct Direction(i32);

impl Direction {
    pub const FORWARD: Self = Self(1);
    pub const RIGHT: Self = Self(1);
    pub const BACKWARD: Self = Self(-1);
    pub const LEFT: Self = Self(-1);
}

impl From<Direction> for i32 {
    fn from(direction: Direction) -> Self {
        direction.0 as i32
    }
}

fn calculate_distance(pos1: f64, pos2: f64, config: &Config) -> f64 {
    if pos1 == pos2 {
        return config.road_length;
    }
    return (pos2 - pos1 + config.road_length) % config.road_length;
}

fn calculate_appropriate_velocity(config: &Config, distance: f64) -> f64 {
    return f64::min(f64::max(f64::sqrt(2. * config.max_deceleration * distance), 0.), config.max_velocity);
}

fn calculate_break_distance(config: &Config, vel1: f64, vel2: f64) -> f64 {
    return (vel2 * vel2 - vel1 * vel1) / (2. * config.max_deceleration); 
}

fn find_nearest_car(
    cars: &AllCars,
    start_index: usize,
    lane: i32,
    direction: Direction,
) -> Option<usize> {
    let mut index = start_index as i32;

    loop {
        let n = cars.len() as i32;
        let increment: i32 = direction.into();
        index = (index + increment + n) % (n as i32);

        if cars[index as usize].lane == lane || index == start_index as i32 {
            break;
        }
    }

    if cars[index as usize].lane == lane {
        return Some(index as usize);
    }

    return None;
}

fn lane_parameters(cars: &AllCars, start_index: usize, lane: i32) -> Option<LaneParameters> {
    let next_car = find_nearest_car(cars, start_index, lane, Direction::FORWARD);
    let prev_car = find_nearest_car(cars, start_index, lane, Direction::BACKWARD);

    if next_car.is_some() && prev_car.is_some() {
        return Some(LaneParameters {
            forward: next_car.unwrap(),
            backward: prev_car.unwrap(),
        });
    }

    return None;
}

fn update_car_velocity(car: &Car, car_in_front: &Car, config: &Config, log: &mut StepLog) -> f64 {
    let new_velocity;

    let distance = calculate_distance(car.position, car_in_front.position, config) - config.car_length;
    
    if distance > 0. {
        let target_velocity = calculate_appropriate_velocity(config, distance);
        if target_velocity < car.velocity {
            new_velocity = f64::max(car.velocity - config.max_deceleration * config.dt, target_velocity);
        } else {
            new_velocity = f64::min(car.velocity + config.max_acceleration * config.dt, target_velocity);
        }
    } else {
        log.collisions += 1;
        new_velocity = 0.;
    }

    return new_velocity;
}

fn calculate_lane_score (car: &Car, car_in_front: &Car, car_behind: &Car, config: &Config) -> f64 {
    let distance_front = calculate_distance(car.position, car_in_front.position, config);
    let distance_behind = calculate_distance(car_behind.position, car.position, config);

    let velocity_for_behind = calculate_appropriate_velocity(config, distance_behind);
    let velocity_for_front = calculate_appropriate_velocity(config, distance_front);

    return (velocity_for_behind + velocity_for_front) / (2. * config.max_velocity);
} 

fn evaluate_lane_change(cars: &AllCars, config: &Config, i: usize, direction: Direction) -> (f64, i32) {
    let current_lane = cars[i].lane;
    let increment: i32 = direction.into();

    let mut score = 0.;
    let mut lanes_away = 1;
    loop {
        let lane = current_lane + increment * lanes_away;
        if lane < 0 || lane >= config.num_lanes { break; }
        if lanes_away > config.view_width { break; }

        if let Some(parameters) = lane_parameters(cars, i, lane) {
            let behind = &cars[parameters.backward];
            let in_front = &cars[parameters.forward];

            let distance_required = config.car_length + config.max_velocity * config.dt * ((lanes_away - 1) as f64);

            let distance_behind = calculate_distance(behind.position, cars[i].position, config);
            let distance_in_front = calculate_distance(cars[i].position, in_front.position, config);

            let would_collide = 
                distance_behind < distance_required 
                    + f64::max(calculate_break_distance(config, cars[i].velocity, behind.velocity), 0.) 
                && 
                distance_in_front < distance_required
                    + f64::max(calculate_break_distance(config, in_front.velocity, cars[i].velocity), 0.);

            if would_collide {
                break;
            } else {
                score = f64::max(score, calculate_lane_score(&cars[i], &in_front, &behind, config));
            }
        } else {
            score = f64::max(score, calculate_lane_score(&cars[i], &cars[i],  &cars[i], config));
        }

        lanes_away+=1;
    }

    return (score, current_lane + increment);
}

pub fn random_breaking(cars: &mut AllCars, probability: f64) {
    let mut rng = rand::thread_rng();
    if probability > 1. || rng.gen_bool(probability) {
        let breaking_car = rng.gen_range(0..cars.len());
        cars[breaking_car].velocity = 0.;
        if probability > 1. {
            random_breaking(cars, probability - 1.);
        }
    }
}

pub fn update_velocity(cars: &mut AllCars, config: &Config, log: &mut StepLog) {
    for i in 0..cars.len() {
        if let Some(current_lane) = lane_parameters(cars, i, cars[i].lane) {
            let in_front = current_lane.forward;
            let behind = current_lane.backward;

            cars[i].velocity = update_car_velocity(&cars[i], &cars[in_front], config, log);

            let current_lane_score = 
                calculate_lane_score(&cars[i], &cars[in_front], &cars[behind], config)
                + config.current_lane_bias;

            let mut lane_options = [
                (current_lane_score, cars[i].lane),
                evaluate_lane_change(cars, config, i, Direction::RIGHT),
                evaluate_lane_change(cars, config, i, Direction::LEFT),
            ].map(|(score, lane)| (score, lane, i32::abs(lane - cars[i].lane)));

            lane_options.sort_by(|a, b| {
                match b.0.partial_cmp(&a.0) {
                    Some(std::cmp::Ordering::Equal) => a.2.cmp(&b.2),
                    other => other.unwrap(),
                }
            });

            cars[i].lane = i32::min(i32::max(lane_options[0].1, 0), config.num_lanes - 1);
        }
    }
}

pub fn update_position(cars: &mut AllCars, config: &Config) {
    for car in cars.iter_mut() {
        car.position = (car.position + car.velocity * config.dt) % config.road_length;
    }

    cars.sort_by(|a, b| {
        a.position
            .partial_cmp(&b.position)
            .unwrap_or(Ordering::Equal)
    });
}
