use derive_new::new;
use std::io::Write;

/// A constant for the CSV file headers
const CSV_HEADERS: &str = "alarm, rms_error, response_time, correct_key, key";

/// A struct that helps to collect DialReactions and can output them to a CSV file
pub struct SessionOutput {
    /// The current dial reactions in chronological order
    pub dial_reactions: Vec<DialReaction>,
    /// The output path to the CSV
    pub output_path: String,
}

/// Information about a user's response to an instance of an alarm being fired
#[derive(Debug, Clone, new)]
pub struct DialReaction {
    /// The name of the alarm that the dial reaction was to
    pub alarm_name: String,
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
            dial_reactions: Vec::new(),
            output_path,
        }
    }

    /// Adds a DialReaction to be outputted
    pub fn add_reaction(&mut self, reaction: DialReaction) {
        self.dial_reactions.push(reaction);
    }

    /// Writes all of the currently held DialReactions to the SessionOutput's path in CSV format
    pub fn write_to_file(&self) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.output_path)
            .unwrap();

        writeln!(file, "{CSV_HEADERS}").unwrap();
        println!("{CSV_HEADERS}");

        for reaction in &self.dial_reactions {
            writeln!(
                file,
                "{}, {}, {}, {}, {}",
                reaction.alarm_name,
                reaction.rms_error,
                reaction.millis,
                reaction.correct_key,
                reaction.key
            )
            .unwrap();
            println!(
                "{}, {}, {}, {}, {}",
                reaction.alarm_name,
                reaction.rms_error,
                reaction.millis,
                reaction.correct_key,
                reaction.key
            );
        }
    }
}
