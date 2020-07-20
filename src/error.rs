#[derive(Debug, thiserror::Error)]
pub enum VividError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerializeError(#[from] toml::ser::Error),
    #[error(transparent)]
    DeserializeError(#[from] toml::de::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}

pub type VividResult<T> = Result<T, VividError>;
