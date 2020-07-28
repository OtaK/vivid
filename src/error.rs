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
    #[error(r#"Vivid detected both AMD and Nvidia drivers on your system.
Please launch the app with the appropriate flag to choose which driver you use to display."#r)]
    #[allow(dead_code)]
    DualDriversDetected,
    #[error("Vivid couldn't detect any GPU on your system. Is your computer okay?")]
    #[allow(dead_code)]
    NoGpuDetected,
    #[error("Vivid couldn't detect any Displays on your system. How are you seeing this?")]
    NoDisplayDetected,
    #[error(transparent)]
    NvAPIError(#[from] nvapi_hi::sys::Status),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type VividResult<T> = Result<T, VividError>;
