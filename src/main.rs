use anyhow::Result;

use audio::Controller;
use config::Config;
use iced::time::Instant;
use iced::Element;
use iced::Length;
use iced::Subscription;

mod audio;
mod color;
mod config;
mod ring;
mod state;

use color::StateColorConfig;
use ring::RingSemiPending;
use state::{PauseKind, State, StateKind};

const DEFAULT_CONFIG_PATH: &str = "./config.toml";

struct App {
    pub state: State,
    pub config: Config,
    pub config_path: String,

    pub audio_started_once: bool,
    pub audio: Controller,
}

pub enum ButtonKind {
    Start,
    Stop,
    Pause,
}

#[derive(Debug, Clone)]
pub enum Event {
    TimerTick(Instant),
    Reload,
    Start,
    Stop,
    Pause,
    CancelAudio,
}

impl App {
    fn color_config(&self) -> &StateColorConfig {
        self.config.color_config.with_state(self.state.kind())
    }

    fn subscription(&self) -> Subscription<Event> {
        let ticks = if self.state.kind.needs_tick() {
            iced::time::every(iced::time::Duration::from_millis(300)).map(Event::TimerTick)
        } else {
            Subscription::none()
        };

        let reloader = iced::keyboard::on_key_press(|k, _mod| match k {
            iced::keyboard::Key::Character(c) if c == "z" || c == "Z" => Some(Event::Reload),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                Some(Event::CancelAudio)
            }
            _ => None,
        });

        Subscription::batch(vec![ticks, reloader])
    }

    fn update_config(&mut self, new_config: Config) {
        let Config {
            audio_file_path,
            mute,

            color_config: _,
            work_expected_duration: _,
            break_divisor: _,
            auto_break: _,
        } = std::mem::replace(&mut self.config, new_config);
        let is_audio_changed = audio_file_path != self.config.audio_file_path;
        let is_mute_changed = mute != self.config.mute;

        if is_audio_changed {
            self.audio
                .change_source(self.config.audio_file_path.clone());
        }
        if is_mute_changed {
            if self.config.mute {
                self.audio.mute();
            } else {
                self.audio.unmute();
            }
        }

        self.state.update_config(&self.config);
    }

    fn update(&mut self, ev: Event) {
        match ev {
            Event::Reload => match Config::from_file(&self.config_path) {
                Ok(new_config) => self.update_config(new_config),
                Err(err) => {
                    eprintln!("failed to read config at {}: {}", self.config_path, err)
                }
            },
            Event::TimerTick(at) => {
                self.state.on_tick_at(at);
                if !self.audio_started_once && self.state.is_completed() {
                    self.audio.start();
                    self.audio_started_once = true;
                }
            }
            Event::Start => {
                self.audio.stop();
                let was_paused = self.state.kind.is_paused();
                self.state.start();
                if !was_paused {
                    self.audio_started_once = false;
                }
            }
            Event::Stop => {
                self.audio.stop();
                let was_paused = self.state.kind.is_paused();
                self.state.stop();
                if !was_paused {
                    self.audio_started_once = false;
                }
            }
            Event::Pause => {
                self.audio.stop();
                self.state.pause();
            }
            Event::CancelAudio => {
                self.audio.stop();
            }
        }
    }

    fn make_ring(&self) -> RingSemiPending {
        RingSemiPending {
            ratio: self.state.completed_ratio(),
            stroke_width: 6.0,
            padding: 4.0,
            color_background: self.color_config().circle_background,
            color_filled: self.color_config().active_circle,
            color_pending: self.color_config().pending_circle,
        }
    }

    fn button_style(
        &self,
    ) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + '_
    {
        |_, _| iced::widget::button::Style {
            background: iced::Background::Color(
                iced::Color::from(self.color_config().button_background).scale_alpha(0.5),
            )
            .into(),
            text_color: self.color_config().button_text.into(),
            border: iced::Border::default().rounded(0.2),
            shadow: iced::Shadow {
                color: iced::Color::BLACK,
                offset: [0., 0.].into(),
                blur_radius: 3.,
            },
        }
    }

    fn view(&self) -> Element<Event> {
        use iced::widget;

        let ring = self.make_ring();

        let mut controls = widget::Row::with_capacity(ButtonKind::all().size_hint().0 * 2);
        {
            controls = controls.push(widget::vertical_space().width(Length::FillPortion(4)));

            for button in ButtonKind::all() {
                let Some(text) = button.text(self.state.kind()) else {
                    continue;
                };
                controls = controls.push(
                    widget::button(text)
                        .on_press(button.event())
                        .style(self.button_style())
                        .height(Length::FillPortion(3)),
                );
                controls = controls.push(widget::vertical_space().width(Length::FillPortion(1)));
            }

            controls = controls.push(widget::vertical_space().width(Length::FillPortion(3)));
        }
        let columns = widget::column![
            widget::horizontal_space().height(Length::FillPortion(1)),
            widget::text(self.state.name())
                .color(self.color_config().title_text)
                .height(Length::FillPortion(2))
                .size(24),
            widget::canvas(ring).height(Length::FillPortion(4)),
            controls,
            widget::horizontal_space().height(Length::FillPortion(1)),
            widget::text(self.state.time())
                .color(self.color_config().timer_text)
                .size(16)
                .height(Length::FillPortion(1)),
            widget::horizontal_space().height(Length::FillPortion(2)),
        ]
        .align_x(iced::Alignment::Center);

        widget::container(columns)
            .style(|_| {
                widget::container::background(iced::Color::from(self.color_config().background))
            })
            .center_x(Length::Fixed(400.))
            .into()
    }
}

