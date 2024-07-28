use std::sync::PoisonError;


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoConnection,
    CantConnect,
    SkillIssue,
}

impl From<mp3lame_encoder::BuildError> for Error {
    fn from(_: mp3lame_encoder::BuildError) -> Self {
        Error::SkillIssue
    }
}
impl From<mp3lame_encoder::EncodeError> for Error {
    fn from(_: mp3lame_encoder::EncodeError) -> Self {
        Error::SkillIssue
    }
}
impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::SkillIssue
    }
}
impl From<songbird::error::JoinError> for Error {
    fn from(_: songbird::error::JoinError) -> Self {
        Error::CantConnect
    }
}
