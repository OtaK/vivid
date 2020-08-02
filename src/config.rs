use winapi::um::winuser::SW_SHOWNORMAL;
use winapi::shared::ntdef::NULL;
use winapi::um::shellapi::ShellExecuteA;
use crate::error::VividError;

pub const DEFAULT_CONFIG_FILENAME: &str = "vivid.toml";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Program {
    pub exe_name: String,
    pub vibrance: u8,
    pub fullscreen_only: Option<bool>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    desktop_vibrance: u8,
    program_settings: Vec<Program>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            desktop_vibrance: 50,
            program_settings: vec![],
        }
    }
}

impl Config {
    fn sample() -> crate::VividResult<Self> {
        let vibrance = (*crate::GPU).as_ref()?.read().get_vibrance()?;
        let mut default = Self::default();
        default.desktop_vibrance = vibrance;
        default.program_settings.push(Program {
            exe_name: "sample_program.exe".into(),
            vibrance,
            fullscreen_only: Some(false),
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

    #[allow(dead_code)]
    pub fn load() -> crate::VividResult<Self> {
        use std::io::Read as _;
        let mut file = Self::load_file()?;
        let mut file_contents = vec![];
        file.read_to_end(&mut file_contents)?;
        toml::from_slice(&file_contents).map_err(Into::into)
    }

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

    pub fn test() -> Self {
        Self {
            desktop_vibrance: 50,
            program_settings: vec![Program {
                exe_name: "Code.exe".into(),
                vibrance: 90,
                fullscreen_only: None,
            }],
        }
    }

    pub fn programs(&self) -> &Vec<Program> {
        &self.program_settings
    }

    pub fn default_vibrance(&self) -> u8 {
        self.desktop_vibrance
    }
}
