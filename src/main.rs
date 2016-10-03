mod entities;

use std::env;

use entities::*;

fn main() {
    let ticks = 10000;

    let users_to_gen: f32 = 100.0;
    let users_per_tick = users_to_gen / (ticks as f32);

    generate(ticks, users_per_tick);
}

fn generate(ticks: i32, users_per_tick: f32) {
    let mut queue = Queue::new();

    let generator = entities::SimpleUserGenerator {
        skill: 1500.0,
    };

    let mut users_to_gen : f32 = 0.0;
    for i in 1..ticks {
        users_to_gen += users_per_tick;

        while  users_to_gen >= 1.0 {
            users_to_gen -= 1.0;
            queue.add(generator.generate());
        }
    }

    println!("in queue:{}", queue.size());
}