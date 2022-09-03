use crate::config::Alarm;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, sync::Arc, time::Instant};

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

    /// Returns a random value that is near the provided value within half of the maximum range
    pub fn random_near(&self, value: f32) -> f32 {
        // If we should increase or decrease
        let decrease: bool = rand::random();

        let random_magnitude = (self.end - self.start) * rand::random::<f32>();
        let mut tamed_magnitude = random_magnitude / 2.0;

        if decrease {
            tamed_magnitude *= -1.0;
        }

        if !self.contains(value + tamed_magnitude) {
            value - tamed_magnitude
        } else {
            value + tamed_magnitude
        }
    }

    pub fn random_in(&self) -> f32 {
        self.start + (self.end - self.start) * rand::random::<f32>()
    }

    /// Returns a value that is slightly outside of the range, useful for when we have to drift out
    /// but not too quickly. It takes into account the current value so that it can drift to the
    /// closer side
    pub fn slightly_out(&self, value: f32) -> f32 {
        let halfway = (self.end - self.start) / 2.0 + self.start;
        let amount = rand::random::<f32>() * 400.0;

        if value <= halfway {
            // Here we will choose a value that is less than our range
            self.start - amount
        } else {
            // Here we will choose a value that is greater than our range
            self.end + amount
        }
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

#[derive(Debug, Clone)]
pub struct Dial {
    value: f32,
    dial_id: usize,
    in_range: DialRange,
    key: char,
    alarm_path: String,
    alarm_fired: bool,
    audio: Arc<crate::audio::AudioManager>,
    random_path: VecDeque<PathSegment>,
    segment_time: f32,
    time_to_drift: f32,
    travel_direction: f32,
}

impl Dial {
    pub fn new(
        dial_id: usize,
        in_range: DialRange,
        alarm: &Alarm,
        audio: Arc<crate::audio::AudioManager>,
        time_to_drift: f32,
    ) -> Self {
        let random_path = generate_random_dial_path(&in_range, time_to_drift);

        Self {
            value: in_range.random_in(),
            dial_id,
            in_range,
            key: alarm.clear_key,
            alarm_path: alarm.audio_path.clone(),
            alarm_fired: false,
            random_path,
            segment_time: 0.0,
            time_to_drift,
            travel_direction: 1.0,
            audio,
        }
    }

    pub fn reset(&mut self) {
        self.random_path = generate_random_dial_path(&self.in_range, self.time_to_drift);
        self.value = self.in_range().random_in();
        self.alarm_fired = false;
    }

    /// Updates the dial using the amount of time that has passed since the last update
    /// A DialReaction data structure is returned if this dial has gone out of range.
    pub fn update(&mut self, delta_time: f32) -> Option<DialAlarm> {
        // Update the current time within the segment
        self.segment_time += delta_time;

        // If we haven't run out of path segments yet
        if let Some(current) = self.random_path.front() {
            // If we are still in our current path segment
            if current.in_segment(self.segment_time) {
                // Calculate our current position in the path at the current time
                self.value = current.value_at_time(self.segment_time);
            } else {
                // Move onto the next path segment
                self.travel_direction = current.travel_direction();
                self.random_path.pop_front();
                self.segment_time = 0.0;
            }
        } else {
            // Keep drifting
            self.value += self.travel_direction * 20.0 * delta_time;
        }

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
        // we preleaded each audio file so this shouldn't fail, and if it does we don't care
        log::info!("out of range");
        let _ = self.audio.play(&self.alarm_path);
        self.alarm_fired = true;
    }

    pub fn cheese_play(&self) {
        let _ = self.audio.play(&self.alarm_path);
    }
}

#[derive(Debug, Clone, Copy)]
struct PathSegment {
    start: f32,
    end: f32,
    duration: f32,
}

impl PathSegment {
    /// Returns the value at the time in the path segment.
    fn value_at_time(&self, time: f32) -> f32 {
        const X_OFFSET: f32 = -5.0;

        let scale_factor = self.end - self.start;
        let x_value = (10.0 / self.duration) * time;

        sigmoid(x_value + X_OFFSET) * scale_factor + self.start
    }

    /// Returns true if the time is within the path segment duration, false if not
    fn in_segment(&self, time: f32) -> bool {
        time <= self.duration
    }

    /// Returns 1.0 if the segment has the value increasing, and -1.0 if it is decreasing
    fn travel_direction(&self) -> f32 {
        if self.start < self.end {
            1.0
        } else {
            -1.0
        }
    }
}

/// Generates a new random dial path for the dial to perform within the time_to_drift, and within
/// the given range.
fn generate_random_dial_path(range: &DialRange, time_to_drift: f32) -> VecDeque<PathSegment> {
    const MAX_PATH_SEGMENTS: usize = 8;
    const MIN_PATH_SEGMENTS: usize = 4;

    let num: f32 = rand::random();
    let num_segments = ((MAX_PATH_SEGMENTS as f32 * num) as usize).max(MIN_PATH_SEGMENTS);

    let mut segments = VecDeque::with_capacity(num_segments);
    let mut last_value = range.random_in();

    let duration = time_to_drift / num_segments as f32;

    for _ in 0..(num_segments - 1) {
        let next_value = range.random_near(last_value);

        let segment = PathSegment {
            start: last_value,
            end: next_value,
            duration,
        };

        segments.push_back(segment);

        last_value = next_value;
    }

    let end_value = range.slightly_out(last_value);

    let last_segment = PathSegment {
        start: last_value,
        end: end_value,
        duration,
    };

    segments.push_back(last_segment);

    segments
}

fn sigmoid(a: f32) -> f32 {
    1.0 / (1.0 + (-a).exp())
}
