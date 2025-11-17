use std::time::Duration;

use pid_lite::Controller;

use crate::utils::{percentage::Percentage, speed::WheelSpeed, time::Timestamp};

// Linear curve over 5secs to full speed
fn level_0(msec: u32) -> Percentage {
    if msec > 5000 {
        Percentage::full()
    } else {
        Percentage::from(0.02 * msec as f32)
    }
}

// Linear curve over 1secs to full speed
fn level_1(msec: u32) -> Percentage {
    if msec > 1000 {
        Percentage::full()
    } else {
        Percentage::from(0.1 * msec as f32)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TractionControlMode {
    Level0(),
    Level1(),
}

impl Into<u8> for TractionControlMode {
    fn into(self) -> u8 {
        match self {
            TractionControlMode::Level0() => 0,
            TractionControlMode::Level1() => 1,
        }
    }
}

impl From<u8> for TractionControlMode {
    fn from(value: u8) -> Self {
        if value == 1 {
            TractionControlMode::Level1()
        } else {
            TractionControlMode::Level0()
        }
    }
}

impl TractionControlMode {
    pub fn prop_gain(&self) -> f64 {
        match self {
            TractionControlMode::Level0() => 0.1,
            TractionControlMode::Level1() => 0.5,
        }
    }
    pub fn int_gain(&self) -> f64 {
        match self {
            TractionControlMode::Level0() => 0.0,
            TractionControlMode::Level1() => 0.0,
        }
    }
    pub fn der_gain(&self) -> f64 {
        match self {
            TractionControlMode::Level0() => 0.0,
            TractionControlMode::Level1() => 0.0,
        }
    }
    pub fn scale_factor(&self) -> f64 {
        match self {
            TractionControlMode::Level0() => 1.0,
            TractionControlMode::Level1() => 1.0,
        }
    }
}

pub struct TractionControl {
    pub mode: TractionControlMode,
    prev_timestamp: Option<Timestamp>,
    controller: Controller,
    desired_slip: Percentage,
}

impl TractionControl {
    pub fn new(mode: TractionControlMode, desired_slip: Percentage) -> Self {
        let controller = Controller::new(
            Into::<f32>::into(desired_slip) as f64,
            mode.prop_gain(),
            mode.int_gain(),
            mode.der_gain(),
        );
        TractionControl {
            mode,
            prev_timestamp: None,
            controller: controller,
            desired_slip: desired_slip,
        }
    }
    pub fn update_mode(&mut self, mode: TractionControlMode) {
        self.controller.set_derivative_gain(mode.der_gain());
        self.controller.set_integral_gain(mode.int_gain());
        self.controller.set_proportional_gain(mode.prop_gain());
        self.mode = mode;
    }
    pub fn update_desired_slip(&mut self, desired_slip: Percentage) {
        self.desired_slip = desired_slip;
        self.controller.set_target(Into::<f64>::into(desired_slip));
    }

    pub fn run_algo(
        &mut self,
        curr_time: Timestamp,
        current_slip: Percentage,
        curr_req: Percentage,
    ) -> Percentage {
        let elapsed_time = if let Some(prev_time) = self.prev_timestamp {
            Duration::from_micros((curr_time - prev_time).as_micros())
        } else {
            Duration::from_micros(0)
        };
        let adjustment = self
            .controller
            .update_elapsed(current_slip.into(), elapsed_time);

        if adjustment <= 0.0 {
            curr_req
        // only modify adjustment if it's negative (e.g. we should reduce our slip angle)
        } else {
            ((adjustment * self.mode.scale_factor()) + Into::<f64>::into(curr_req)).into()
        }
    }

    pub fn reset(&mut self) {
        self.prev_timestamp = None;
        self.controller.reset();
    }
}
