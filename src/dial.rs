use derive_new::new;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The value required to have the needle point to the very end of the dial
pub const DIAL_MAX_VALUE: f32 = 10000.0;
/// The average number of seconds per segment that should be used when the dial has a time to drift out
const SECONDS_PER_SEGMENT: f32 = 2.0;
/// The random deviation allowed in the seconds per segment
const SECONDS_PER_SEGMENT_DEVIATION: f32 = 1.0;
/// The number of path segments that should be generated after the dial is reset for the final time
const AFTER_RESET_PATH_SEGMENTS: usize = 4000;

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

    /// Returns either self.start or self.end depending on which is closer to the provided value
    pub fn end_closer_to_point(&self, value: f32) -> f32 {
        let halfway = (self.end - self.start) / 2.0 + self.start;

        if value <= halfway {
            self.start
        } else {
            self.end
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
    // If this dial is randomly wandering in its range or if it is scheduled to drift out
    is_wandering: bool,
    // The current time into the current path segment
    segment_time: f32,
    // The current direction of travel in the path segment.
    travel_direction: f32,
}

impl Dial {
    /// Creates a new Dial with the provided name and in-range
    pub fn new(name: String, in_range: DialRange) -> Self {
        Self {
            value: in_range.middle(),
            name,
            in_range,
            path: generate_random_dial_path(&in_range, in_range.middle(), None),
            is_wandering: true,
            segment_time: 0.0,
            travel_direction: 1.0,
        }
    }

    /// Resets the dial to the middle of the range and continues "wandering"
    /// If a drift out time is specified, that is used to generate the path, if not the dial will
    /// drift "forever"
    pub fn reset(&mut self, drift_out_time: Option<f32>) {
        self.segment_time = 0.0;
        self.is_wandering = drift_out_time.is_none();
        self.path = if self.is_wandering {
            generate_random_dial_path(&self.in_range, self.in_range.middle(), drift_out_time)
        } else {
            generate_random_dial_path(&self.in_range, self.value, drift_out_time)
        }
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

    // If the dial is just wandering or if it was told to drift out yet
    pub fn is_wandering(&self) -> bool {
        self.is_wandering
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
    start_value: f32,
    drift_out_time: Option<f32>,
) -> VecDeque<PathSegment> {
    const MIN_SEGMENT_TIME: f32 = SECONDS_PER_SEGMENT - SECONDS_PER_SEGMENT_DEVIATION;
    const MAX_SEGMENT_TIME: f32 = SECONDS_PER_SEGMENT + SECONDS_PER_SEGMENT_DEVIATION;

    let mut segments = VecDeque::new();

    if let Some(drift_out_time) = drift_out_time {
        let mut time_remaining = drift_out_time;
        let mut start = range.random_in();
        let mut end = range.slightly_out(start);

        while time_remaining > MAX_SEGMENT_TIME {
            let duration = rand::thread_rng().gen_range(MIN_SEGMENT_TIME..=MAX_SEGMENT_TIME);

            let segment = PathSegment {
                start,
                end,
                duration,
            };

            segments.push_front(segment);

            let last_end = end;
            end = start;
            start = range.random_near(last_end);
            time_remaining -= duration;
        }

        let final_segment = PathSegment {
            start: start_value,
            end,
            duration: time_remaining,
        };

        segments.push_front(final_segment);
    } else {
        let mut last_value = start_value;

        for _ in 0..AFTER_RESET_PATH_SEGMENTS {
            let next_value = range.random_near(last_value);
            let duration = rand::thread_rng().gen_range(MIN_SEGMENT_TIME..=MAX_SEGMENT_TIME);

            let segment = PathSegment {
                start: last_value,
                end: next_value,
                duration,
            };

            segments.push_back(segment);

            last_value = next_value;
        }
    }

    segments
}

fn sigmoid(a: f32) -> f32 {
    1.0 / (1.0 + (-a).exp())
}
