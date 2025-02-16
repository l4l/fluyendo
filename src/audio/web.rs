use serde::Deserialize;
use web_sys::HtmlAudioElement;

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct Param;

pub struct Controller(HtmlAudioElement);

impl Controller {
    pub fn new(_: Param) -> Self {
        Self(HtmlAudioElement::new_with_src("lofi-alarm-clock.mp3").unwrap())
    }

    pub fn start(&mut self) {
        let _ = self.0.play().unwrap();
    }

    pub fn stop(&mut self) {
        self.0.pause().unwrap();
        self.0.fast_seek(0.).unwrap();
    }

    pub fn update(&self, _: Param) {}

    pub fn mute(&self) {
        self.0.set_volume(0.0);
    }

    pub fn unmute(&self) {
        self.0.set_volume(1.0);
    }
}
