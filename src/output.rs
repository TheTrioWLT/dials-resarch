use std::io::Write;

use crate::dial::DialReaction;

const CSV_HEADERS: &str = "alarm, rms_error, response_time, correct_key, key";

pub struct SessionOutput {
    pub dial_reactions: Vec<DialReaction>,
    pub output_path: String,
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

        for reaction in &self.dial_reactions {
            writeln!(
                file,
                "{}, {}, {}, {}, {}",
                reaction.dial_id,
                reaction.rms_error,
                reaction.millis,
                reaction.correct_key,
                reaction.key
            )
            .unwrap();
            println!(
                "{}, {}, {}, {}, {}",
                reaction.dial_id,
                reaction.rms_error,
                reaction.millis,
                reaction.correct_key,
                reaction.key
            );
        }
    }
}
