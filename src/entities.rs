use rand::{thread_rng, Rng};

pub type UserId = usize;

pub struct UserData {
    id: UserId,
    pub score: f32,
    pub real_score: f32,
}

impl UserData {
    fn new(id: UserId, initial_score: f32, real_score: f32) -> UserData {
        UserData {
            id: id,
            score: initial_score,
            real_score: real_score,
        }
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

    pub fn generate(&mut self, initial_score: f32, real_score: f32) -> UserId {
        let id = self.users.len();
        self.users.push(UserData::new(id, initial_score, real_score));
        id
    }

    pub fn get_user(&self, id: UserId) -> &UserData {
        &self.users[id]
    }
}

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
    fn search(&self, queue: &mut Vec<UserId>) -> AlgorithmResult;
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
    fn search(&self, queue: &mut Vec<UserId>) -> AlgorithmResult {
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

pub struct FIFOAlgorithm {
    pub team_size: usize,
}

impl Algoritm for FIFOAlgorithm {
    fn search(&self, queue: &mut Vec<UserId>) -> AlgorithmResult {
        if queue.len() < (self.team_size * 2) {
            return AlgorithmResult::None;
        }

        let mut team1 = Vec::new();
        let mut team2 = Vec::new();

        for _ in 0..self.team_size {
            team1.push(queue.remove(0));
            team2.push(queue.remove(0));
        }

        AlgorithmResult::Found(Game::new(team1, team2))
    }
}