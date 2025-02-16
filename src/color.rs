use super::{PauseKind, StateKind};

mod defaults;

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(default)]
pub struct StateColorConfig {
    pub title_text: Color,
    pub timer_text: Color,

    pub button_text: Color,
    pub button_background: Color,

    pub background: Color,

    pub active_circle: Color,
    pub pending_circle: Color,
    pub circle_background: Color,
}

#[derive(serde::Deserialize)]
#[serde(default)]
pub struct ColorConfig {
    work: StateColorConfig,
    r#break: StateColorConfig,
    start: StateColorConfig,

    pause_work: StateColorConfig,
    pause_break: StateColorConfig,
}

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorConfig {
    pub fn with_state(&self, state: &StateKind) -> &StateColorConfig {
        match state {
            StateKind::Begin => &self.start,
            StateKind::Pause(PauseKind::Work) => &self.pause_work,
            StateKind::Pause(PauseKind::Break) => &self.pause_break,
            StateKind::Work => &self.work,
            StateKind::Break => &self.r#break,
        }
    }
}

impl Color {
    pub const fn rgb(rgb: u32) -> Self {
        assert!(rgb < 0x1_00_00_00);
        let [_, r, g, b] = rgb.to_be_bytes();

        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }
}

impl TryFrom<String> for Color {
    type Error = csscolorparser::ParseColorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        csscolorparser::parse(&value).map(|c| Color {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        })
    }
}

impl From<Color> for iced::Color {
    fn from(value: Color) -> Self {
        let Color { r, g, b, a } = value;
        iced::Color::from_rgba(r, g, b, a)
    }
}