impl ButtonKind {
    fn all() -> impl Iterator<Item = Self> {
        #[allow(dead_code)]
        fn f(t: ButtonKind) {
            match t {
                ButtonKind::Start | ButtonKind::Stop | ButtonKind::Pause => {}
            }
        }

        IntoIterator::into_iter([ButtonKind::Start, ButtonKind::Stop, ButtonKind::Pause])
    }

    fn event(&self) -> Event {
        match self {
            ButtonKind::Start => Event::Start,
            ButtonKind::Stop => Event::Stop,
            ButtonKind::Pause => Event::Pause,
        }
    }

    fn text(&self, state: &StateKind) -> Option<&'static str> {
        Some(match (self, state) {
            (ButtonKind::Start, StateKind::Work | StateKind::Break)
            | (ButtonKind::Stop, StateKind::Begin)
            | (ButtonKind::Pause, StateKind::Begin | StateKind::Pause(_)) => return None,
            (ButtonKind::Start, StateKind::Begin) => "Start Session",
            (ButtonKind::Start, StateKind::Pause(PauseKind::Work))
            | (ButtonKind::Start, StateKind::Pause(PauseKind::Break)) => "Continue",
            (ButtonKind::Stop, StateKind::Work | StateKind::Pause(PauseKind::Work)) => "Break",
            (ButtonKind::Stop, StateKind::Break | StateKind::Pause(PauseKind::Break)) => "Work",
            (ButtonKind::Pause, StateKind::Work | StateKind::Break) => "Pause",
        })
    }
}

fn init_config() -> Result<(Config, String)> {
    let mut args = std::env::args().skip(1);
    let (config, config_path) = match (args.next().as_deref(), args.next().as_deref()) {
        (None, None) => (Config::default(), None),
        (Some("-c"), Some(p)) | (Some("--config"), Some(p)) => {
            use anyhow::Context;
            (
                Config::from_file(p).with_context(|| format!("at path {p}"))?,
                Some(p.into()),
            )
        }
        _ => {
            println!("Usage: fluyendo [--config <path/to/config.toml>]");
            std::process::exit(0);
        }
    };
    let config_path = config_path.unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());

    Ok((config, config_path))
}

fn main() -> Result<()> {
    // See https://github.com/iced-rs/iced/issues/1810
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        std::env::set_var("ICED_PRESENT_MODE", "mailbox");
    }

    let (config, config_path) = init_config()?;

    let audio_path = config.audio_file_path.clone();
    let audio = audio::start_audio_thread(audio_path);
    if config.mute {
        audio.mute();
    } else {
        audio.unmute();
    }

    iced::application("fluyendo", App::update, App::view)
        .window_size((400., 600.))
        .subscription(App::subscription)
        .antialiasing(true)
        .run_with(|| {
            (
                App {
                    state: State::from_config(&config),
                    config,
                    config_path,
                    audio_started_once: false,
                    audio,
                },
                iced::Task::none(),
            )
        })
        .map_err(Into::into)
}
