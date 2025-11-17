use std::{ops::{Add, Div, Mul, Sub}};

// pub struct for a timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Duration(u64);

impl Duration {
    // Create a new timestamp from nanoseconds.
    pub fn from_millis(ms: u64) -> Self {
        Duration(ms)
    }

    // Get the timestamp in nanoseconds.
    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, other: Duration) -> Duration {
        Duration(self.0 - other.0)
    }
}

impl Mul<u64> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u64) -> Duration {
        Duration(self.0 * rhs)
    }
}

impl Div<u64> for Duration {
    type Output = Duration;

    fn div(self, rhs: u64) -> Duration {
        Duration(self.0 / rhs)
    }
}

impl Add<u64> for Duration {
    type Output = Duration;

    fn add(self, rhs: u64) -> Duration {
        Duration(self.0 + rhs)
    }
}

impl From<std::time::Duration> for Duration {
    fn from(value: std::time::Duration) -> Self {
        Duration::from_millis(value.as_millis() as u64)
    }
}
impl Into<std::time::Duration> for Duration {
    fn into(self) -> std::time::Duration {
        std::time::Duration::from_millis(self.as_millis())
    }
}

// pub struct for a timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Timestamp(u64);

impl Timestamp {
    // Create a new timestamp from nanoseconds.
    pub fn from_micros(ms: u64) -> Self {
        Timestamp(ms)
    }

    // Get the timestamp in nanoseconds.
    pub fn as_micros(&self) -> u64 {
        self.0
    }
}

impl Sub for Timestamp {
    type Output = Timestamp;

    fn sub(self, other: Timestamp) -> Timestamp {
        Timestamp(self.0 - other.0)
    }
}

impl Mul<u64> for Timestamp {
    type Output = Timestamp;

    fn mul(self, rhs: u64) -> Timestamp {
        Timestamp(self.0 * rhs)
    }
}

impl Div<u64> for Timestamp {
    type Output = Timestamp;

    fn div(self, rhs: u64) -> Timestamp {
        Timestamp(self.0 / rhs)
    }
}

impl Add<u64> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: u64) -> Timestamp {
        Timestamp(self.0 + rhs)
    }
}
