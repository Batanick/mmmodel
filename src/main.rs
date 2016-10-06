#![allow(dead_code)]

extern crate rand;

mod entities;
mod stats;

use stats::Event;
use entities::*;

fn main() {
    let ticks = 1000;
    let users_to_gen = 100;

    let mut model = Model {
        queue: Vec::new(),
        user_pool: UserPool::new(),
        algorithm: Box::new(FIFOAlgorithm {
            team_size: 5,
        }),
        decider: Box::new(SkillLevelDecider {}),
    };
    model.run(ticks, users_to_gen);
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
            events.push(log);
        }

        println!("in queue:{}", self.queue.len());

        events
    }
}

fn save_results(events: Vec<Vec<stats::Event>>) {

}