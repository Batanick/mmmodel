use rand::{thread_rng, Rng};

use std::f32::consts::PI;
use std::cell::Cell;

use std::fmt::Debug;

pub type UserId = usize;

#[derive(Debug)]
pub struct UserData {
    pub id: UserId,
    pub real_skill: f32,

    skill: Cell<f32>,
    join_time: Cell<u32>,

    use_real_skill: bool,
}

impl UserData {
    fn new(id: UserId, initial_skill: f32, real_skill: f32, use_real_skill: bool) -> UserData {
        UserData {
            id: id,
            skill: Cell::new(initial_skill),
            real_skill: real_skill,
            join_time: Cell::new(0),
            use_real_skill: use_real_skill,
        }
    }

    pub fn set_join_time(&self, join_time: u32) {
        self.join_time.set(join_time);
    }

    pub fn get_join_time(&self) -> u32 {
        self.join_time.get()
    }

    pub fn update_skill(&self, delta: f32) {
        if self.use_real_skill {
            return;
        }

        self.skill.set(self.skill.get() + delta);
    }

    pub fn get_skill(&self) -> f32 {
        if self.use_real_skill { self.real_skill } else { self.skill.get() }
    }
}

#[derive(Debug)]
pub struct UserPool {
    users: Vec<UserData>,
    pub use_real_skill: bool,
}

impl UserPool {
    pub fn new(use_real_skill: bool) -> UserPool {
        UserPool {
            users: Vec::new(),
            use_real_skill: use_real_skill,
        }
    }

    pub fn generate(&mut self, initial_skill: f32, real_skill: f32) -> UserId {
        let id = self.users.len();
        self.users.push(UserData::new(id, initial_skill, real_skill, self.use_real_skill));
        id
    }

    pub fn get_user(&self, id: &UserId) -> &UserData {
        &self.users[*id]
    }

    pub fn get_avg_skill_error(&self) -> f32 {
        let sum = self.users.iter().fold(0.0, |sum, data| sum + (data.real_skill - data.get_skill()).abs());
        sum / (self.users.len() as f32)
    }
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

