use std::path::PathBuf;

pub type Param = Option<PathBuf>;

pub struct Controller;

impl Controller {
    pub fn new(_: Param) -> Self {
        Self
    }

    pub fn start(&mut self) {}
    pub fn stop(&mut self) {}
    pub fn update(&self, _: Param) {}
    pub fn mute(&self) {}
    pub fn unmute(&self) {}
}
