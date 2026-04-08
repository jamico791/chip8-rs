use rodio::source::SquareWave;
use rodio::{MixerDeviceSink, Player};

pub struct Audio {
    #[allow(dead_code)]
    handle: MixerDeviceSink,
    player: Player,
}

impl Audio {
    pub fn new(freq: f32) -> Self {
        let mut handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");

        // remove logging when handle is dropped
        handle.log_on_drop(false);

        // make handle immutable again
        let handle = handle;
        let player = Player::connect_new(handle.mixer());
        let source = SquareWave::new(freq);
        player.append(source);
        player.set_volume(0.01);

        Self { handle, player }
    }

    pub fn on(&mut self) {
        self.player.play();
    }

    pub fn off(&mut self) {
        self.player.pause();
    }
}
