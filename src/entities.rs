use rand::{thread_rng, Rng};

use std::f32::consts::PI;
use std::cell::Cell;

pub type UserId = usize;

pub struct UserData {
    id: UserId,
    pub skill: f32,
    pub real_skill: f32,

    join_time: Cell<u32>,
}

impl UserData {
    fn new(id: UserId, initial_skill: f32, real_skill: f32) -> UserData {
        UserData {
            id: id,
            skill: initial_skill,
            real_skill: real_skill,
            join_time: Cell::new(0),
        }
    }

    pub fn set_join_time(&self, join_time: u32) {
        self.join_time.set(join_time);
    }

    pub fn get_join_time(&self) -> u32 {
        self.join_time.get()
    }
}

pub struct UserPool {
    users: Vec<UserData>,
}

impl UserPool {
    pub fn new() -> UserPool {
        UserPool {
            users: Vec::new(),
        }
    }

    pub fn generate(&mut self, initial_skill: f32, real_skill: f32) -> UserId {
        let id = self.users.len();
        self.users.push(UserData::new(id, initial_skill, real_skill));
        id
    }

    pub fn get_user(&self, id: &UserId) -> &UserData {
        &self.users[*id]
    }
}

pub struct Stats {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct Game {
    pub team1: Vec<UserId>,
    pub team2: Vec<UserId>,
}

impl Game {
    pub fn new(team1: Vec<UserId>, team2: Vec<UserId>) -> Game {
        Game {
            team1: team1,
            team2: team2,
        }
    }
}

#[derive(PartialEq)]
pub enum AlgorithmResult {
    None,
    Found(Game),
}

pub trait GameDecider {
    fn decide(&self, game: &Game) -> i32;
}

pub struct SkillLevelDecider {}

impl GameDecider for SkillLevelDecider {
    fn decide(&self, game: &Game) -> i32 {
        let skill1 = game.team1.iter().fold(0, |mut sum, &x| {
            sum += x;
            sum
        });
        let skill2 = game.team2.iter().fold(0, |mut sum, &x| {
            sum += x;
            sum
        });

        if skill1 == skill2 {
            if thread_rng().gen() {
                return 1;
            } else {
                return 2;
            }
        }

        if skill1 > skill2 {
            return 1;
        } else {
            return 2;
        }
    }
}

pub trait Algoritm {
    fn search(&self, queue: &mut Vec<UserId>, pool: &UserPool) -> AlgorithmResult;
}

pub struct SimpleUserGenerator {
    pub skill: f32,
}

pub fn peek_random<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.len() == 0 {
        return Option::None;
    }

    let index = thread_rng().gen_range(0, vec.len());
    Option::Some(vec.remove(index))
}

pub struct RandomPeekAlgorithm {
    team_size: usize,
}

impl Algoritm for RandomPeekAlgorithm {
    fn search(&self, queue: &mut Vec<UserId>, _: &UserPool) -> AlgorithmResult {
        if queue.len() < (self.team_size * 2) {
            return AlgorithmResult::None;
        }

        let mut team1 = Vec::new();
        let mut team2 = Vec::new();

        for _ in 0..self.team_size {
            team1.push(peek_random(queue).unwrap());
            team2.push(peek_random(queue).unwrap());
        }

        AlgorithmResult::Found(Game::new(team1, team2))
    }
}

pub struct SkillLevelAlgorithm {
    pub team_size: usize,
    pub size_factor: f32,
}

impl Algoritm for SkillLevelAlgorithm {
    fn search(&self, queue: &mut Vec<UserId>, pool: &UserPool) -> AlgorithmResult {
        if (queue.len() as f32) < ((self.team_size as f32) * self.size_factor * 2.0) {
            return AlgorithmResult::None;
        }

        let queue_sum = queue.iter().fold(0.0, |sum, id| sum + pool.get_user(id).skill);
        let queue_avg = queue_sum / (queue.len() as f32);

        let mut team1 = Vec::new();
        let mut team2 = Vec::new();

        while team1.len() < self.team_size || team2.len() < self.team_size {
            let team1_active = team1.len() < team2.len();

            let (active_team, opp_team): (&mut Vec<UserId>, &mut Vec<UserId>) = if team1_active { (&mut team1, &mut team2) } else { (&mut team2, &mut team1) };

            let active_team_skill = active_team.iter().fold(0.0, |sum, v| sum + pool.get_user(v).skill);
            let opponent_team_skill = opp_team.iter().fold(0.0, |sum, v| sum + pool.get_user(v).skill);

            let delta = opponent_team_skill - active_team_skill;
            let avg_skill =
            if active_team.len() == 0 && opp_team.len() == 0
                { queue_avg } else { (opponent_team_skill + active_team_skill) / ((active_team.len() + opp_team.len()) as f32) };

            let desired_skill = avg_skill + delta;

            let (index, _) = queue.iter().enumerate()
                .map(|(index, id)| (index, (pool.get_user(id).skill - desired_skill).abs()))
                .fold((usize::max_value(), 1.0 / 0.0), |i, v| if v.1 > i.1 { i } else { (v.0, v.1) });

            assert!(index != usize::max_value());

            let candidate = queue.remove(index);
//            println!("required:{}, found:{}", desired_skill, pool.get_user(&candidate).skill);

            active_team.push(candidate)
        }

        AlgorithmResult::Found(Game::new(team1, team2))
    }
}

