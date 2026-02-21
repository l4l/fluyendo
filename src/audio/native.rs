use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

type Result<T = ()> = std::result::Result<T, raplay::Error>;

const DEFAULT_ALARM: &[u8] = include_bytes!("../../res/lofi-alarm-clock.mp3");

pub type Param = Option<PathBuf>;

pub struct Controller(std::sync::mpsc::SyncSender<Command>);

enum Command {
    Start,
    Stop,
    ChangeSource(Option<PathBuf>),
    ChangeVolume(f32),
}

impl Controller {
    pub fn new(path: Param) -> Self {
        start_audio_thread(path)
    }

    fn send(&self, cmd: Command) {
        if let Err(err) = self.0.try_send(cmd) {
            eprintln!("failed to send cmd to audio thread: {err:?}");
        }
    }

    pub fn start(&mut self) {
        self.send(Command::Start);
    }

    pub fn stop(&mut self) {
        self.send(Command::Stop);
    }

    pub fn update(&self, path: Param) {
        self.send(Command::ChangeSource(path));
    }

    pub fn mute(&self) {
        self.send(Command::ChangeVolume(0.0));
    }

    pub fn unmute(&self) {
        self.send(Command::ChangeVolume(1.0));
    }
}

#[derive(Default)]
struct Player {
    sink: raplay::Sink,
}

impl Player {
    fn load(&mut self, p: &Path) -> Result<()> {
        let buf = std::fs::read(p).map_err(|err| raplay::Error::Other(err.into()))?;
        self.read_from_buf(Cow::Owned(buf))
    }

    fn load_default(&mut self) -> Result<()> {
        self.read_from_buf(Cow::Borrowed(DEFAULT_ALARM))
    }

    fn read_from_buf(&mut self, buf: Cow<'static, [u8]>) -> Result<()> {
        let src = raplay::source::Symph::try_new(std::io::Cursor::new(buf), &Default::default())?;
        self.sink.load(src, false)
    }
}

fn start_audio_thread(audio_path: Option<PathBuf>) -> Controller {
    use std::time::Duration;
    let (sender, rx) = std::sync::mpsc::sync_channel::<Command>(5);

    std::thread::spawn(move || {
        let handle_res = |res| {
            if let Err(err) = res {
                eprintln!("audio error: {err:?}");
            }
        };

        let mut player = Player::default();
        let res = if let Some(p) = audio_path {
            player.load(&p)
        } else {
            player.load_default()
        };
        handle_res(res);

        while let Ok(c) = rx.recv() {
            let res = match c {
                Command::Start => player.sink.play(true),
                Command::Stop if !player.sink.is_playing().map_or(true, |x| x) => continue,
                Command::Stop => player
                    .sink
                    .pause()
                    .and_then(|()| player.sink.seek_to(Duration::from_secs(0)))
                    .map(|_| ()),
                Command::ChangeSource(None) => player.load_default(),
                Command::ChangeSource(Some(p)) => player.load(&p),
                Command::ChangeVolume(v) => player.sink.volume(v),
            };

            handle_res(res);
        }
    });

    Controller(sender)
}
