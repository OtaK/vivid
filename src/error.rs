
#[derive(Debug, thiserror::Error)]
pub enum WindowsHookError {
    #[error("Failed to hook w32 event [SetWinEventHook]")]
    SetWinEventHook,
    #[error("Failed to unhook w32 event [UnhookWinEvent]")]
    UnhookWinEvent,
    #[error("There's no hook to unhook! You should call register() first.")]
    NoHookToUnRegister,
}

#[derive(Debug, thiserror::Error)]
pub enum VividError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerializeError(#[from] toml::ser::Error),
    #[error(transparent)]
    DeserializeError(#[from] toml::de::Error),
    #[error(transparent)]
    WindowsHookError(#[from] WindowsHookError),
    #[error("Vivid couldn't detect any GPU on your system. Is your computer okay?")]
    NoGpuDetected,
    #[error("Vivid couldn't detect any Displays on your system. How are you seeing this?")]
    NoDisplayDetected,
    #[error(transparent)]
    NvAPIError(#[from] nvapi_hi::sys::Status),
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}

pub type VividResult<T> = Result<T, VividError>;
