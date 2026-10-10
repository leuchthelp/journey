use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crossbeam::atomic::AtomicCell;
use rodio::Float;
use rodio::mixer::Mixer;
use rodio::source::SeekError;
use rodio::{Source, queue, source::Done};
use std::sync::mpsc::{Receiver, Sender};

/// Handle to a device that outputs sounds.
///
/// Dropping the `Player` stops all its sounds. You can use `detach` if you want the sounds to continue
/// playing.
pub struct Player {
    queue_tx: Arc<queue::SourcesQueueInput>,
    controls: Arc<Controls>,
    sound_count: Arc<AtomicUsize>,
}

struct SeekOrder {
    pos: Duration,
    feedback: Sender<Result<(), SeekError>>,
}

impl SeekOrder {
    fn new(pos: Duration) -> (Self, Receiver<Result<(), SeekError>>) {
        let (tx, rx) = {
            use std::sync::mpsc;
            mpsc::channel()
        };
        (Self { pos, feedback: tx }, rx)
    }

    fn attempt<S>(self, maybe_seekable: &mut S)
    where
        S: Source,
    {
        let res = maybe_seekable.try_seek(self.pos);
        let _ignore_receiver_dropped = self.feedback.send(res);
    }
}

struct Controls {
    stopped: AtomicCell<bool>,
    pause: AtomicCell<bool>,
    volume: AtomicCell<Float>,
    speed: AtomicCell<f32>,
    to_clear: AtomicCell<u32>,
    seek: AtomicCell<Option<SeekOrder>>,
    position: AtomicCell<Duration>,
}

struct RestrictedControls(Arc<Controls>);

impl RestrictedControls {
    pub fn get_stopped(&self) -> bool {
        self.0.stopped.load()
    }
    pub fn get_pause(&self) -> bool {
        self.0.pause.load()
    }
    pub fn get_volume(&self) -> Float {
        self.0.volume.load()
    }
    pub fn get_speed(&self) -> f32 {
        self.0.speed.load()
    }
    pub fn get_seek(&mut self) -> Option<SeekOrder> {
        self.0.seek.take()
    }
}

trait RequiredForPlayer: Send + Sync {}

trait PlayerImpl: RequiredForPlayer {
    fn connect_new(mixer: &Mixer) -> Player;
    fn new() -> (Player, queue::SourcesQueueOutput);
    fn append<S>(&self, source: S)
    where
        S: Source + Send + 'static;
    fn volume(&self) -> Float;
    fn set_volume(&self, val: Float);
    fn speed(&self) -> f32;
    fn set_speed(&self, val: f32);
    fn play(&self);
    fn try_seek(&self, pos: Duration) -> Result<(), SeekError>;
    fn pause(&self);
    fn is_paused(&self) -> bool;
    fn clear(&self);
    fn skip_one(&self);
    fn stop(&self);
    fn empty(&self) -> bool;
    fn len(&self) -> usize;
    fn get_pos(&self) -> Duration;
}

impl RequiredForPlayer for Player {}

impl PlayerImpl for Player {
    fn connect_new(mixer: &Mixer) -> Player {
        let (sink, source) = Player::new();
        mixer.add(source);
        sink
    }

    fn new() -> (Player, queue::SourcesQueueOutput) {
        let (queue_tx, queue_rx) = queue::queue(true);

        let sink = Player {
            queue_tx,
            controls: Arc::new(Controls {
                stopped: AtomicCell::new(false),
                pause: AtomicCell::new(false),
                volume: AtomicCell::new(1.0),
                speed: AtomicCell::new(1.0),
                to_clear: AtomicCell::new(0),
                seek: AtomicCell::new(None),
                position: AtomicCell::new(Duration::ZERO),
            }),
            sound_count: Arc::new(AtomicUsize::new(0)),
        };
        (sink, queue_rx)
    }

    fn append<S>(&self, source: S)
    where
        S: Source + Send + 'static,
    {
        let mut controls = RestrictedControls(self.controls.clone());

        let source = source
            .speed(controls.get_speed())
            // Must be placed before pausable but after speed & delay
            .track_position()
            .pausable(true)
            .amplify(controls.get_volume())
            .skippable()
            .stoppable()
            .periodic_access(Duration::from_millis(5), move |src| {
                if controls.get_stopped() {
                    src.stop();
                }

                let amp = src.inner_mut().inner_mut();
                amp.set_factor(controls.get_volume());

                let pausable = amp.inner_mut();
                pausable.set_paused(controls.get_pause());

                let speed = pausable.inner_mut().inner_mut();
                speed.set_factor(controls.get_speed());

                if let Some(seek) = controls.get_seek() {
                    seek.attempt(amp)
                }
            });

        self.sound_count.fetch_add(1, Ordering::Relaxed);
        let source = Done::new(source, self.sound_count.clone());
        self.queue_tx.append(source)
    }

    fn volume(&self) -> Float {
        self.controls.volume.load()
    }

    fn set_volume(&self, val: Float) {
        self.controls.volume.store(val);
    }

    fn speed(&self) -> f32 {
        self.controls.speed.load()
    }

    fn set_speed(&self, val: f32) {
        self.controls.speed.store(val);
    }

    fn play(&self) {
        self.controls.pause.store(false);
    }

    fn try_seek(&self, pos: Duration) -> Result<(), SeekError> {
        let (order, feedback) = SeekOrder::new(pos);
        self.controls.seek.store(Some(order));

        if self.sound_count.load(Ordering::Acquire) == 0 {
            // No sound is playing, seek will not be performed
            return Ok(());
        }

        match feedback.recv() {
            Ok(seek_res) => {
                self.controls.position.store(pos);
                seek_res
            }
            // The feedback channel closed. Probably another SeekOrder was set
            // invalidating this one and closing the feedback channel
            // ... or the audio thread panicked.
            Err(_) => Ok(()),
        }
    }

    fn pause(&self) {
        self.controls.pause.store(true);
    }

    fn is_paused(&self) -> bool {
        self.controls.pause.load()
    }

    fn clear(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        self.controls.to_clear.store(len);
        self.pause();
    }

    fn skip_one(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        let mut to_clear = self.controls.to_clear.load();
        if len > to_clear {
            to_clear += 1;
        }
    }

    fn stop(&self) {
        self.controls.stopped.store(true);
    }

    fn empty(&self) -> bool {
        self.len() == 0
    }

    fn len(&self) -> usize {
        self.sound_count.load(Ordering::Relaxed)
    }

    fn get_pos(&self) -> Duration {
        self.controls.position.load()
    }
}

impl Drop for Player {
    #[inline]
    fn drop(&mut self) {
        self.queue_tx.set_keep_alive_if_empty(false)
    }
}
