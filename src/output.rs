use derive_new::new;
use std::io::Write;

/// A constant for the CSV file headers
const CSV_HEADERS: &str = "trial, response_time, correct_key, key";

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
    pub rms_error: Vec<f32>,
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

        write!(file, "{CSV_HEADERS}").unwrap();
        print!("{CSV_HEADERS}");

        for t in 0..self.trial_reactions.len() {
            write!(file, ", trial {} rmse", t + 1).unwrap();
            print!(", trial {} rmse", t + 1);
        }

        writeln!(file).unwrap();
        println!();

        let mut rms_errors: Vec<_> = self
            .trial_reactions
            .iter()
            .map(|r| r.rms_error.iter())
            .collect();

        for reaction in &self.trial_reactions {
            write!(
                file,
                "{}, {}, {}, {}",
                reaction.trial_num, reaction.millis, reaction.correct_key, reaction.key
            )
            .unwrap();

            print!(
                "{}, {}, {}, {}",
                reaction.trial_num, reaction.millis, reaction.correct_key, reaction.key
            );

            for rmse_entry in rms_errors.iter_mut() {
                if let Some(entry) = rmse_entry.next() {
                    write!(file, ", {}", entry).unwrap();
                    print!(", {}", entry);
                } else {
                    write!(file, ",").unwrap();
                    print!(",");
                }
            }

            writeln!(file).unwrap();
            println!();
        }

        let mut is_done = false;

        while !is_done {
            is_done = true;

            write!(file, ",,,").unwrap();
            print!(",,,");

            for rmse_entry in rms_errors.iter_mut() {
                if let Some(entry) = rmse_entry.next() {
                    write!(file, ", {}", entry).unwrap();
                    print!(", {}", entry);
                    is_done = false;
                } else {
                    write!(file, ",").unwrap();
                    print!(",");
                }
            }

            writeln!(file).unwrap();
            println!();
        }
    }
}
