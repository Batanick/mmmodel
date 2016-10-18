#![allow(dead_code)]

extern crate rand;
extern crate clap;

mod entities;

use entities::*;

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
        .arg(Arg::with_name("users")
            .short("u")
            .help("Amount of users to be generated")
            .default_value("1000")
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

        .get_matches();

    let ticks = params.value_of("time").unwrap().parse::<u32>().unwrap();
    let users_to_gen = params.value_of("users").unwrap().parse::<u32>().unwrap();
    let default_skill_level = params.value_of("skill").unwrap().parse::<f32>().unwrap();
    let real_skill_min = params.value_of("real_skill_min").unwrap().parse::<f32>().unwrap();
    let real_skill_max = params.value_of("real_skill_max").unwrap().parse::<f32>().unwrap();

    let mut model = Model {
        queue: Vec::new(),
        user_pool: UserPool::new(),
        algorithm: Box::new(FIFOAlgorithm {
            team_size: 5,
        }),
        decider: Box::new(SkillLevelDecider {}),
        properties: HashMap::new(),
        default_skill: default_skill_level,
        real_skill_gen: RandomRangeGen::new(real_skill_min, real_skill_max, DistributionType::Uniform),
    };

    let log = model.run(ticks, users_to_gen);

    save_results(log);
}

fn save_results(mut events: Vec<Event>) {
    events.sort_by(|a, b| a.tick.cmp(&b.tick));
    const REPORT_DIR: &'static str = "reports";

    std::fs::create_dir(REPORT_DIR).ok();

    use std::io::Write;
    let mut report = std::fs::File::create(REPORT_DIR.to_owned() + "/report.csv").unwrap();

    for event in events {
        report.write(format!("{},{},{}\n", event.tick, event.key, event.value).as_bytes()).unwrap();
    }
}

struct Model {
    queue: Vec<UserId>,
    user_pool: UserPool,

    algorithm: Box<Algoritm>,
    decider: Box<GameDecider>,

    default_skill: f32,
    real_skill_gen: RandomRangeGen,

    properties: HashMap<&'static str, f32>,
}

impl Model {
    pub fn run(&mut self, ticks: u32, users: u32) -> Vec<Event> {
        let users_per_tick = (users as f32) / (ticks as f32);

        let mut users_to_gen: f32 = 0.0;
        let mut events = Vec::new();
        for tick in 1..ticks + 1 {
            users_to_gen += users_per_tick;

            events.push(Event::new(tick, "users_joined_queue", users_to_gen.floor() as f32));
            while users_to_gen >= 1.0 {
                users_to_gen -= 1.0;

                let real_skill = self.real_skill_gen.generate();
                let id = self.user_pool.generate(self.default_skill, real_skill);

                events.push(Event::timeless("user_generated_skill", real_skill));
                self.queue.push(id);
            }


            loop {
                let result = self.algorithm.search(&mut self.queue);
                match result {
                    AlgorithmResult::None => break,
                    AlgorithmResult::Found(game) => {
//                        let result = self.decider.decide(&game);
                        let avg_skill_t1 = game.team1.iter().fold(0.0, |sum, id| sum + self.user_pool.get_user(id).real_skill) / (game.team1.len() as f32) ;
                        let avg_skill_t2 = game.team2.iter().fold(0.0, |sum, id| sum + self.user_pool.get_user(id).real_skill) / (game.team2.len() as f32) ;;

                        events.push(Event::new(tick, "game_created_skill_delta", (avg_skill_t1 - avg_skill_t2).abs()));

                        *(self.properties.entry("games_created").or_insert(0.0)) += 1.0;
                    },
                }
            }

            events.push(Event::new(tick, "users_in_queue", self.queue.len() as f32));

            for (key, value) in &self.properties {
                events.push(Event::new(tick, key, value.clone()));
            }
        }

        println!("in queue:{}", self.queue.len());

        events
    }
}

struct Event {
    pub tick: u32,
    pub key: &'static str,
    pub value: f32,
}

impl Event {
    pub fn new(tick: u32, key: &'static str, value: f32) -> Event {
        Event {
            tick: tick,
            key: key,
            value: value
        }
    }

    pub fn timeless(key: &'static str, value: f32) -> Event {
        Event {
            tick: 0,
            key: key,
            value: value
        }
    }
}

