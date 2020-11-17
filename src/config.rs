use winapi::{
    shared::ntdef::NULL,
    um::{
        winuser::SW_SHOWNORMAL,
        shellapi::ShellExecuteA,
    }
};
use crate::error::VividError;

pub const DEFAULT_CONFIG_FILENAME: &str = "vivid.toml";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoMode {
    /// Screen pixel width
    pub width: u32,
    /// Screen pixel height
    pub height: u32,
    /// Refresh rate
    pub freq: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Program {
    /// Name of the program to react on
    pub exe_name: String,
    /// Vibrance value in percentage to apply when this program comes to foreground.
    pub vibrance: u8,
    /// Only apply settings when the program comes to foreground in FullScreen mode
    pub fullscreen_only: Option<bool>,
    /// Only apply this video mode when this program starts
    pub resolution: Option<VideoMode>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Vibrance to restore when any non-selected program comes to foreground, included explorer.exe
    desktop_vibrance: u8,
    /// Default desktop resolution
    resolution: Option<VideoMode>,
    /// Program-specific settings
    program_settings: Vec<Program>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            desktop_vibrance: 50,
            program_settings: vec![],
            resolution: None,
        }
    }
}

impl Config {
    fn sample() -> crate::VividResult<Self> {
        let vibrance = unsafe { crate::GPU.as_ref()?.read().get_vibrance()? };
        let mut default = Self::default();
        default.desktop_vibrance = vibrance;
        default.program_settings.push(Program {
            exe_name: "sample_program.exe".into(),
            vibrance,
            fullscreen_only: Some(false),
            resolution: None,
        });

        Ok(default)
    }

    fn config_path() -> crate::VividResult<std::path::PathBuf> {
        let mut path = std::env::current_exe()?;
        path.set_file_name(DEFAULT_CONFIG_FILENAME);
        Ok(path)
    }

    fn load_file() -> crate::VividResult<std::fs::File> {
        use std::io::Write as _;
        let path = Self::config_path()?;
        let res = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(path.clone());

        if let Ok(mut file) = res {
            write!(file, "{}", toml::to_string_pretty(&Self::sample()?)?)?;
            Ok(file)
        } else {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .read(true)
                .truncate(false)
                .open(path)?;

            Ok(file)
        }
    }

    /// Loads the configuration file at the standard location (alongside the .exe)
    pub fn load() -> crate::VividResult<Self> {
        use std::io::Read as _;
        let mut file = Self::load_file()?;
        let mut file_contents = vec![];
        file.read_to_end(&mut file_contents)?;
        toml::from_slice(&file_contents).map_err(Into::into)
    }

    /// Launches windows standard editor for this file.
    pub fn edit() -> crate::VividResult<()> {
        let _ = Self::load_file()?;
        let hwnd = unsafe { ShellExecuteA(
            NULL as _,
            NULL as _,
            std::ffi::CString::new(Self::config_path()?.to_str().unwrap().as_bytes()).unwrap().as_ptr(),
            NULL as _,
            NULL as _,
            SW_SHOWNORMAL
        )};

        if hwnd as u32 > 32 {
            Ok(())
        } else {
            return Err(VividError::windows_error());
        }
    }

    pub fn vibrance_for_program(&self, program_exe: &str) -> Option<(u8, bool)> {
        self.program_settings
            .iter()
            .find(|&program| program.exe_name == program_exe)
            .map(|program| (program.vibrance, program.fullscreen_only.unwrap_or_default()))
    }

    pub fn default_vibrance(&self) -> u8 {
        self.desktop_vibrance
    }
}
