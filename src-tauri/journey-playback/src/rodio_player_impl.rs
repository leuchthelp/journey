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
    /// Builds a new `Player`, beginning playback on a stream.
    #[inline]
    fn connect_new(mixer: &Mixer) -> Player {
        let (sink, source) = Player::new();
        mixer.add(source);
        sink
    }

    /// Builds a new `Player`.
    #[inline]
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

    /// Appends a sound to the queue of sounds to play.
    #[inline]
    fn append<S>(&self, source: S)
    where
        S: Source + Send + 'static,
    {
        // Wait for the queue to flush then resume stopped playback
        if self.controls.stopped.load() {
            self.controls.stopped.store(false);
        }

        let controls = self.controls.clone();

        let start_played = AtomicCell::new(false);

        let source = source
            .speed(1.0)
            // Must be placed before pausable but after speed & delay
            .track_position()
            .pausable(false)
            .amplify(1.0)
            .skippable()
            .stoppable()
            // If you change the duration update the docs for try_seek!
            .periodic_access(Duration::from_millis(5), move |src| {
                if controls.stopped.load() {
                    src.stop();
                    controls.position.store(Duration::ZERO);
                }

                let amp = src.inner_mut().inner_mut();
                amp.set_factor(controls.volume.load());
                amp.inner_mut().set_paused(controls.pause.load());
                amp.inner_mut()
                    .inner_mut()
                    .inner_mut()
                    .set_factor(controls.speed.load());

                if let Some(seek) = controls.seek.take() {
                    seek.attempt(amp)
                }
                start_played.store(true);
            });
        self.sound_count.fetch_add(1, Ordering::Relaxed);
        let source = Done::new(source, self.sound_count.clone());
        self.queue_tx.append(source)
    }

    /// Gets the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than 1.0 will
    /// multiply each sample by this value.
    #[inline]
    fn volume(&self) -> Float {
        self.controls.volume.load()
    }

    /// Changes the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than `1.0` will
    /// multiply each sample by this value.
    #[inline]
    fn set_volume(&self, val: Float) {
        self.controls.volume.store(val);
    }

    /// Gets the speed of the sound.
    ///
    /// See [`Player::set_speed`] for details on what *speed* means.
    #[inline]
    fn speed(&self) -> f32 {
        self.controls.speed.load()
    }

    /// Changes the play speed of the sound. Does not adjust the samples, only the playback speed.
    ///
    /// # Note:
    /// 1. **Increasing the speed will increase the pitch by the same factor**
    /// - If you set the speed to 0.5 this will halve the frequency of the sound
    ///   lowering its pitch.
    /// - If you set the speed to 2 the frequency will double raising the
    ///   pitch of the sound.
    /// 2. **Change in the speed affect the total duration inversely**
    /// - If you set the speed to 0.5, the total duration will be twice as long.
    /// - If you set the speed to 2 the total duration will be halve of what it
    ///   was.
    ///
    #[inline]
    fn set_speed(&self, val: f32) {
        self.controls.speed.store(val);
    }

    /// Resumes playback of a paused player.
    ///
    /// No effect if not paused.
    #[inline]
    fn play(&self) {
        self.controls.pause.store(false);
    }

    // There is no `can_seek()` method as it is impossible to use correctly. Between
    // checking if a source supports seeking and actually seeking the sink can
    // switch to a new source.

    /// Attempts to seek to a given position in the current source.
    ///
    /// This blocks between 0 and ~5 milliseconds.
    ///
    /// As long as the duration of the source is known, seek is guaranteed to saturate
    /// at the end of the source. For example given a source that reports a total duration
    /// of 42 seconds calling `try_seek()` with 60 seconds as argument will seek to
    /// 42 seconds.
    ///
    /// # Errors
    /// This function will return [`SeekError::NotSupported`] if one of the underlying
    /// sources does not support seeking.
    ///
    /// It will return an error if an implementation ran
    /// into one during the seek.
    ///
    /// When seeking beyond the end of a source this
    /// function might return an error if the duration of the source is not known.
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

    /// Pauses playback of this player.
    ///
    /// No effect if already paused.
    ///
    /// A paused sink can be resumed with `play()`.
    fn pause(&self) {
        self.controls.pause.store(true);
    }

    /// Gets if a sink is paused
    ///
    /// Players can be paused and resumed using `pause()` and `play()`. This returns `true` if the
    /// sink is paused.
    fn is_paused(&self) -> bool {
        self.controls.pause.load()
    }

    /// Removes all currently loaded `Source`s from the `Player`, and pauses it.
    ///
    /// See `pause()` for information about pausing a `Player`.
    fn clear(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        self.controls.to_clear.store(len);
        self.pause();
    }

    /// Skips to the next `Source` in the `Player`
    ///
    /// If there are more `Source`s appended to the `Player` at the time,
    /// it will play the next one. Otherwise, the `Player` will finish as if
    /// it had finished playing a `Source` all the way through.
    fn skip_one(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        let mut to_clear = self.controls.to_clear.load();
        if len > to_clear {
            to_clear += 1;
        }
    }

    /// Stops the sink by emptying the queue.
    #[inline]
    fn stop(&self) {
        self.controls.stopped.store(true);
    }

    /// Returns true if this sink has no more sounds to play.
    #[inline]
    fn empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of sounds currently in the queue.
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    fn len(&self) -> usize {
        self.sound_count.load(Ordering::Relaxed)
    }

    /// Returns the position of the sound that's being played.
    ///
    /// This takes into account any speedup or delay applied.
    ///
    /// Example: if you apply a speedup of *2* to an mp3 decoder source and
    /// [`get_pos()`](Player::get_pos) returns *5s* then the position in the mp3
    /// recording is *10s* from its start.
    #[inline]
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

