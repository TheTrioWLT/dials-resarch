use anyhow::Result;
use log::{debug, error, info};
use rodio::OutputStreamHandle;
use rodio::{buffer::SamplesBuffer, source::Source, Decoder, OutputStream};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AudioManager {
    samples: Mutex<HashMap<String, BadBuffer>>,
    stream_handle: Arc<OutputStreamHandle>,
    pool: rayon::ThreadPool,
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
            .field("pool", &self.pool)
            .finish()
    }
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();

        //source.convert_samples()
        Ok(Self {
            samples: Mutex::new(HashMap::new()),
            stream_handle: Arc::new(stream_handle),
            pool,
        })
    }

    /// Loads a file into the samples cache if it hasn't been loaded already.
    /// Returns the sample from the cache, or the new one loaded
    pub fn preload_file(&self, path: &str) -> Result<BadBuffer> {
        let mut guard = self.samples.lock().unwrap();
        match guard.get(path) {
            Some(sample) => Ok(sample.clone()),
            None => {
                info!("loading sound file {}", path);
                // Load a sound from a file, using a path relative to Cargo.toml
                let file = BufReader::new(File::open(path)?);
                // Decode that sound file into a source
                let source = Decoder::new(file)?;
                let samples = source.convert_samples();
                let buf = BadBuffer::new(samples);
                guard.insert(String::from(path), buf.clone());
                Ok(buf)
            }
        }
    }

    /// Does its best to play the given alarm sound
    pub fn play(&self, path: &str) -> Result<()> {
        let sample = self.preload_file(path)?;
        let handle = Arc::clone(&self.stream_handle);
        let path = path.to_owned();

        info!("called play");
        self.pool.spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            info!("play start");
            if let Err(e) = stream_handle.play_raw(sample) {
                error!("failed to play audio file `{}`: {}", path, e);
            }
            info!("play end");
            std::thread::sleep(std::time::Duration::from_secs_f64(6.3));
        });
        Ok(())

        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        //std::thread::sleep(std::time::Duration::from_secs_f64(6.3));
    }
}

#[derive(Clone, Debug)]
pub struct BadBuffer {
    current_frame_len: Option<usize>,
    channels: u16,
    sample_rate: u32,
    total_duration: Option<Duration>,
    samples: Arc<Vec<f32>>,
    offset: usize,
}

impl BadBuffer {
    pub fn new<S>(source: S) -> Self
    where
        S: Source<Item = f32> + Send + 'static,
    {
        let current_frame_len = source.current_frame_len();
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        let total_duration = source.total_duration();
        let samples = Arc::new(source.into_iter().collect());

        Self {
            current_frame_len,
            channels,
            sample_rate,
            total_duration,
            samples,
            offset: 0,
        }
    }
}

impl Source for BadBuffer {
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

impl Iterator for BadBuffer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == 0 {
            debug!("iterating at 0");
        }
        self.samples
            .get(self.offset)
            .map(|v| {
                self.offset += 1;
                *v
            })
            .ok_or_else(|| debug!("finished iterating {}", self.offset))
            .ok()
    }
}
