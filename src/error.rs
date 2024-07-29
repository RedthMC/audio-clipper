use std::{fmt::Display, sync::PoisonError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoConnection,
    CantConnect,
    SkillIssue,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoConnection => write!(f, "No connection to vc"),
            Error::CantConnect => write!(f, "Failed to connect"),
            Error::SkillIssue => write!(f, "Internal error"),
        }
    }
}

impl From<serenity::Error> for Error {
    fn from(_: serenity::Error) -> Self {
        Error::SkillIssue
    }
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
