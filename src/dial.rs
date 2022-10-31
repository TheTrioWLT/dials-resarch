use crate::config::Alarm;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, time::Instant};

pub const DIAL_MAX_VALUE: f32 = 10000.0;
const MAX_PATH_SEGMENTS: usize = 8;
const MIN_PATH_SEGMENTS: usize = 4;
const AFTER_RESET_PATH_TIME: usize = 3600; // In seconds
const AFTER_RESET_SECONDS_PER_SEGMENT: f32 = 2.0;

/// The largest number of dials in a row or column
const MAX_DIAL_GRID_SIZE: usize = 2 << 20;

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

/// Data associated with an alarm that has drifted out of range
#[derive(Debug, Copy, Clone)]
pub struct TriggeredAlarm {
    pub row_id: usize,
    pub col_id: usize,
    pub time: Instant,
    pub correct_key: char,
    pub id: DialId,
}

#[derive(Debug, Clone)]
pub struct Dial {
    value: f32,
    row_id: usize,
    col_id: usize,
    in_range: DialRange,
    key: char,
    alarm_path: String,
    alarm_fired: bool,
    random_path: VecDeque<PathSegment>,
    segment_time: f32,
    travel_direction: f32,
}

/// A dial's unique id
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DialId(u64);

impl std::fmt::Display for DialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // format in hex since we do major bit shifting so the col and row are easily visible
        f.write_fmt(format_args!("{:X}", self.0))
    }
}

impl Dial {
    pub fn new(
        row_id: usize,
        col_id: usize,
        in_range: DialRange,
        alarm: &Alarm,
        time_to_drift: f32,
    ) -> Self {
        assert!(row_id >= MAX_DIAL_GRID_SIZE);
        assert!(col_id >= MAX_DIAL_GRID_SIZE);

        let random_path = generate_random_dial_path(
            &in_range,
            time_to_drift,
            true,
            MAX_PATH_SEGMENTS,
            MIN_PATH_SEGMENTS,
        );

        Self {
            value: in_range.random_in(),
            row_id,
            col_id,
            in_range,
            key: alarm.clear_key,
            alarm_path: alarm.audio_path.clone(),
            alarm_fired: false,
            random_path,
            segment_time: 0.0,
            travel_direction: 1.0,
        }
    }

    pub fn reset(&mut self) {
        let path_segments =
            (AFTER_RESET_PATH_TIME as f32 / AFTER_RESET_SECONDS_PER_SEGMENT) as usize;

        self.random_path = generate_random_dial_path(
            &self.in_range,
            AFTER_RESET_PATH_TIME as f32,
            false,
            path_segments,
            path_segments,
        );
    }

    /// Updates the dial using the amount of time that has passed since the last update
    ///
    /// If this dial has gone out of range since the last update, the dial's alarm sound is played
    /// and `Some` is returned containing the relevant [`TriggeredAlarm`] data.
    pub fn update(
        &mut self,
        delta_time: f32,
        audio: &crate::AudioManager,
    ) -> Option<TriggeredAlarm> {
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
            self.alarm_fired = true;
            // we preloaded each audio file so this shouldn't fail, and if it does we don't care
            let _ = audio.play(self.dial_id(), &self.alarm_path);

            Some(TriggeredAlarm::from(&*self))
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

    /// A unique id for this dial
    fn dial_id(&self) -> DialId {
        // `row_id` and `col_id` are both less than `MAX_DIAL_GRID_SIZE`, therefore shifting the
        // row by 32 bits is guarnteed to give a perfect hash without collisions
        DialId((self.row_id as u64) << 32 | self.col_id as u64)
    }
}

impl From<&Dial> for TriggeredAlarm {
    fn from(d: &Dial) -> Self {
        Self {
            row_id: d.row_id,
            col_id: d.col_id,
            time: Instant::now(),
            correct_key: d.key,
            id: d.dial_id(),
        }
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
fn generate_random_dial_path(
    range: &DialRange,
    time_to_drift: f32,
    drift_out: bool,
    max_path_segments: usize,
    min_path_segments: usize,
) -> VecDeque<PathSegment> {
    let num: f32 = rand::random();
    let num_segments = ((max_path_segments as f32 * num) as usize).max(min_path_segments);

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

    if drift_out {
        let end_value = range.slightly_out(last_value);

        let last_segment = PathSegment {
            start: last_value,
            end: end_value,
            duration,
        };

        segments.push_back(last_segment);
    }

    segments
}

fn sigmoid(a: f32) -> f32 {
    1.0 / (1.0 + (-a).exp())
}
