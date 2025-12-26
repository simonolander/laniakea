use crate::model::universe::Universe;
use ordered_float::OrderedFloat;
use std::thread::sleep;
use std::time::Duration;

mod model;

fn main() {
    loop {
        let universe = Universe::generate(10, 10);
        println!("{universe}");
        println!("{}", universe.get_score());
        let g = universe
            .get_galaxies()
            .iter()
            .max_by_key(|g| OrderedFloat(g.get_score()))
            .cloned()
            .unwrap();
        println!("{g}");
        println!("{}", g.get_score());
        // let largest_galaxy = universe
        //     .get_galaxies()
        //     .iter()
        //     .max_by_key(|g| g.size())
        //     .cloned()
        //     .unwrap();
        // println!("{largest_galaxy}");
        // let largest_galaxy_skeleton = largest_galaxy.get_skeleton();
        // println!("{largest_galaxy_skeleton}");

        sleep(Duration::from_millis(1000));
    }
}
