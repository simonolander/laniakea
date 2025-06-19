use std::thread::sleep;
use std::time::Duration;
use crate::app::App;
use crate::model::universe::Universe;

mod app;
mod model;

const CONSOLE: bool = false;

fn main() {
    if CONSOLE {
       loop {
           let universe = Universe::generate(10, 10);
           println!("{universe}");
           println!("{}", universe.get_score());
           let g = universe.get_galaxies().iter().max_by_key(|g| g.get_swirl().abs() as i64).cloned().unwrap();
           println!("{g}");
           println!("{}", g.get_swirl());
           let g = universe.get_galaxies().iter().max_by_key(|g| g.get_curl().abs() as i64).cloned().unwrap();
           println!("{g}");
           println!("{}", g.get_curl());
           println!();
           sleep(Duration::from_millis(1000));
       }
    }
    else {
        console_error_panic_hook::set_once();
        App::new().expect("Failed to create application");
    }
}
