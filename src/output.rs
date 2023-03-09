use derive_new::new;
use std::io::Write;

/// A constant for the CSV file headers
const CSV_HEADERS: &str = "trial, rms_error, response_time, correct_key, key";

/// A struct that helps to collect AlarmReactions and can output them to a CSV file
pub struct SessionOutput {
    /// The current trial reactions in chronological order
    pub trial_reactions: Vec<TrialReaction>,
    /// The output path to the CSV
    pub output_path: String,
}

/// Information about a user's response to an instance of an alarm being fired, a trial executing
#[derive(Debug, Clone, new)]
pub struct TrialReaction {
    pub trial_num: usize,
    /// The reaction time to the alarm in milliseconds
    pub millis: u32,
    /// If the correct key to respond to the alarm with was pressed or not
    pub correct_key: bool,
    /// The correct key that should have been pressed
    pub key: char,
    /// The root-mean-square error of the distance from the ball to the center crosshair
    pub rms_error: f32,
}

impl SessionOutput {
    /// Creates a new session output that outputs to the provided path
    pub fn new(output_path: String) -> Self {
        Self {
            trial_reactions: Vec::new(),
            output_path,
        }
    }

    /// Adds a TrialReaction to be outputted
    pub fn add_reaction(&mut self, reaction: TrialReaction) {
        self.trial_reactions.push(reaction);
    }

    /// Writes all of the currently held TrialReactions to the SessionOutput's path in CSV format
    pub fn write_to_file(&self) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.output_path)
            .unwrap();

        writeln!(file, "{CSV_HEADERS}").unwrap();
        println!("{CSV_HEADERS}");

        for reaction in &self.trial_reactions {
            writeln!(
                file,
                "{}, {}, {}, {}, {}",
                reaction.trial_num,
                reaction.rms_error,
                reaction.millis,
                reaction.correct_key,
                reaction.key
            )
            .unwrap();
            println!(
                "{}, {}, {}, {}, {}",
                reaction.trial_num,
                reaction.rms_error,
                reaction.millis,
                reaction.correct_key,
                reaction.key
            );
        }
    }
}
