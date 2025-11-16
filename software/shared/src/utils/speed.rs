use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub struct WheelSpeed {
    rpm: u16,
}

impl From<u16> for WheelSpeed {
    fn from(value: u16) -> Self {
        Self { rpm: value }
    }
}

impl Into<u16> for WheelSpeed {
    fn into(self) -> u16 {
        return self.rpm;
    }
}

impl From<f32> for WheelSpeed {
    fn from(value: f32) -> Self {
        Self { rpm: value as u16 }
    }
}

impl Into<f32> for WheelSpeed {
    fn into(self) -> f32 {
        return self.rpm as f32;
    }
}

impl Ord for WheelSpeed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rpm.cmp(&other.rpm)
    }
}

impl WheelSpeed {
    pub fn to_packets(&self) -> [u8; 2] {
        [(self.rpm & 0xFF) as u8, (self.rpm >> 8) as u8]
    }
    pub fn from_packets(data: &[u8; 2]) -> Self {
        Self {
            rpm: (data[0] as u16) | ((data[1] as u16) << 8),
        }
    }
}

pub struct GroundSpeed {
    pub mph: f32,
}

impl GroundSpeed {
    pub fn from_wheel_speed(wheel_speed: WheelSpeed, wheel_diameter_inch: f32) -> Self {
        // Convert wheel diameter from inches to miles
        let wheel_diameter_miles = wheel_diameter_inch / 63360.0; // 1 mile = 63360 inches
        // Calculate circumference in miles
        let circumference_miles = std::f32::consts::PI * wheel_diameter_miles;
        // Convert rpm to rph (revolutions per hour)
        let rph = (wheel_speed.rpm as f32) * 60.0;
        // Calculate ground speed in mph
        let mph = rph * circumference_miles;
        Self { mph }
    }
}
