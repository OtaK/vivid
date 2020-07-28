const DEFAULT_CONFIG_FILENAME: &str = "vivid.toml";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Program {
    pub exe_name: String,
    pub vibrance: u8,
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
    #[allow(dead_code)]
    pub fn load() -> crate::VividResult<Self> {
        use std::io::Read as _;
        let mut path = std::env::current_exe()?;
        path.push(DEFAULT_CONFIG_FILENAME);
        let mut file = std::fs::File::open(path)?;
        let mut file_contents = vec![];
        file.read_to_end(&mut file_contents)?;
        toml::from_slice(&file_contents).map_err(Into::into)
    }

    pub fn test() -> Self {
        Self {
            desktop_vibrance: 50,
            program_settings: vec![Program {
                exe_name: "Code.exe".into(),
                vibrance: 90,
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
