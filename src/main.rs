extern crate rand;
extern crate docopt;
extern crate rustc_serialize;
extern crate crossbeam;
extern crate simple_parallel;

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

struct SimulationResult {
    last_prisoner_interrogated_on_day: u32,
    prisoners_freed_on_day: u32,
}

fn count_true(iter: std::slice::Iter<bool>) -> u32 {
    iter.fold(
        0u32,
        |sum, &is_true|
            sum + if is_true { 1 } else { 0 }
    )
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

    fn get_todays_prisoner_indexes(&self, day: u32) -> Vec<usize> {
        let prisoner_count = self.known_visited_prisoners.len();
        let period = prisoner_count;
        let day_index = (day as usize) % period;

        vec![day_index]
    }

    fn select_light_position(&mut self, day: u32, light_is_on: bool, self_index: usize) -> bool {
        // I now know that I've been interrogated
        self.known_visited_prisoners[self_index] = true;

        // If the previous prisoner knew that this prisoner has been interrogated
        if day > 0 && light_is_on {
            for i in self.get_todays_prisoner_indexes(day - 1) {
                 self.known_visited_prisoners[i] = true;
            }
        }

        // Return whether we know if today's prisoner has been interrogated
        self.get_todays_prisoner_indexes(day).iter().all(|&i| self.known_visited_prisoners[i])
    }

    fn count_known(&self) -> u32 {
        count_true(self.known_visited_prisoners.iter())
    }
}

struct WorldState {
    prisoners: Vec<Prisoner>,
    interrogated_prisoners: Vec<bool>,
    day: u32,
    light_is_on: bool,
    last_prisoner_interrogated_on_day: Option<u32>,
}
impl WorldState {
    fn new(prisoner_count: usize) -> WorldState {
        WorldState {
            prisoners: vec![Prisoner::new(prisoner_count); prisoner_count],
            interrogated_prisoners: vec![false; prisoner_count],
            day: 0,
            light_is_on: false,
            last_prisoner_interrogated_on_day: None,
        }
    }

    fn iterate(&mut self) -> bool {
        let prisoner_count = self.prisoners.len();
        let chosen_prisoner_index = rand::thread_rng().gen_range(0usize, prisoner_count);
        let mut chosen_prisoner = &mut self.prisoners[chosen_prisoner_index];

        // Mark the chosen prisoner as having been interrogated (for checking the prisoner's statement)
        self.interrogated_prisoners[chosen_prisoner_index] = true;

        self.light_is_on = chosen_prisoner.select_light_position(
            self.day,
            self.light_is_on,
            chosen_prisoner_index,
        );

        let actual_count = count_true(self.interrogated_prisoners.iter());
        let reported_count = chosen_prisoner.count_known();

        // If the last prisoner hasn't yet been interrogated
        match self.last_prisoner_interrogated_on_day {
            Some(_) => {},
            None => {
                // If the last prisoner was just interrogated
                if actual_count == (prisoner_count as u32) {
                    // Record the day when the last prisoner was interrogated
                    self.last_prisoner_interrogated_on_day = Some(self.day);
                }
            },
        }

        if reported_count > actual_count {
            panic!(
                "Prisoner over-estimated the count of interrogated prisoners. Actual count: {}, Reported count: {}",
                actual_count,
                reported_count
            );
        }

        // If this prisoner now knows that all prisoners have been interrogated
        if reported_count == (prisoner_count as u32) {
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
fn run_simulation(prisoner_count: usize, maybe_log_period: Option<u32>) -> SimulationResult {
    // If a log period was provided
    match maybe_log_period {
        Some(_) => {
            println!("\nBeginning simulation with {} prisoners", prisoner_count);
        },
        None => {},
    }

    let mut state = WorldState::new(prisoner_count);

    while !state.iterate() {
        // If a log period was provided
        match maybe_log_period {
            Some(log_period) => {
                // If the current day aligns with the specified period
                if state.day % log_period == 0 {
                    println!("Day: {}, max-known: {}", state.day, state.best_known());
                }
            },
            None => {},
        }
    }

    println!(
        "Done! Day: {}, Last Interrogation: {}",
        state.day,
        state.last_prisoner_interrogated_on_day.unwrap(),
    );

    SimulationResult {
        last_prisoner_interrogated_on_day: state.last_prisoner_interrogated_on_day.unwrap(),
        prisoners_freed_on_day: state.day,
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    let simulation_results = crossbeam::scope(|scope| {
        let result_iterator = simple_parallel::map(scope, (0..args.flag_repetitions), |_| {
            run_simulation(
                args.flag_prisoner_count,
                if args.flag_log_period == 0 {
                    None
                } else {
                    Some(args.flag_log_period)
                }
            )
        });

        result_iterator.collect::<Vec<SimulationResult>>()
    });

    let average_runtime: u32 = simulation_results.iter().fold(0, |sum, sim_result| sum + sim_result.prisoners_freed_on_day) / args.flag_repetitions;
    let average_last_interrogated_day: u32 = simulation_results.iter().fold(0, |sum, sim_result| sum + sim_result.last_prisoner_interrogated_on_day) / args.flag_repetitions;
    println!("Average results (over {} simulations): last interrogation day {}, last {} day", args.flag_repetitions, average_last_interrogated_day, average_runtime);
}