#[cfg(test)]
mod tests {
    use rodio::Source;
    use rodio::buffer::SamplesBuffer;
    use rodio::math::nz;

    use crate::rodio_player_impl::{Player, PlayerImpl};

    #[test]
    fn test_pause_and_stop() {
        let (player, mut source) = Player::new();

        assert_eq!(source.next(), Some(0.0));
        // TODO (review) How did this test passed before? I might have broken something but
        //      silence source should come first as next source is only polled while previous ends.
        //      Respective test in Queue seem to be ignored (see queue::test::no_delay_when_added()
        //      at src/queue.rs:293).
        let mut source = source.skip_while(|x| *x == 0.0);

        let v = vec![10.0, -10.0, 20.0, -20.0, 30.0, -30.0];

        // Low rate to ensure immediate control.
        player.append(SamplesBuffer::new(nz!(1), nz!(1), v.clone()));
        let mut reference_src = SamplesBuffer::new(nz!(1), nz!(1), v);

        assert_eq!(source.next(), reference_src.next());
        assert_eq!(source.next(), reference_src.next());

        player.pause();

        assert_eq!(source.next(), Some(0.0));

        player.play();

        assert_eq!(source.next(), reference_src.next());
        assert_eq!(source.next(), reference_src.next());

        player.stop();

        assert_eq!(source.next(), Some(0.0));

        assert!(player.empty());
    }

    #[test]
    fn test_stop_and_start() {
        let (player, mut queue_rx) = Player::new();

        let v = vec![10.0, -10.0, 20.0, -20.0, 30.0, -30.0];

        player.append(SamplesBuffer::new(nz!(1), nz!(1), v.clone()));
        let mut src = SamplesBuffer::new(nz!(1), nz!(1), v.clone());

        assert_eq!(queue_rx.next(), src.next());
        assert_eq!(queue_rx.next(), src.next());

        player.stop();

        assert!(player.controls.stopped.load());
        assert_eq!(queue_rx.next(), Some(0.0));

        src = SamplesBuffer::new(nz!(1), nz!(1), v.clone());
        player.append(SamplesBuffer::new(nz!(1), nz!(1), v));

        assert!(!player.controls.stopped.load());
        // Flush silence
        let mut queue_rx = queue_rx.skip_while(|v| *v == 0.0);

        assert_eq!(queue_rx.next(), src.next());
        assert_eq!(queue_rx.next(), src.next());
    }

    #[test]
    fn test_volume() {
        let (player, mut queue_rx) = Player::new();

        let v = vec![10.0, -10.0, 20.0, -20.0, 30.0, -30.0];

        // High rate to avoid immediate control.
        player.append(SamplesBuffer::new(nz!(2), nz!(44100), v.clone()));
        let src = SamplesBuffer::new(nz!(2), nz!(44100), v.clone());

        let mut src = src.amplify(0.5);
        player.set_volume(0.5);

        for _ in 0..v.len() {
            assert_eq!(queue_rx.next(), src.next());
        }
    }
}
