#![allow(dead_code)]

#[macro_use] extern crate rand;
extern crate clap;

mod entities;

use entities::*;

use rand::{thread_rng, Rng};
use std::collections::HashMap;
use clap::{App, Arg};

fn main() {
    let params = App::new("MatchMaking modeling")
        .version("1.0")
        .author("botanick333@gmail.com")
        .about("A humble attempt to simulate typical matchmaking algorithms")

        .arg(Arg::with_name("time")
            .short("t")
            .help("A period of time to simulate in seconds")
            .default_value("86400")
        )

        .arg(Arg::with_name("users_at_start")
            .short("u")
            .help("Amount of users to be generated")
            .default_value("500")
        )
        .arg(Arg::with_name("users_to_gen")
            .short("g")
            .help("Amount of users to be generated")
            .default_value("500")
        )

        .arg(Arg::with_name("name")
            .short("n")
            .takes_value(true)
            .help("Name of the simulation")
        )
        .arg(Arg::with_name("search_delay")
            .takes_value(true)
            .help("Delay between searches in ticks")
            .default_value("10")
        )
        .arg(Arg::with_name("skill")
            .short("s")
            .help("Default skill level assigned to the user")
            .default_value("1500"))
        .arg(Arg::with_name("real_skill_max")
            .long("rmax")
            .help("Maximum value of the skill level")
            .default_value("2200"))
        .arg(Arg::with_name("real_skill_min")
            .long("rmin")
            .help("Minimum value of the skill level")
            .default_value("800"))

        .arg(Arg::with_name("max_game_length")
            .long("max_game_length")
            .help("The amount of time before user reenter queue")
            .default_value("300"))
        .arg(Arg::with_name("continuous_play_prob")
            .long("continuous_play_prob")
            .help("The probability that after a game user will join the queue")
            .default_value("0.0"))

        .arg(Arg::with_name("algorithm")
            .short("a")
            .long("alg")
            .help("Algorithm type")
            .possible_values(&["fifo", "rnd", "skill"])
            .default_value("rnd"))
        .arg(Arg::with_name("team_size")
            .long("team_size")
            .takes_value(true)
            .help("The size of the team")
            .default_value("5"))
        .arg(Arg::with_name("queue_factor")
            .long("queue_factor")
            .takes_value(true)
            .help("Queue overloading factor")
            .default_value("1.0"))
        .arg(Arg::with_name("prefill_factor")
            .long("prefill_factor")
            .takes_value(true)
            .help("Amount of users to be added to the team on the first run of the search algorithm")
            .default_value("0.0"))

        .get_matches();

    let ticks = params.value_of("time").unwrap().parse::<u32>().unwrap();

    let users_to_gen = params.value_of("users_to_gen").unwrap().parse::<u32>().unwrap();
    let users_at_start = params.value_of("users_at_start").unwrap().parse::<u32>().unwrap();

    let search_delay = params.value_of("search_delay").unwrap().parse::<u32>().unwrap();

    let default_skill_level = params.value_of("skill").unwrap().parse::<f32>().unwrap();
    let continuous_play_prob = params.value_of("continuous_play_prob").unwrap().parse::<f32>().unwrap();
    let max_game_length = params.value_of("max_game_length").unwrap().parse::<u32>().unwrap();

    let real_skill_min = params.value_of("real_skill_min").unwrap().parse::<f32>().unwrap();
    let real_skill_max = params.value_of("real_skill_max").unwrap().parse::<f32>().unwrap();

    let name = params.value_of("name")
        .map(|str| String::from(str))
        .unwrap_or((String::from("report_") + &thread_rng().next_u32().to_string()));

    let team_size = params.value_of("team_size").unwrap().parse::<usize>().unwrap();
    let queue_factor = params.value_of("queue_factor").unwrap().parse::<f32>().unwrap();
    let prefill_factor = params.value_of("prefill_factor").unwrap().parse::<f32>().unwrap();

    let algorithm: Box<entities::Algoritm> = match params.value_of("algorithm").unwrap() {
        "fifo" => Box::new(FIFOAlgorithm {
            team_size: team_size,
        }),
        "rnd" => Box::new(RandomPeekAlgorithm {
            team_size: team_size,
        }),
        "skill" => Box::new(SkillLevelAlgorithm {
            size_factor: queue_factor,
            team_size: team_size,
            prefill_factor: prefill_factor,
        }),
        _ => panic!()
    };

    let mut model = Model {
        name: name.clone(),
        queue: Vec::new(),

        user_pool: UserPool::new(),
        users_at_start: users_at_start,
        users_to_gen: users_to_gen,

        algorithm: algorithm,
        decider: Box::new(RealSkillLevelDecider {}),

        properties: HashMap::new(),
        delayed_enter: HashMap::new(),

        default_skill: default_skill_level,
        continuous_play_prob: continuous_play_prob,
        max_game_length: max_game_length,

        real_skill_gen: RandomRangeGen::new(real_skill_min, real_skill_max, DistributionType::Uniform),
    };

    let log = model.run(ticks, search_delay);

    save_results(&name, log);
}

