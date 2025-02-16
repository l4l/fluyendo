use iced::time::Duration;

use crate::config::Config;
use crate::Instant;

pub enum PauseKind {
    Work,
    Break,
}

#[derive(Default)]
pub enum StateKind {
    #[default]
    Begin,
    Pause(PauseKind),
    Work,
    Break,
}

pub struct State {
    pub kind: StateKind,

    // We calculate elapsed manually because the timer might be skewed.
    // Apparently it's better write more adequate Subscription instead.
    pub prev_tick: Option<Instant>,
    pub elapsed: Duration,

    pub saved_break_time: Duration,

    pub work_bound_duration: Duration,
    pub break_divisor: f32,
    pub auto_break: bool,
}

impl StateKind {
    pub fn needs_tick(&self) -> bool {
        !matches!(self, Self::Begin | Self::Pause(_))
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Pause(_))
    }
}

impl State {
    pub fn from_config(config: &Config) -> Self {
        let mut this = Self {
            kind: StateKind::default(),
            prev_tick: None,
            elapsed: Duration::from_secs(0),
            saved_break_time: Duration::default(),
            work_bound_duration: Duration::default(),
            break_divisor: 5.0,
            auto_break: false,
        };

        this.update_config(config);
        this
    }

    pub fn update_config(&mut self, config: &Config) {
        let Config {
            work_expected_duration,
            break_divisor,
            auto_break,
            ..
        } = config;

        self.work_bound_duration = *work_expected_duration;
        self.break_divisor = *break_divisor;
        self.auto_break = *auto_break;
    }

    pub fn kind(&self) -> &StateKind {
        &self.kind
    }

    pub fn on_tick_at(&mut self, tick_at: Instant) {
        let Some(prev_tick) = self.prev_tick else {
            return;
        };
        self.elapsed += tick_at.duration_since(prev_tick);
        self.prev_tick = Some(tick_at);
    }

    pub fn is_completed(&self) -> bool {
        self.completed_ratio() == 1.0
    }

    pub fn completed_ratio(&self) -> f32 {
        match self.kind {
            StateKind::Begin => 0.0,
            StateKind::Pause(PauseKind::Work) | StateKind::Work => {
                self.elapsed.as_secs_f32() / self.work_bound_duration.as_secs_f32()
            }
            StateKind::Pause(PauseKind::Break) | StateKind::Break => {
                self.elapsed.as_secs_f32() / self.saved_break_time.as_secs_f32()
            }
        }
        .min(1.0)
    }

    pub fn start(&mut self) {
        self.prev_tick = Some(Instant::now());
        self.kind = match self.kind {
            StateKind::Begin | StateKind::Pause(PauseKind::Work) => StateKind::Work,
            StateKind::Pause(PauseKind::Break) => StateKind::Break,
            StateKind::Work | StateKind::Break => return,
        }
    }

    pub fn stop(&mut self) {
        self.prev_tick = None;
        self.kind = match self.kind {
            StateKind::Begin => return,
            StateKind::Pause(PauseKind::Work) | StateKind::Work => {
                let elapsed = std::mem::take(&mut self.elapsed);
                self.saved_break_time +=
                    Duration::from_secs_f32(elapsed.as_secs_f32() / self.break_divisor);
                if self.auto_break {
                    self.prev_tick = Some(Instant::now());
                    StateKind::Break
                } else {
                    StateKind::Pause(PauseKind::Break)
                }
            }
            StateKind::Pause(PauseKind::Break) | StateKind::Break => {
                self.saved_break_time = self
                    .saved_break_time
                    .checked_sub(std::mem::take(&mut self.elapsed))
                    .unwrap_or_default();
                StateKind::Pause(PauseKind::Work)
            }
        };
    }

    pub fn pause(&mut self) {
        self.prev_tick = None;
        self.kind = match self.kind {
            StateKind::Begin
            | StateKind::Pause(PauseKind::Work)
            | StateKind::Pause(PauseKind::Break) => return,
            StateKind::Work => StateKind::Pause(PauseKind::Work),
            StateKind::Break => StateKind::Pause(PauseKind::Break),
        };
    }

    pub fn name(&self) -> String {
        match self.kind {
            StateKind::Begin => "Ready to start",
            StateKind::Pause(PauseKind::Work) => "Pause (Work)",
            StateKind::Pause(PauseKind::Break) => "Pause (Break)",
            StateKind::Work => "Working",
            StateKind::Break => "Breaking",
        }
        .into()
    }

    pub fn time(&self) -> String {
        let (elapsed, limit) = match self.kind {
            StateKind::Begin => (Duration::default(), Duration::default()),
            StateKind::Pause(PauseKind::Work) | StateKind::Work => {
                (self.elapsed, self.work_bound_duration)
            }
            StateKind::Pause(PauseKind::Break) | StateKind::Break => {
                (self.elapsed, self.saved_break_time)
            }
        };

        let elapsed = duration_to_str(elapsed);
        let limit = duration_to_str(limit);

        format!("{elapsed}    / {limit}")
    }
}

pub fn duration_to_str(d: Duration) -> String {
    let secs = d.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;

    let hours = mins / 60;
    let mins = mins % 60;

    format!("{hours:4}:{mins:02}:{secs:02}")
}
