use std::ops::{Add, Div, Mul, Sub};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Percentage {
    // stored as raw value percetnage e.g 100.0% is 100.0
    raw_val: f32,
}

impl From<u8> for Percentage {
    fn from(value: u8) -> Self {
        Self {
            raw_val: (value as f32 / std::u8::MAX as f32),
        }
    }
}

impl Into<u8> for Percentage {
    fn into(self) -> u8 {
        ((self.raw_val) * (std::u8::MAX as f32)) as u8
    }
}

impl From<f32> for Percentage {
    fn from(value: f32) -> Self {
        Self { raw_val: value }
    }
}

impl Into<f32> for Percentage {
    fn into(self) -> f32 {
        return self.raw_val;
    }
}

impl From<f64> for Percentage {
    fn from(value: f64) -> Self {
        Self {
            raw_val: value as f32,
        }
    }
}

impl Into<f64> for Percentage {
    fn into(self) -> f64 {
        return self.raw_val as f64;
    }
}

impl Add for Percentage {
    type Output = Percentage;

    fn add(self, other: Percentage) -> Percentage {
        Percentage {
            raw_val: self.raw_val + other.raw_val,
        }
    }
}

impl Sub for Percentage {
    type Output = Percentage;

    fn sub(self, other: Percentage) -> Percentage {
        Percentage {
            raw_val: self.raw_val - other.raw_val,
        }
    }
}

impl Mul<Percentage> for Percentage {
    type Output = Percentage;

    fn mul(self, rhs: Percentage) -> Percentage {
        Percentage {
            raw_val: self.raw_val * rhs.raw_val,
        }
    }
}

impl Div<Percentage> for Percentage {
    type Output = Percentage;

    fn div(self, rhs: Percentage) -> Percentage {
        Percentage {
            raw_val: self.raw_val / rhs.raw_val,
        }
    }
}

impl Percentage {
    pub const fn from_fractional(value: f32) -> Self {
        Self { raw_val: value }
    }
    pub fn full() -> Self {
        return Percentage { raw_val: 1.0 };
    }
    pub fn zero() -> Self {
        return Percentage { raw_val: 0.0 };
    }
    pub fn clamp(mut self) {
        if self.raw_val < 0.0 {
            self.raw_val = 0.0;
        } else if self.raw_val > 1.0 {
            self.raw_val = 1.0;
        }
    }
}
