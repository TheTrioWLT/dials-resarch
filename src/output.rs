use derive_new::new;
use std::io::Write;

const CSV_HEADERS: &str = "alarm, rms_error, response_time, correct_key, key";

pub struct SessionOutput {
    pub dial_reactions: Vec<DialReaction>,
    pub output_path: String,
}

/// Information about a user's response to an instance of an alarm being fired
#[derive(Debug, Clone, new)]
pub struct DialReaction {
    pub alarm_name: String,
    pub millis: u32,
    pub correct_key: bool,
    pub key: char,
    pub rms_error: f32,
}

impl SessionOutput {
    pub fn new(output_path: String) -> Self {
        Self {
            dial_reactions: Vec::new(),
            output_path,
        }
    }

    pub fn add_reaction(&mut self, reaction: DialReaction) {
        self.dial_reactions.push(reaction);
    }

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
