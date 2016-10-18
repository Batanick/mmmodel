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
                let result = self.algorithm.search(&mut self.queue, &self.user_pool);
                match result {
                    AlgorithmResult::None => break,
                    AlgorithmResult::Found(game) => {
                        let (skill_t1, rskill_t1) = self.build_team_data(&game.team1);
                        let (skill_t2, rskill_t2) = self.build_team_data(&game.team2);

                        events.push(Event::new(tick, "game_created_avg_skill_delta", (skill_t1.avg - skill_t2.avg).abs()));
                        events.push(Event::new(tick, "game_created_avg_rskill_delta", (rskill_t1.avg - rskill_t2.avg).abs()));

                        events.push(Event::new(tick, "game_created_max_skill_delta", f32::max(skill_t1.max, skill_t2.max) - f32::min(skill_t1.min, skill_t2.min)));
                        events.push(Event::new(tick, "game_created_max_rskill_delta", f32::max(rskill_t1.max, rskill_t2.max) - f32::min(rskill_t1.min, rskill_t2.min)));

                        *(self.properties.entry("games_created").or_insert(0.0)) += 1.0;
                    },
                }
            }

            self.properties.insert("users_in_queue", self.queue.len() as f32);

            let times_in_queue:Vec<u32> = self.queue.iter().map(|id| tick - self.user_pool.get_user(id).get_join_time()).collect();
            let time_in_queue_max = times_in_queue.iter().fold(0, |max, v| if max < *v {*v} else {max});
            self.properties.insert("time_in_queue_max", time_in_queue_max as f32);

            if self.queue.len() == 0 {
                self.properties.insert("time_in_queue_avg", 0.0);
            } else {
                let time_in_queue_sum:u32 = times_in_queue.iter().sum();
                self.properties.insert("time_in_queue_avg", (time_in_queue_sum as f32) / (self.queue.len() as f32));
            }

            for (key, value) in &self.properties {
                events.push(Event::new(tick, key, value.clone()));
            }
        }

        events
    }

    fn build_team_data(&self, ids: &Vec<usize>) -> (SkillValue, SkillValue) {
        let skill_levels = ids.iter().map(|id| self.user_pool.get_user(id).skill).collect();
        let real_skill_levels = ids.iter().map(|id| self.user_pool.get_user(id).skill).collect();
        (SkillValue::build(&skill_levels) , SkillValue::build(&real_skill_levels))
    }

}

struct Event {
    pub tick: u32,
    pub key: &'static str,
    pub value: f32,
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
        let sum:f32 = data.iter().sum();

        SkillValue {
            min: min,
            max: max,
            avg: sum / (data.len() as f32),
        }
    }
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

