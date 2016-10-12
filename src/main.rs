#![allow(dead_code)]

extern crate rand;

mod entities;
mod stats;

use stats::Event;
use entities::*;

fn main() {
    let ticks = 10000;
    let users_to_gen = 100;

    let mut model = Model {
        queue: Vec::new(),
        user_pool: UserPool::new(),
        algorithm: Box::new(FIFOAlgorithm {
            team_size: 5,
        }),
        decider: Box::new(SkillLevelDecider {}),
    };

    let log = model.run(ticks, users_to_gen);

    save_results(log);
}

struct Model {
    queue: Vec<UserId>,
    user_pool: UserPool,

    algorithm: Box<Algoritm>,
    decider: Box<GameDecider>,
}

impl Model {
    pub fn run(&mut self, ticks: i32, users: u32) -> Vec<Vec<stats::Event>> {
        let skill = 1500.0;
        let users_per_tick = (users as f32) / (ticks as f32);

        let mut users_to_gen: f32 = 0.0;
        let mut events: Vec<Vec<stats::Event>> = Vec::with_capacity(ticks as usize);
        for _ in 0..ticks {
            let mut log = Vec::new();

            users_to_gen += users_per_tick;

            while users_to_gen >= 1.0 {
                users_to_gen -= 1.0;

                let id = self.user_pool.generate(skill, skill);
                self.queue.push(id);
                log.push(Event::UserJoinedQueue(id));
            }

            loop {
                let result = self.algorithm.search(&mut self.queue);
                match result {
                    AlgorithmResult::None => break,
                    AlgorithmResult::Found(game) => {
                        let result = self.decider.decide(&game);

                        {
                            for id in &game.team1 {
                                log.push(Event::UserPlayed(id.clone(), result == 1));
                            }
                            for id in &game.team2 {
                                log.push(Event::UserPlayed(id.clone(), result == 2));
                            }
                        }

                        log.push(Event::GameCreated(game));
                    },
                }
            }
            
            log.push(Event::UsersInQueue(self.queue.len() as u32));

            events.push(log);
        }

        println!("in queue:{}", self.queue.len());

        events
    }
}

fn save_results(events: Vec<Vec<stats::Event>>) {
    const REPORT_DIR: &'static str = "reports";

    std::fs::create_dir(REPORT_DIR).ok();

    use std::io::Write;
    let mut report = std::fs::File::create(REPORT_DIR.to_owned() + "/report.csv").unwrap();

    for tick in 0..events.len() {
        let ref log = events[tick];
        for event in log {
            let event_log = match event {
                &Event::UserJoinedQueue(id) => format!("user_joined_queue, {}", id),
                &Event::GameCreated(_) => "game_created".to_owned(),
                &Event::GamePlayed(_) => "game_played".to_owned(),
                &Event::UserPlayed(id, won) => format!("user_game_played,{},{}", id, won),
                &Event::UsersInQueue(count) => format!("users_in_queue,{}",count),
            };

            report.write(format!("{},{}\n", tick, event_log).as_bytes()).unwrap();
        }
    }
}