fn save_results(name: &str, events: Vec<Event>) {
    //    events.sort_by(|a, b| a.tick.cmp(&b.tick));
    const REPORT_DIR: &'static str = "reports";

    std::fs::create_dir(REPORT_DIR).ok();

    use std::io::Write;
    let path = REPORT_DIR.to_owned() + "/" + name + ".csv";
    let mut report = std::fs::File::create(&path).unwrap();

    for event in events {
        match event {
            Event::StrParam(name, value) => report.write(format!("{},{}\n", name, value).as_bytes()).unwrap(),
            Event::Float(name, value) => report.write(format!("{},{}\n", name, value).as_bytes()).unwrap(),
            Event::TimedFloat(tick, name, value) => report.write(format!("{},{},{}\n", tick, name, value).as_bytes()).unwrap(),
        };
    }
    println!("Report saved into: {}", path);
}

struct Model {
    name: String,
    queue: Vec<UserId>,

    user_pool: UserPool,
    users_at_start: u32,
    users_to_gen: u32,

    algorithm: Box<Algoritm>,
    decider: Box<GameDecider>,

    default_skill: f32,
    max_game_length: u32,
    continuous_play_prob: f32,

    real_skill_gen: RandomRangeGen,

    delayed_enter: HashMap<u32, Vec<UserId>>,
    properties: HashMap<&'static str, f32>,
}

