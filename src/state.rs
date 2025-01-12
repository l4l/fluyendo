use std::time::Duration;

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

    pub elapsed: Duration,
    pub work_bound_duration: Duration,
    pub break_divisor: f32,
    pub saved_break_time: Duration,
}

impl State {
    pub fn kind(&self) -> &StateKind {
        &self.kind
    }

    pub fn add_elapsed(&mut self, delta: Duration) {
        self.elapsed += delta;
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
        self.kind = match self.kind {
            StateKind::Begin | StateKind::Pause(PauseKind::Work) => StateKind::Work,
            StateKind::Pause(PauseKind::Break) => StateKind::Break,
            StateKind::Work | StateKind::Break => return,
        }
    }

    pub fn stop(&mut self) {
        self.kind = match self.kind {
            StateKind::Begin => return,
            StateKind::Pause(PauseKind::Work) | StateKind::Work => {
                self.saved_break_time += Duration::from_secs_f32(
                    std::mem::take(&mut self.elapsed).as_secs_f32() / self.break_divisor,
                );
                StateKind::Pause(PauseKind::Break)
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

impl Default for State {
    fn default() -> Self {
        Self {
            kind: StateKind::default(),
            elapsed: Duration::default(),
            work_bound_duration: Duration::from_secs(25),
            break_divisor: 5.0,
            saved_break_time: Duration::default(),
        }
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
