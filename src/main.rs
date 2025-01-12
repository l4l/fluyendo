use std::time::Duration;

use audio::Controller;
use iced::Element;
use iced::Length;
use iced::Subscription;

mod audio;
mod colors;
mod ring;
mod state;

use colors::{Color, ColorConfig, StateColorConfig};
use ring::RingSemiPending;
use state::{PauseKind, State, StateKind};

struct App {
    pub state: State,
    pub color_config: ColorConfig,

    pub audio: Controller,
}

pub enum ButtonKind {
    Start,
    Stop,
    Pause,
}

#[derive(Debug, Clone)]
pub enum Event {
    TimerTick(Duration),
    Reload,
    Start,
    Stop,
    Pause,
    CancelAudio,
}

impl App {
    fn color_config(&self) -> &StateColorConfig {
        self.color_config.with_state(self.state.kind())
    }

    fn subscription(&self) -> Subscription<Event> {
        let ticks = if matches!(self.state.kind(), StateKind::Begin | StateKind::Pause(_))
            || self.state.is_completed()
        {
            Subscription::none()
        } else {
            let duration = Duration::from_millis(100);
            iced::time::every(duration).map(move |_| Event::TimerTick(Duration::from_millis(100)))
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

    fn update(&mut self, ev: Event) {
        match ev {
            Event::Reload => match std::fs::read_to_string("./color_config.toml") {
                Ok(content) => match toml::from_str::<ColorConfig>(&content) {
                    Ok(config) => {
                        self.color_config = config;
                        println!("reloaded");
                    }
                    Err(err) => {
                        eprintln!("parse failed: {:?}", err);
                    }
                },
                Err(err) => {
                    eprintln!("i/o failed: {:?}", err);
                }
            },
            Event::TimerTick(at) => {
                self.state
                    .add_elapsed(Duration::from_secs_f32(at.as_secs_f32()));
                if self.state.is_completed() {
                    self.audio.start();
                }
            }
            Event::Start => {
                self.audio.stop();
                self.state.start()
            }
            Event::Stop => {
                self.audio.stop();
                self.state.stop()
            }
            Event::Pause => {
                self.audio.stop();
                self.state.pause()
            }
            Event::CancelAudio => self.audio.stop(),
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
            | (ButtonKind::Stop, StateKind::Begin | StateKind::Pause(PauseKind::Work))
            | (ButtonKind::Pause, StateKind::Begin | StateKind::Pause(_)) => return None,
            (ButtonKind::Start, StateKind::Begin) => "Start Session",
            (ButtonKind::Start, StateKind::Pause(PauseKind::Work))
            | (ButtonKind::Start, StateKind::Pause(PauseKind::Break)) => "Continue",
            (ButtonKind::Stop, StateKind::Pause(PauseKind::Break)) => "Skip",
            (ButtonKind::Stop, StateKind::Work) => "Break",
            (ButtonKind::Stop, StateKind::Break) => "Work",
            (ButtonKind::Pause, StateKind::Work | StateKind::Break) => "Pause",
        })
    }
}

fn main() -> iced::Result {
    iced::application("fluyendo", App::update, App::view)
        .window_size((400., 600.))
        .subscription(App::subscription)
        .antialiasing(true)
        .run_with(|| {
            (
                App {
                    state: Default::default(),
                    color_config: Default::default(),
                    audio: audio::start_audio_thread(),
                },
                iced::Task::none(),
            )
        })
}