impl Model {
    pub fn run(&mut self, ticks: u32, search_delay: u32) -> Vec<Event> {
        println!("Simulating: {}, ticks: {}", self.name, ticks);
        println!("Algorithm: {:?}, will run each {} ticks", self.algorithm, search_delay);
        println!("Game result decider: {:?}", self.decider);
        println!("Real skill level generation strategy: {:?}", self.real_skill_gen);
        println!("Users at the start of the simulation: {}, users to be generated during the simulation: {}", self.users_at_start, self.users_to_gen);
        println!("Maximum Game length: {}, after game join queue probability after: {}", self.max_game_length, self.continuous_play_prob);

        let mut events = Vec::new();
        events.push(Event::StrParam("name", self.name.clone()));

        let users_per_tick = (self.users_to_gen as f32) / (ticks as f32);

        let mut users_to_gen: f32 = self.users_to_gen as f32;
        let mut last_search: u32 = 0;

        for tick in 1..ticks + 1 {
            users_to_gen += users_per_tick;

            let users_to_reuse = self.delayed_enter.remove(&tick);
            let mut users_rejoin = 0;
            match users_to_reuse {
                Some(users) => {
                    users_rejoin = users.len();
                    for id in users {
                        self.join_queue(id, tick)
                    }
                }
                _ => {}
            }

            while users_to_gen >= 1.0 {
                users_to_gen -= 1.0;

                let real_skill = self.real_skill_gen.generate();
                let id = self.user_pool.generate(self.default_skill, real_skill);

                events.push(Event::Float("user_generated_skill", real_skill));

                self.join_queue(id, tick);
            }

            if (last_search + search_delay) <= tick {
                loop {
                    let result = self.algorithm.search(&mut self.queue, &self.user_pool);
                    match result {
                        AlgorithmResult::None => break,
                        AlgorithmResult::Found(game) => {
                            let (skill_t1, rskill_t1) = self.build_team_data(&game.team1);
                            let (skill_t2, rskill_t2) = self.build_team_data(&game.team2);

                            events.push(Event::TimedFloat(tick, "game_created_avg_skill_delta", (skill_t1.avg - skill_t2.avg).abs()));
                            events.push(Event::TimedFloat(tick, "game_created_avg_rskill_delta", (rskill_t1.avg - rskill_t2.avg).abs()));

                            events.push(Event::TimedFloat(tick, "game_created_max_skill_delta", f32::max(skill_t1.max, skill_t2.max) - f32::min(skill_t1.min, skill_t2.min)));
                            events.push(Event::TimedFloat(tick, "game_created_max_rskill_delta", f32::max(rskill_t1.max, rskill_t2.max) - f32::min(rskill_t1.min, rskill_t2.min)));

                            *(self.properties.entry("games_created").or_insert(0.0)) += 1.0;

                            self.on_game_started(tick, game);
                        },
                    }
                }
                last_search = tick;
            }

            self.properties.insert("users_in_queue", self.queue.len() as f32);
            self.properties.insert("avg_skill_error", self.user_pool.get_avg_skill_error());

            let times_in_queue: Vec<u32> = self.queue.iter().map(|id| tick - self.user_pool.get_user(id).get_join_time()).collect();
            let time_in_queue_max = times_in_queue.iter().fold(0, |max, v| if max < *v { *v } else { max });
            self.properties.insert("time_in_queue_max", time_in_queue_max as f32);

            let active_users = self.get_active_users();
            self.properties.insert("active_users", active_users as f32);

            if self.queue.len() == 0 {
                self.properties.insert("time_in_queue_avg", 0.0);
            } else {
                let time_in_queue_sum: u32 = times_in_queue.iter().sum();
                self.properties.insert("time_in_queue_avg", (time_in_queue_sum as f32) / (self.queue.len() as f32));
            }

            for (key, value) in &self.properties {
                events.push(Event::TimedFloat(tick, key, value.clone()));
            }

            if tick % (ticks / 10) == 0 {
                println!("{}", tick);
            }
        }

        events
    }

    fn build_team_data(&self, ids: &Vec<usize>) -> (SkillValue, SkillValue) {
        let skill_levels = ids.iter().map(|id| self.user_pool.get_user(id).get_skill()).collect();
        let real_skill_levels = ids.iter().map(|id| self.user_pool.get_user(id).real_skill).collect();

        (SkillValue::build(&skill_levels), SkillValue::build(&real_skill_levels))
    }

    fn on_game_started(&mut self, tick: u32, game: Game) {
        let mut rng = thread_rng();

        let winner = self.decider.decide(&game, &self.user_pool);
        let game_length = thread_rng().gen_range(1, self.max_game_length);

        game.process(&self.user_pool, winner);

        for id in game.team1 {
            if rng.next_f32() < self.continuous_play_prob {
                self.delayed_enter.entry(tick + game_length).or_insert(Vec::new()).push(id);
            }
        }
        for id in game.team2 {
            if rng.next_f32() < self.continuous_play_prob {
                self.delayed_enter.entry(tick + game_length).or_insert(Vec::new()).push(id);
            }
        }
    }

    fn join_queue(&mut self, id: UserId, tick: u32) {
        self.user_pool.get_user(&id).set_join_time(tick);
        self.queue.push(id);
    }

    fn get_active_users(&self) -> u32 {
        let mut sum = self.queue.len() as u32;

        sum
    }
}

enum Event {
    TimedFloat(u32, &'static str, f32),
    Float(&'static str, f32),
    StrParam(&'static str, String)
}

struct SkillValue {
    avg: f32,
    min: f32,
    max: f32,
}

impl SkillValue {
    fn build(data: &Vec<f32>) -> SkillValue {
        let max = data.iter().fold(-1. / 0., |max, v| f32::max(max, *v));
        let min = data.iter().fold(1. / 0., |min, v| f32::min(min, *v));
        let sum: f32 = data.iter().sum();

        SkillValue {
            min: min,
            max: max,
            avg: sum / (data.len() as f32),
        }
    }
}
