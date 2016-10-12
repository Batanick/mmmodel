use entities;

pub enum Event {
    UserJoinedQueue(entities::UserId),
    GamesCreated(u32),
    UserPlayed(entities::UserId, bool),
    UsersInQueue(u32),
}
