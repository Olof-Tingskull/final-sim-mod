extern crate piston;
extern crate piston_window;

use piston::window::WindowSettings;
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston_window::{Context, G2d, PistonWindow};

pub const WINDOW_WIDTH: u32 = 1400;
pub const WINDOW_HEIGHT: u32 = 400;

pub trait Simulation {
    fn update(&mut self);
    fn render(&self, context: &Context, graphics: &mut G2d);
    fn log(&self);
}

pub fn start_simulation<S>(mut simulation: S)
    where S: Simulation
{
    let mut window: PistonWindow = 
    WindowSettings::new("Cars ", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let time_between_logs = 1.;
    let mut log_timer = 0.;
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        if let Some(_) = e.render_args() {
            window.draw_2d(&e, |context, mut graphics, _| {
                piston_window::clear([1.0; 4], graphics);
                simulation.render(&context, &mut graphics);
            });
        }

        if let Some(u) = e.update_args() {
            simulation.update();

            log_timer += u.dt;
            if log_timer > time_between_logs {
                log_timer = 0.;
                simulation.log();
            }
        }
    }
}