    pub fn process(&self, user_pool: &UserPool, win_team: u32) {
        let (winners, losers) = match win_team {
            1 => (&self.team1, &self.team2),
            2 => (&self.team2, &self.team1),
            _ => panic!()
        };

        let winners_avg = winners.iter().fold(0.0, |sum, id| sum + user_pool.get_user(id).get_skill()) / (winners.len() as f32);
        let losers_avg = losers.iter().fold(0.0, |sum, id| sum + user_pool.get_user(id).get_skill()) / (losers.len() as f32);

        let r_win = 10.0_f32.powf(winners_avg / 400.0);
        let r_lose = 10.0_f32.powf(losers_avg / 400.0);

        let e_win = r_win / (r_win + r_lose);
        let e_lose = r_lose / (r_win + r_lose);

        static K_FACTOR: f32 = 32.0;

        let winner_delta = K_FACTOR * (1.0 - e_win);
        let loser_delta = K_FACTOR * (-e_lose);

        for id in winners {
            user_pool.get_user(id).update_skill(winner_delta);
        }

        for id in losers {
            user_pool.get_user(id).update_skill(loser_delta);
        }
    }
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum AlgorithmResult {
    None,
    Found(Game),
}

pub trait GameDecider: Debug {
    fn decide(&self, game: &Game, pool: &UserPool) -> u32;
}

#[derive(Debug)]
pub struct RealSkillLevelDecider {}

impl GameDecider for RealSkillLevelDecider {
    fn decide(&self, game: &Game, pool: &UserPool) -> u32 {
        let skill1 = game.team1.iter().fold(0.0, |sum, id| sum + pool.get_user(id).real_skill);
        let skill2 = game.team2.iter().fold(0.0, |sum, id| sum + pool.get_user(id).real_skill);

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

pub trait Algoritm: Debug {
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

#[derive(Debug)]
pub struct RandomPeekAlgorithm {
    pub team_size: usize,
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

#[derive(Debug)]
pub struct SkillLevelAlgorithm {
    pub team_size: usize,
    pub size_factor: f32,
    pub prefill_factor: f32,
}

impl Algoritm for SkillLevelAlgorithm {
    fn search(&self, queue: &mut Vec<UserId>, pool: &UserPool) -> AlgorithmResult {
        if (queue.len() as f32) < ((self.team_size as f32) * self.size_factor * 2.0) {
            return AlgorithmResult::None;
        }

        let mut team1 = Vec::new();
        let mut team2 = Vec::new();

        let to_add = (self.prefill_factor * (self.team_size as f32) * 2.0) as u32;
        if to_add > 0 {
            for _ in 0..to_add {
                let team_to_add = if team1.len() > team2.len() { &mut team2 } else { &mut team1 };
                team_to_add.push(queue.remove(0));
            }
        }

        while team1.len() < self.team_size || team2.len() < self.team_size {
            let team1_active = team1.len() < team2.len();

            let (active_team, opp_team): (&mut Vec<UserId>, &mut Vec<UserId>) = if team1_active { (&mut team1, &mut team2) } else { (&mut team2, &mut team1) };

            let active_team_skill = active_team.iter().fold(0.0, |sum, v| sum + pool.get_user(v).get_skill());
            let opponent_team_skill = opp_team.iter().fold(0.0, |sum, v| sum + pool.get_user(v).get_skill());

            let delta = opponent_team_skill - active_team_skill;

            let queue_sum = queue.iter().fold(0.0, |sum, id| sum + pool.get_user(id).get_skill());
            let queue_avg = queue_sum / (queue.len() as f32);

            let desired_skill = if active_team.len() != opp_team.len() { delta } else { delta + queue_avg };

            let (index, _) = queue.iter().enumerate()
                .map(|(index, id)| (index, (pool.get_user(id).get_skill() - desired_skill).abs()))
                .fold((usize::max_value(), 1.0 / 0.0), |i, v| if v.1 > i.1 { i } else { (v.0, v.1) });

            assert!(index != usize::max_value());

            let candidate = queue.remove(index);
            //            println!("required:{}, found:{}:{}", desired_skill, candidate, pool.get_user(&candidate).get_skill());

            active_team.push(candidate)
        }

        AlgorithmResult::Found(Game::new(team1, team2))
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum DistributionType {
    Uniform,
    Normal
}

#[derive(Debug)]
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
                let u1 = thread_rng().next_f32();
                let u2 = thread_rng().next_f32();
                // https://en.wikipedia.org/wiki/Box%E2%80%93Muller_transform
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
        prefill_factor: 0.0,
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
        prefill_factor: 0.0,
    };

    let mut pool = UserPool::new();
    let mut queue = Vec::new();

    for _ in 0..15 {
        queue.push(pool.generate(500.0, 500.0))
    }

    assert!(algorithm.search(&mut queue, &pool) == AlgorithmResult::None);
}

#[test]
fn test_skill_small_prefill() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 2.0,
        prefill_factor: 0.1,
    };

    let mut pool = UserPool::new();
    let mut queue = Vec::new();

    queue.push(pool.generate(10000.0, 10000.0));

    for _ in 0..20 {
        queue.push(pool.generate(500.0, 500.0))
    }

    let result = algorithm.search(&mut queue, &pool);

    match result {
        AlgorithmResult::Found(game) => {
            assert!(game.team1.len() == 5);
            assert!(game.team2.len() == 5);

            assert_eq!(10000.0, pool.get_user(game.team1.get(0).unwrap()).get_skill());
        }
        _ => panic!("Incorrect result")
    }
}

#[test]
fn test_skill_small_prefill2() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 2.0,
        prefill_factor: 0.4,
    };

    let mut pool = UserPool::new();
    let mut queue = Vec::new();

    for _ in 0..5 {
        queue.push(pool.generate(10000.0, 10000.0));
    }

    for _ in 0..20 {
        queue.push(pool.generate(500.0, 500.0))
    }

    let result = algorithm.search(&mut queue, &pool);
    println!("{:?}", pool);
    println!("{:?}", queue);
    println!("{:?}", result);

