pub struct UserData {
    score: f32,
    realScore: f32,
}

pub struct Queue {
    users: Vec<UserData>,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            users: Vec::new(),
        }
    }

    pub fn add(&mut self, user: UserData) {
        self.users.push(user);
    }

    pub fn size(&self) -> usize {
        self.users.len()
    }
}

impl UserData {
    pub fn new(score: f32) -> UserData {
        UserData {
            score: score,
            realScore: score,
        }
    }
}

impl UserData {
    pub fn new(initialScore: f32, realScore: f32) -> UserData {
        UserData {
            score: initialScore,
            realScore: realScore,
        }
    }
}

enum QueueRunAlgorithm {}

pub trait UserGenerator {
    fn generate(&self) -> UserData;
}

trait QueueAlgorithm {}

pub struct SimpleUserGenerator {
    pub skill: f32,
}

impl UserGenerator for SimpleUserGenerator {
    fn generate(&self) -> UserData {
        UserData {
            score: self.skill,
            realScore: self.skill,
        }
    }
}