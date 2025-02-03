use std::path::{Path, PathBuf};

use anyhow::Result;
use iced::time::Duration;
use serde::Deserialize;

use crate::color::ColorConfig;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(with = "humantime_serde")]
    pub work_expected_duration: Duration,
    pub break_divisor: f32,
    pub auto_break: bool,

    pub audio_file_path: Option<PathBuf>,
    pub mute: bool,

    pub color_config: ColorConfig,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_expected_duration: Duration::from_secs(25 * 60),
            break_divisor: 5.0,
            auto_break: true,
            audio_file_path: None,
            mute: false,
            color_config: ColorConfig::default(),
        }
    }
}