    assert_eq!(15, queue.len());
    assert_eq!(10000.0, pool.get_user(queue.get(0).unwrap()).get_skill());
    for i in 1..queue.len() {
        assert_eq!(500.0, pool.get_user(queue.get(i).unwrap()).get_skill());
    }

    match result {
        AlgorithmResult::Found(game) => {
            assert!(game.team1.len() == 5);
            assert!(game.team2.len() == 5);

            let team1_sum = game.team1.iter().fold(0.0, |sum, id| sum + pool.get_user(id).get_skill());
            let team2_sum = game.team2.iter().fold(0.0, |sum, id| sum + pool.get_user(id).get_skill());
            assert_eq!(team1_sum, team2_sum);
        }
        _ => panic!("Incorrect result")
    }
}


#[test]
fn test_skill_found() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 2.0,
        prefill_factor: 0.0,
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
        prefill_factor: 0.0,
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

    match result {
        AlgorithmResult::Found(game) => {
            println!("{:?}", game);
            assert!(game.team1.len() == 5);
            assert!(game.team2.len() == 5);

            let team1_sum = game.team1.iter().fold(0.0, |sum, id| sum + pool.get_user(id).get_skill());
            let team2_sum = game.team2.iter().fold(0.0, |sum, id| sum + pool.get_user(id).get_skill());
            assert_eq!(team1_sum, team2_sum);
        }
        _ => panic!("Incorrect result")
    }
}

#[test]
fn test_skill_level_sum() {
    let algorithm = SkillLevelAlgorithm {
        team_size: 5,
        size_factor: 1.0,
        prefill_factor: 0.0,
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

    match result {
        AlgorithmResult::Found(game) => {
            println!("{:?}", game);
            assert!(game.team1.len() == 5);
            assert!(game.team2.len() == 5);

            let team1_sum = game.team1.iter().fold(0.0, |sum, id| sum + pool.get_user(&id).get_skill());
            let team2_sum = game.team2.iter().fold(0.0, |sum, id| sum + pool.get_user(&id).get_skill());
            assert!((team1_sum - team2_sum).abs() <= 100.0);
        }
        _ => panic!("Incorrect result")
    }
}

#[test]
fn rating_update() {
    let mut pool = UserPool::new();

    let user1 = pool.generate(2400.0, 0.0);
    let user2 = pool.generate(2000.0, 0.0);

    let game = Game::new(vec!(user1), vec!(user2));

    game.process(&pool, 1);

    assert!((pool.get_user(&user1).get_skill() - 2403.0).abs() < 0.1);
    assert!((pool.get_user(&user2).get_skill() - 1997.0).abs() < 0.1);
}

#[test]
fn rating_update2() {
    let mut pool = UserPool::new();

    let user1 = pool.generate(2400.0, 0.0);
    let user2 = pool.generate(2000.0, 0.0);

    let game = Game::new(vec!(user1), vec!(user2));

    game.process(&pool, 2);

    assert!((pool.get_user(&user1).get_skill() - 2371.0).abs() < 0.1);
    assert!((pool.get_user(&user2).get_skill() - 2029.0).abs() < 0.1);
}

#[test]
fn team_rating_update() {
    let mut pool = UserPool::new();

    let user1 = pool.generate(2800.0, 0.0);
    let user2 = pool.generate(2000.0, 0.0);

    let user3 = pool.generate(1000.0, 0.0);
    let user4 = pool.generate(3000.0, 0.0);

    println!("{:?}", pool);

    let game = Game::new(vec!(user1, user2), vec!(user3, user4));

    game.process(&pool, 2);

    println!("{:?}", pool);

    assert!((pool.get_user(&user1).get_skill() - 2771.0).abs() < 0.1);
    assert!((pool.get_user(&user2).get_skill() - 1971.0).abs() < 0.1);
    assert!((pool.get_user(&user3).get_skill() - 1029.0).abs() < 0.1);
    assert!((pool.get_user(&user4).get_skill() - 3029.0).abs() < 0.1);
}


