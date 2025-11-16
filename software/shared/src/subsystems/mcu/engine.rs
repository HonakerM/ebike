use crate::{operations::{throttle_map::{ThottleMap, ThottleMapMode}, traction_control::{TractionControl, TractionControlMode}}, subsystems::shared::Subsystem, utils::{percentage::Percentage, speed::WheelSpeed}};




#[derive(Debug, Clone, Copy)]
pub struct EngineRequest {
    pub rear_ws: WheelSpeed,
    pub front_ws: WheelSpeed,
    pub throttle_req: Percentage,
    pub timestamp: u64,
}


#[derive(Debug, Clone, Copy)]
pub struct EngineResponse {
    pub throttle_req: Percentage,
}


#[derive(Debug, Clone, Copy)]
pub struct EngineConfig {
    pub throttle_map_mode: ThottleMapMode,
    pub traction_control_mode: TractionControlMode,
    pub desired_slip: Percentage,
}

pub struct EngineSubsystem {
    pub throttle_map: ThottleMap,
    pub traction_control: TractionControl,
}


impl Subsystem<EngineConfig, EngineRequest, EngineResponse> for EngineSubsystem {
    fn new(config: EngineConfig)->Self {
        EngineSubsystem {
            throttle_map: ThottleMap::new(config.throttle_map_mode),
            traction_control: TractionControl::new(config.traction_control_mode, config.desired_slip),
        }
    }

    fn update(&mut self, config: EngineConfig) {
        self.throttle_map.update_mode(config.throttle_map_mode);
        self.traction_control.update_mode(config.traction_control_mode);
        self.traction_control.update_desired_slip(config.desired_slip);
        self.reset();
    }

    fn reset(&mut self) {
        self.traction_control.reset();
    }

    fn run(&mut self, req: EngineRequest)->EngineResponse {
        // reset subsystem back to default if no throttle request (it means we've finished this acceleration cycle)
        if req.throttle_req == Percentage::zero() {
            self.reset();
            return EngineResponse { throttle_req: Percentage::zero() };
        }

        // calculate throttle position from map
        let mut desired_throttle = self.throttle_map.run_algo(req.throttle_req);

        // if we have wheel slip then run TC algo
        if req.rear_ws > req.front_ws {
            let slip = Percentage::from_fractional(((Into::<f32>::into(req.rear_ws) - Into::<f32>::into(req.front_ws)) / Into::<f32>::into(req.rear_ws)));
            desired_throttle = self.traction_control.run_algo(req.timestamp, slip, desired_throttle);
        }

        EngineResponse { throttle_req: desired_throttle }
    }
}