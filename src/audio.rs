use anyhow::Result;
use rodio::{source::Source, Decoder, OutputStream, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

/// A command that can be sent to the audio thread
#[derive(Debug)]
enum AudioCommand {
    /// A command to begin playing an audio sample
    Play(String, SoundSample),
    /// A command to stop playing an audio sample
    Stop(String),
}

pub struct AudioManager {
    samples: Mutex<HashMap<String, SoundSample>>,
    _thread: std::thread::JoinHandle<()>,
    tx: Mutex<mpsc::Sender<AudioCommand>>,
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager").finish()
    }
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        //source.convert_samples()
        Ok(Self {
            samples: Mutex::new(HashMap::new()),
            _thread: std::thread::spawn(move || Self::audio_thread(rx)),
            tx: Mutex::new(tx),
        })
    }

    fn audio_thread(rx: mpsc::Receiver<AudioCommand>) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let mut sink_map = HashMap::new();

        loop {
            match rx.recv() {
                Ok(AudioCommand::Play(name, sample)) => {
                    let sink = match Sink::try_new(&stream_handle) {
                        Ok(sink) => sink,
                        Err(e) => {
                            log::error!("error starting new audio sink {}", e);
                            continue;
                        }
                    };

                    log::info!("got sample, with dial name {}", name);
                    // Starts playing the sample
                    sink.append(sample);
                    sink_map.insert(name, sink);
                    log::info!("returned from play_raw");
                }
                Ok(AudioCommand::Stop(name)) => {
                    log::info!("Stopping alarm with dial name: {}", name);
                    // Drops the Sink
                    sink_map.remove(&name);
                }
                _ => {}
            }
        }
    }

    /// Loads a file into the samples cache if it hasn't been loaded already.
    /// Returns the sample from the cache, or the new one loaded
    pub fn preload_file(&self, path: &str) -> Result<SoundSample> {
        let mut guard = self.samples.lock().unwrap();
        if let Some(sample) = guard.get(path) {
            Ok(sample.clone())
        } else {
            log::info!("loading sound file {}", path);
            // Load a sound from a file, using a path relative to Cargo.toml
            let file = BufReader::new(File::open(path)?);
            // Decode that sound file into a source
            let source = Decoder::new(file)?;
            let samples = source.convert_samples();
            let buf = SoundSample::new(samples);
            guard.insert(String::from(path), buf.clone());
            Ok(buf)
        }
    }

    /// Does its best to play the given alarm sound
    pub fn play(&self, name: &str, path: &str) -> Result<()> {
        log::info!("about to preload file");
        let sample = self.preload_file(path)?;
        log::info!("got sample");
        let guard = self.tx.lock().unwrap();
        log::info!("sending sample to other thread");
        let _ = guard.send(AudioCommand::Play(name.to_string(), sample));
        Ok(())
    }

    /// Cancels playing of an alarm sound by its unique alarm name
    pub fn stop(&self, name: &str) {
        let guard = self.tx.lock().unwrap();
        let _ = guard.send(AudioCommand::Stop(name.to_string()));
    }
}

/// Custom [`Source`] that holds a shallow copy of its data to allow for easy cloning since
/// playing a sample consumes self
#[derive(Clone, Debug)]
pub struct SoundSample {
    channels: u16,
    sample_rate: u32,
    total_duration: Option<Duration>,
    samples: Arc<Vec<f32>>,
    offset: usize,
}

impl SoundSample {
    pub fn new<S>(source: S) -> Self
    where
        S: Source<Item = f32> + Send + 'static,
    {
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        let total_duration = source.total_duration();
        let samples = Arc::new(source.into_iter().collect());

        Self {
            channels,
            sample_rate,
            total_duration,
            samples,
            offset: 0,
        }
    }
}

impl Source for SoundSample {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}

impl Iterator for SoundSample {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.get(self.offset).map(|v| {
            self.offset += 1;
            *v
        })
    }
}
