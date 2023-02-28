use derive_new::new;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The value required to have the needle point to the very end of the dial
pub const DIAL_MAX_VALUE: f32 = 10000.0;
/// The maximum number of segments in a random path that is traversed by the dial in its wandering
const MAX_PATH_SEGMENTS: usize = 8;
/// The minimum number of segments in a random path that is traversed by the dial in its wandering
const MIN_PATH_SEGMENTS: usize = 4;
/// The number of seconds that a path for the dial should be generated for, for after the dial is reset
const AFTER_RESET_PATH_TIME: usize = 3600;
/// The number of seconds per path segment for after the dial has been reset. This determines the number
/// of segments
const AFTER_RESET_SECONDS_PER_SEGMENT: f32 = 2.0;

/// A "range" which a dial can be inside or out of. This is used to keep track of if the dial is
/// "in range" so that we know when to sound an alarm.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, new)]
pub struct DialRange {
    /// The "start" of the range. This should always be *less* than what the value should be
    pub start: f32,
    /// The "end" of the range. This should always be *greater* than what the value should be
    pub end: f32,
}

impl DialRange {
    /// Returns true if the value is contained within this range "inclusively"
    pub fn contains(&self, value: f32) -> bool {
        value <= self.end && value >= self.start
    }

    /// Returns the value that is in the direct middle of the range
    pub fn middle(&self) -> f32 {
        (self.end - self.start) / 2.0 + self.start
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

    /// Returns a random value that is inside of this range, with no other constraints
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

/// Represents a dial inside of our application "model"
#[derive(Debug, Clone)]
pub struct Dial {
    // The current value of the dial, which is where the needle is pointing
    value: f32,
    // The name of this dial
    name: String,
    // The "in-range" for this dial: where it is supposed to be, and if it exits, the alarm sounds
    in_range: DialRange,
    // This dial's random paths that it needs to traverse in order to drift up and down
    path: VecDeque<PathSegment>,
    // The current time into the current path segment
    segment_time: f32,
    // The current direction of travel in the path segment.
    travel_direction: f32,
}

impl Dial {
    /// Creates a new Dial with the provided name and in-range
    pub fn new(name: String, in_range: DialRange) -> Self {
        // We will wander before our trial time
        let wander_path_segments =
            (AFTER_RESET_PATH_TIME as f32 / AFTER_RESET_SECONDS_PER_SEGMENT) as usize;

        Self {
            value: in_range.middle(),
            name,
            in_range,
            path: generate_random_dial_path(
                &in_range,
                AFTER_RESET_PATH_TIME as f32,
                false,
                wander_path_segments,
                wander_path_segments,
            ),
            segment_time: 0.0,
            travel_direction: 1.0,
        }
    }

    /// Resets the dial to the middle of the range until the program is over. This means there are no
    /// more trials that use this dial.
    pub fn reset(&mut self) {
        self.path.clear();
        self.value = self.in_range.middle();
    }

    /// Updates the dial using the amount of time that has passed since the last update
    /// Returns a bool stating whether or not the dial has drifted out of range this update.
    /// It only returns true once, and then it must be reset
    pub fn update(&mut self, delta_time: f32) {
        // Update the current time within the segment
        self.segment_time += delta_time;

        if let Some(current) = self.path.front() {
            // If we are still in our current path segment
            if current.in_segment(self.segment_time) {
                // Calculate our current position in the path at the current time
                self.value = current.value_at_time(self.segment_time);
            } else {
                // Move onto the next path segment
                self.travel_direction = current.travel_direction();
                self.path.pop_front();
                self.segment_time = 0.0;
            }
        }
    }

    /// The value of the dial, where it is currently pointing
    pub fn value(&self) -> f32 {
        self.value
    }

    /// The in-range for this dial
    pub fn in_range(&self) -> DialRange {
        self.in_range
    }

    // The unique name of this dial
    pub fn name(&self) -> &String {
        &self.name
    }
}

/// A single segment in a Dial's random path that it traverses over time
#[derive(Debug, Clone, Copy)]
struct PathSegment {
    /// The start position of the dial
    start: f32,
    /// The end position of the dial
    end: f32,
    /// The time that this path segment should take
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
