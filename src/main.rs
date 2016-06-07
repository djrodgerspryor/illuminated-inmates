extern crate rand;

use rand::Rng;

const PRISONER_COUNT: usize = 100;
const LOG_PERIOD: u32 = 1000;

#[derive(Clone)]
struct Prisoner {
    known_visited_prisoners: Vec<bool>,
}
impl Prisoner {
    fn new() -> Prisoner {
        Prisoner {
            known_visited_prisoners: vec![false; PRISONER_COUNT],
        }
    }

    fn select_light_position(&mut self, day: u32, light_is_on: bool, self_index: usize) -> bool {
        // I now know that I've been interrogated
        self.known_visited_prisoners[self_index] = true;

        // If the previous prisoner knew that this prisoner has been interrogated
        if day > 0 && light_is_on {
            // TODO: multiple strategies
            self.known_visited_prisoners[((day - 1) as usize) % PRISONER_COUNT] = true;
        }

        // Return whether we know if today's prisoner has been interrogated
        self.known_visited_prisoners[(day as usize) % PRISONER_COUNT]
    }

    fn count_known(&self) -> u32 {
        self.known_visited_prisoners.iter().fold(
            0u32,
            |sum, &has_been_interrogated|
                sum + if has_been_interrogated { 1 } else { 0 }
        )
    }
}

struct WorldState {
    prisoners: Vec<Prisoner>,
    day: u32,
    light_is_on: bool,
}
impl WorldState {
    fn new() -> WorldState {
        WorldState {
            prisoners: vec![Prisoner::new(); PRISONER_COUNT],
            day: 0,
            light_is_on: false,
        }
    }

    fn iterate(&mut self) -> bool {
        let chosen_prisoner_index = rand::thread_rng().gen_range(0usize, PRISONER_COUNT);
        let mut chosen_prisoner = &mut self.prisoners[chosen_prisoner_index];

        self.light_is_on = chosen_prisoner.select_light_position(
            self.day,
            self.light_is_on,
            chosen_prisoner_index,
        );

        // If this prisoner now knows that all prisoners have been interrogated
        if chosen_prisoner.count_known() == (PRISONER_COUNT as u32) {
            return true; // Done
        }

        self.day += 1;

        false
    }

    fn best_known(&self) -> u32 {
        self.prisoners.iter()
            .map(|ref prisoner| prisoner.count_known())
            .max()
            .unwrap() // There should always be non-zero prisoners
    }
}

fn main() {
    let mut state = WorldState::new();

    while !state.iterate() {
        if state.day % LOG_PERIOD == 0 {
            println!("Day: {}, max-known: {}", state.day, state.best_known());
        }
    }

    println!("Done! Day: {}", state.day);
}