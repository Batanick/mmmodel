#![allow(dead_code)]

extern crate rand;

mod entities;

use entities::*;

fn main() {
    let ticks = 1000;

    let users_to_gen: f32 = 100.0;
    let users_per_tick = users_to_gen / (ticks as f32);

    generate(ticks, users_per_tick);
}

fn generate(ticks: i32, users_per_tick: f32) {
    let mut queue = Vec::new();
    let mut user_pool = UserPool::new();

    let algorithm = FIFOAlgorithm {
        team_size : 5,
    };

    let skill = 1500.0;

    let mut users_to_gen : f32 = 0.0;
    let mut games_created = 0;
    for _ in 0..ticks {
        users_to_gen += users_per_tick;

        while  users_to_gen >= 1.0 {
            users_to_gen -= 1.0;

            queue.push(user_pool.generate(skill, skill));
        }

        loop {
            let result = algorithm.search(&mut queue);
            match result {
                AlgorithmResult::None  => break,
                AlgorithmResult::Found(game) => games_created += 1,
            }
        }

    }

    println!("in queue:{}", queue.len());
    println!("games created:{}", games_created);
}