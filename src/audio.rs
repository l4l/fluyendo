use std::time::Duration;

const DEFAULT_ALARM: &[u8] = include_bytes!("../res/lofi-alarm-clock.mp3");

pub struct Controller(std::sync::mpsc::SyncSender<Command>);

enum Command {
    Start,
    Stop,
}

impl Controller {
    pub fn start(&self) {
        if let Err(err) = self.0.try_send(Command::Start) {
            eprintln!("failed to send cmd to audio thread: {:?}", err);
        }
    }

    pub fn stop(&self) {
        if let Err(err) = self.0.try_send(Command::Stop) {
            eprintln!("failed to send cmd to audio thread: {:?}", err);
        }
    }
}

pub fn start_audio_thread() -> Controller {
    let (tx, rx) = std::sync::mpsc::sync_channel::<Command>(5);
    std::thread::spawn(move || {
        let mut sink = raplay::Sink::default();
        let src = raplay::source::Symph::try_new(
            std::io::Cursor::new(DEFAULT_ALARM),
            &Default::default(),
        )
        .unwrap();
        sink.load(src, false).unwrap();

        while let Ok(c) = rx.recv() {
            let res = match c {
                Command::Start => sink.play(true),
                Command::Stop if !sink.is_playing().map_or(true, |x| x) => continue,
                Command::Stop => sink
                    .pause()
                    .and_then(|()| sink.seek_to(Duration::from_secs(0)))
                    .map(|_| ()),
            };

            if let Err(err) = res {
                eprintln!("audio error: {:?}", err);
            }
        }
    });

    Controller(tx)
}
