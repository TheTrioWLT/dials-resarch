use serde::{Deserialize, Serialize};
use std::{thread, time::Instant};

pub const DIAL_MAX_VALUE: f32 = 10000.0;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct DialRange {
    pub start: f32,
    pub end: f32,
}

impl DialRange {
    pub fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, value: f32) -> bool {
        value <= self.end && value >= self.start
    }

    pub fn random_in(&self) -> f32 {
        self.start + (self.end - self.start) * rand::random::<f32>()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DialReaction {
    pub dial_id: usize,
    pub millis: u32,
    pub correct_key: bool,
    pub key: char,
}

#[derive(Debug, Copy, Clone)]
pub struct DialAlarm {
    pub dial_id: usize,
    pub time: Instant,
    pub correct_key: char,
}

impl DialAlarm {
    pub fn new(dial_id: usize, time: Instant, correct_key: char) -> Self {
        Self {
            dial_id,
            time,
            correct_key,
        }
    }
}

impl DialReaction {
    pub fn new(dial_id: usize, millis: u32, correct_key: bool, key: char) -> Self {
        Self {
            dial_id,
            millis,
            correct_key,
            key,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Dial {
    value: f32,
    dial_id: usize,
    rate: f32,
    in_range: DialRange,
    key: char,
    alarm_fired: bool,
}

impl Dial {
    pub fn new(dial_id: usize, rate: f32, in_range: DialRange, key: char) -> Self {
        let mut dial = Self {
            value: 0.0,
            dial_id,
            rate,
            in_range,
            key,
            alarm_fired: false,
        };

        // Immediately "reset"
        dial.reset();

        dial
    }

    pub fn reset(&mut self) {
        let reset_value = self.in_range.random_in();

        let new_rate = if rand::random::<bool>() {
            self.rate
        } else {
            -self.rate
        };

        self.value = reset_value;
        self.rate = new_rate;
        self.alarm_fired = false;
    }

    /// Updates the dial using the amount of time that has passed since the last update
    /// A DialReaction data structure is returned if this dial has gone out of range.
    pub fn update(&mut self, delta_time: f32) -> Option<DialAlarm> {
        // Increment the value using the rate and the delta time
        self.increment_value(delta_time * self.rate);

        if !self.alarm_fired && !self.in_range.contains(self.value) {
            self.on_out_of_range();

            let dial_alarm = DialAlarm::new(self.dial_id, Instant::now(), self.key);

            Some(dial_alarm)
        } else {
            None
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn in_range(&self) -> DialRange {
        self.in_range
    }

    fn on_out_of_range(&mut self) {
        thread::spawn(|| crate::audio::play().unwrap());
        self.alarm_fired = true;
    }

    fn increment_value(&mut self, increment: f32) {
        self.value = (self.value + increment) % DIAL_MAX_VALUE;
    }
}
