use super::*;

const DEFAULT_BACKGROUND: Color = Color::rgb(0x0a2b46);

const DEFAULT_TEXT: Color = Color::rgb(0x9db5d8);

const DEFAULT_BUTTON_BACKGROUND: Color = Color::rgb(0x1b86a5);

const DEFAULT_CIRCLE_BACKGROUND: Color = Color::rgb(0x008a5e);
const DEFAULT_ACTIVE_CIRCLE: Color = Color::rgb(0x35c191);
const DEFAULT_PENDING_CIRCLE: Color = Color::rgb(0x77fac7);

impl Default for StateColorConfig {
    fn default() -> Self {
        Self {
            title_text: DEFAULT_TEXT,
            timer_text: DEFAULT_TEXT,
            button_text: DEFAULT_TEXT,
            button_background: DEFAULT_BUTTON_BACKGROUND,
            background: DEFAULT_BACKGROUND,
            active_circle: DEFAULT_ACTIVE_CIRCLE,
            pending_circle: DEFAULT_PENDING_CIRCLE,
            circle_background: DEFAULT_CIRCLE_BACKGROUND,
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        let default_per_state = StateColorConfig::default();

        let work = StateColorConfig {
            title_text: Color::rgb(0x00c6b5),
            ..default_per_state
        };

        let r#break = StateColorConfig {
            title_text: Color::rgb(0xfff7d6),
            ..default_per_state
        };

        let start = StateColorConfig {
            title_text: Color::rgb(0x90aecf),
            circle_background: Color::rgb(0x3c4b5b),
            active_circle: Color::rgb(0x92a1b3),
            pending_circle: Color::rgb(0x667585),
            ..default_per_state
        };
        let pause_work = StateColorConfig {
            title_text: Color::rgb(0x21857c),
            ..start
        };
        let pause_break = StateColorConfig {
            title_text: Color::rgb(0xc48400),
            ..start
        };

        Self {
            work,
            r#break,
            start,
            pause_work,
            pause_break,
        }
    }
}
