#[derive(Debug, thiserror::Error)]
pub enum WindowsHookError {
    #[error("Failed to hook w32 event [SetWinEventHook]")]
    SetWinEventHook(std::io::Error),
    #[error("Failed to unhook w32 event [UnhookWinEvent]")]
    UnhookWinEvent(std::io::Error),
    #[error("There's no hook to unhook! You should call register() first.")]
    NoHookToUnRegister(std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum VividError {
    #[error(transparent)]
    SelfError(#[from] &'static Self),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerializeError(#[from] toml::ser::Error),
    #[error(transparent)]
    DeserializeError(#[from] toml::de::Error),
    #[error(transparent)]
    WindowsHookError(#[from] WindowsHookError),
    #[error(r#"Vivid detected both AMD and Nvidia drivers on your system.
Please launch the app with the appropriate flag to choose which driver you use to display."#r)]
    DualDriversDetected,
    #[error("Vivid couldn't detect any GPU on your system. Is your computer okay?")]
    NoGpuDetected,
    #[error("Vivid couldn't detect any Displays on your system. How are you seeing this?")]
    NoDisplayDetected,
    #[error("Vivid couldn't inspect process #{0}. Probably because it's system owned.")]
    ProcessNotAvailable(usize),
    #[error(transparent)]
    NvAPIError(#[from] nvapi_hi::sys::Status),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type VividResult<T> = Result<T, VividError>;