pub struct FIFOAlgorithm {
    pub team_size: usize,
}

impl Algoritm for FIFOAlgorithm {
    fn search(&self, queue: &mut Vec<UserId>, _: &UserPool) -> AlgorithmResult {
        if queue.len() < (self.team_size * 2) {
            return AlgorithmResult::None;
        }

        let mut team1 = Vec::new();
        let mut team2 = Vec::new();

        // queue is ordered by join time
        for _ in 0..self.team_size {
            team1.push(queue.remove(0));
            team2.push(queue.remove(0));
        }

        AlgorithmResult::Found(Game::new(team1, team2))
    }
}

pub enum DistributionType {
    Uniform,
    Normal
}

pub struct RandomRangeGen {
    min: f32,
    max: f32,
    distribution: DistributionType,
}

impl RandomRangeGen {
    pub fn new(min: f32, max: f32, distribution: DistributionType) -> RandomRangeGen {
        RandomRangeGen {
            min: min,
            max: max,
            distribution: distribution,
        }
    }

    pub fn generate(&self) -> f32 {
        let rand = match self.distribution {
            DistributionType::Uniform => (thread_rng().next_f32()),
            DistributionType::Normal => {
                // https://en.wikipedia.org/wiki/Box%E2%80%93Muller_transform
                let u1 = thread_rng().next_f32();
                let u2 = thread_rng().next_f32();
                let mut result = (-2.0 * u1.ln()).sqrt() * ((2.0 * PI * u2).cos());

                if result > 1.0 {
                    result = 1.0;
                }

                if result < -1.0 {
                    result = -1.0;
                }

                    (result + 1.0) * 0.5
            }
        };

        return ((self.max - self.min) * rand) + self.min;
    }
}

// ============================ TESTS ============================

#[test]
fn test_skill_empty_queue() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 2.0,
    };

    let pool = UserPool::new();
    let mut queue = Vec::new();

    assert!(algorithm.search(&mut queue, &pool) == AlgorithmResult::None);
}

#[test]
fn test_skill_small_queue() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 2.0,
    };

    let mut pool = UserPool::new();
    let mut queue = Vec::new();

    for _ in 0..15 {
        queue.push(pool.generate(500.0, 500.0))
    }

    assert!(algorithm.search(&mut queue, &pool) == AlgorithmResult::None);
}


#[test]
fn test_skill_found() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 2.0,
    };

    let mut pool = UserPool::new();
    let mut queue = Vec::new();

    for _ in 0..20 {
        queue.push(pool.generate(500.0, 500.0))
    }

    let result = algorithm.search(&mut queue, &pool);

    match result {
        AlgorithmResult::Found(game) => {
            assert!(game.team1.len() == 5);
            assert!(game.team2.len() == 5);
        }
        _ => panic!("Incorrect result")
    }

    assert!(queue.len() == 10);
}

#[test]
fn test_clustered_queue() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 2.0,
    };

    let mut pool = UserPool::new();
    let mut queue = Vec::new();

    for _ in 0..15 {
        queue.push(pool.generate(500.0, 500.0))
    }

    for _ in 0..10 {
        queue.push(pool.generate(10000.0, 10000.0))
    }

    thread_rng().shuffle(&mut queue);
    let result = algorithm.search(&mut queue, &pool);

    println!("{:?}", queue);
    assert!(queue.len() == 15);

    let mut high_counter = 0;
    let mut avg_counter = 0;
    for id in queue {
        match pool.get_user(&id).skill {
            10000.0 => { high_counter += 1 }
            500.0 => { avg_counter += 1 }
            _ => panic!()
        }
    }
    assert!(high_counter == 10);
    assert!(avg_counter == 5);

    match result {
        AlgorithmResult::Found(game) => {
            println!("{:?}", game);
            assert!(game.team1.len() == 5);
            assert!(game.team2.len() == 5);
        }
        _ => panic!("Incorrect result")
    }
}


#[test]
fn test_skill_level_sum() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 1.0,
    };

    let mut pool = UserPool::new();
    let mut queue = Vec::new();

    for _ in 0..5 {
        queue.push(pool.generate(450.0, 450.0))
    }

    for _ in 0..5 {
        queue.push(pool.generate(550.0, 550.0))
    }

    for _ in 0..9 {
        queue.push(pool.generate(10000.0, 10000.0))
    }

    thread_rng().shuffle(&mut queue);
    let result = algorithm.search(&mut queue, &pool);

    println!("Queue: {:?}", queue);
    assert!(queue.len() == 9);

    let mut high_counter = 0;
    for id in queue {
        match pool.get_user(&id).skill {
            10000.0 => { high_counter += 1 }
            _ => panic!()
        }
    }
    assert!(high_counter == 9);

    match result {
        AlgorithmResult::Found(game) => {
            println!("{:?}", game);
            assert!(game.team1.len() == 5);
            assert!(game.team2.len() == 5);

            let team1_sum = game.team1.iter().fold(0.0, |sum, id| sum + pool.get_user(&id).skill);
            let team2_sum = game.team2.iter().fold(0.0, |sum, id| sum + pool.get_user(&id).skill);
            assert!((team1_sum - team2_sum).abs() == 100.0);
        }
        _ => panic!("Incorrect result")
    }
}