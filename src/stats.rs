use entities;

pub enum Event {
    UserJoinedQueue(entities::UserId),
    GameCreated(entities::Game),
    GamePlayed(entities::Game),
    UserPlayed(entities::UserId, bool),
    UsersInQueue(u32),
}