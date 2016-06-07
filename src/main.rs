extern crate rand;
extern crate docopt;
extern crate rustc_serialize;

use rand::Rng;
use docopt::Docopt;


// Write the Docopt usage string.
const USAGE: &'static str = "
Usage:
    illuminated-inmates [--prisoner-count=<prisoner-count>] [--repetitions=<repetitions>] [--log-period=<log-period>]

Options:
    -n, --prisoner-count=<prisoner-count>   Number of inmates to simulate [default: 100]
    -r, --repetitions=<repetitions>         Number of times to repeat the simulation (for statistical analysis) [default: 1]
    -l, --log-period=<log-period>           How frequently should the simulation log (in days) [default: 1000]
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_prisoner_count: usize,
    flag_repetitions: u32,
    flag_log_period: u32,
}


#[derive(Clone)]
struct Prisoner {
    known_visited_prisoners: Vec<bool>,
}
impl Prisoner {
    fn new(prisoner_count: usize) -> Prisoner {
        Prisoner {
            known_visited_prisoners: vec![false; prisoner_count],
        }
    }

    fn select_light_position(&mut self, day: u32, light_is_on: bool, self_index: usize) -> bool {
        let prisoner_count = self.known_visited_prisoners.len();

        // I now know that I've been interrogated
        self.known_visited_prisoners[self_index] = true;

        // If the previous prisoner knew that this prisoner has been interrogated
        if day > 0 && light_is_on {
            // TODO: multiple strategies
            self.known_visited_prisoners[((day - 1) as usize) % prisoner_count] = true;
        }

        // Return whether we know if today's prisoner has been interrogated
        self.known_visited_prisoners[(day as usize) % prisoner_count]
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
    fn new(prisoner_count: usize) -> WorldState {
        WorldState {
            prisoners: vec![Prisoner::new(prisoner_count); prisoner_count],
            day: 0,
            light_is_on: false,
        }
    }

    fn iterate(&mut self) -> bool {
        let prisoner_count = self.prisoners.len();
        let chosen_prisoner_index = rand::thread_rng().gen_range(0usize, prisoner_count);
        let mut chosen_prisoner = &mut self.prisoners[chosen_prisoner_index];

        self.light_is_on = chosen_prisoner.select_light_position(
            self.day,
            self.light_is_on,
            chosen_prisoner_index,
        );

        // If this prisoner now knows that all prisoners have been interrogated
        if chosen_prisoner.count_known() == (prisoner_count as u32) {
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

// Returns the duration in days
fn run_simulation(prisoner_count: usize, log_period: u32) -> u32 {
    println!("Beginning simulation with {} prisoners", prisoner_count);

    let mut state = WorldState::new(prisoner_count);

    while !state.iterate() {
        if state.day % log_period == 0 {
            println!("Day: {}, max-known: {}", state.day, state.best_known());
        }
    }

    println!("Done! Day: {}\n", state.day);

    state.day
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    let simulation_results = (0..args.flag_repetitions)
        .map(|_| run_simulation(args.flag_prisoner_count, args.flag_log_period))
        .collect::<Vec<u32>>();

    let average_runtime: u32 = simulation_results.iter().fold(0, |sum, x| sum + x) / args.flag_repetitions;
    println!("Average run-time (over {} runs): {} days", args.flag_repetitions, average_runtime);
}