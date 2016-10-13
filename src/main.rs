#![allow(dead_code)]

extern crate rand;

mod entities;

use entities::*;

use std::collections::HashMap;

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
}

fn main() {
    let ticks = 10000;
    let users_to_gen = 1056;

    let mut model = Model {
        queue: Vec::new(),
        user_pool: UserPool::new(),
        algorithm: Box::new(FIFOAlgorithm {
            team_size: 5,
        }),
        decider: Box::new(SkillLevelDecider {}),
        properties: HashMap::new(),
    };

    let log = model.run(ticks, users_to_gen);

    save_results(log);
}

struct Model {
    queue: Vec<UserId>,
    user_pool: UserPool,

    algorithm: Box<Algoritm>,
    decider: Box<GameDecider>,

    properties: HashMap<&'static str, f32>,
}

impl Model {
    pub fn run(&mut self, ticks: u32, users: u32) -> Vec<Event> {
        let skill = 1500.0;
        let users_per_tick = (users as f32) / (ticks as f32);

        let mut users_to_gen: f32 = 0.0;
        let mut events = Vec::new();
        for tick in 1..ticks+1 {
            users_to_gen += users_per_tick;

            events.push(Event::new(tick, "users_joined_queue", users_to_gen.floor() as f32));
            while users_to_gen >= 1.0 {
                users_to_gen -= 1.0;

                let id = self.user_pool.generate(skill, skill);
                self.queue.push(id);
            }


            loop {
                let result = self.algorithm.search(&mut self.queue);
                match result {
                    AlgorithmResult::None => break,
                    AlgorithmResult::Found(game) => {
                        let result = self.decider.decide(&game);

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
