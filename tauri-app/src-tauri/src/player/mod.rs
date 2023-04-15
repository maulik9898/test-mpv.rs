pub mod mpv;

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play(String),
    QueNext(String),
    Stop,
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Progress(i64),
    Pause(bool),
    Duration(i64),
    Exit,
}
