#![allow(dead_code)]

extern crate rand;

mod entities;
mod stats;

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
    };
    model.run(ticks, users_to_gen);
}

struct Model {
    queue: Vec<UserId>,
    user_pool: UserPool,

    algorithm: Box<Algoritm>,
}

impl Model {
    pub fn run(&mut self, ticks: i32, users: u32) {
        let skill = 1500.0;
        let users_per_tick = (users as f32) / (ticks as f32);

        let mut users_to_gen: f32 = 0.0;
        let mut games_created = 0;
        for _ in 0..ticks {
            users_to_gen += users_per_tick;

            while users_to_gen >= 1.0 {
                users_to_gen -= 1.0;

                self.queue.push(self.user_pool.generate(skill, skill));
            }

            loop {
                let result = self.algorithm.search(&mut self.queue);
                match result {
                    AlgorithmResult::None => break,
                    AlgorithmResult::Found(game) => games_created += 1,
                }
            }
        }

        println!("in queue:{}", self.queue.len());
        println!("games created:{}", games_created);
    }
}