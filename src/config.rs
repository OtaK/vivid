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
        // SAFETY: Data safety is ensured by the fact that the crate::GPU mutable static is wrapped in a RwLock
        let vibrance = unsafe { crate::GPU.as_ref()?.write().get_vibrance()? };
        let default = Self {
            desktop_vibrance: vibrance,
            program_settings: vec![Program {
                exe_name: "sample_program.exe".into(),
                vibrance,
                fullscreen_only: Some(false),
                resolution: None,
            }],
            ..Default::default()
        };

        Ok(default)
    }

    fn config_path() -> crate::VividResult<std::path::PathBuf> {
        let mut path = std::env::current_exe()?;
        path.set_file_name(DEFAULT_CONFIG_FILENAME);
        Ok(path)
    }

    fn load_file(maybe_path: Option<String>) -> crate::VividResult<std::fs::File> {
        use std::io::Write as _;
        let path = maybe_path.map_or_else(Self::config_path, |path| Ok(path.into()))?;
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
    pub fn load(maybe_path: Option<String>) -> crate::VividResult<Self> {
        use std::io::Read as _;
        let mut file = Self::load_file(maybe_path)?;
        let mut file_contents = vec![];
        file.read_to_end(&mut file_contents)?;
        toml::from_slice(&file_contents).map_err(Into::into)
    }

    /// Launches windows standard editor for this file.
    pub fn edit() -> crate::VividResult<()> {
        let _ = Self::load_file(None)?;
        // SAFETY: The following unwrap is safe because:
        // - Self::config_path() fails on not returning a valid UTF-8 path that exists, and this error is handled
        // - Thus the Self::config_path()?.to_str() is infaillible
        let config_path = Self::config_path()?;
        let file_path = config_path.to_str().unwrap();
        // SAFETY: Trivial call to ShellExecuteA; As long as the lpFile parameter is a valid C-style string pointer, we're good
        let hwnd = unsafe {
            windows::Win32::UI::Shell::ShellExecuteA(
                windows::Win32::Foundation::HWND::default(),
                windows::Win32::Foundation::PSTR::default(),
                file_path,
                windows::Win32::Foundation::PSTR::default(),
                windows::Win32::Foundation::PSTR::default(),
                windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL as i32,
            )
        };

        if hwnd as u32 > windows::Win32::System::WindowsProgramming::HINSTANCE_ERROR {
            Ok(())
        } else {
            Err(VividError::windows_error())
        }
    }

    pub fn vibrance_for_program(&self, program_exe: &str) -> Option<(u8, bool)> {
        self.program_settings
            .iter()
            .find(|&program| program.exe_name == program_exe)
            .map(|program| {
                (
                    program.vibrance,
                    program.fullscreen_only.unwrap_or_default(),
                )
            })
    }

    pub fn default_vibrance(&self) -> u8 {
        self.desktop_vibrance
    }
}
