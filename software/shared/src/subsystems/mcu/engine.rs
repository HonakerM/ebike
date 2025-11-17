use crate::{
    operations::{
        throttle_map::{ThottleMap, ThottleMapMode},
        traction_control::{TractionControl, TractionControlMode},
    },
    subsystems::shared::Subsystem,
    utils::{percentage::Percentage, speed::WheelSpeed, time::Timestamp},
};

#[derive(Debug, Clone, Copy)]
pub struct EngineRequest {
    pub rear_ws: Option<WheelSpeed>,
    pub front_ws: Option<WheelSpeed>,
    pub throttle_req: Percentage,
    pub timestamp: Timestamp,
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

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            throttle_map_mode: ThottleMapMode::Level2(),
            traction_control_mode: TractionControlMode::Level1(),
            desired_slip: Percentage::from_fractional(0.1),
        }
    }
}

pub struct EngineSubsystem {
    pub throttle_map: ThottleMap,
    pub traction_control: TractionControl,
}

impl Subsystem<EngineConfig, EngineRequest, EngineResponse> for EngineSubsystem {
    fn new(config: EngineConfig) -> Self {
        EngineSubsystem {
            throttle_map: ThottleMap::new(config.throttle_map_mode),
            traction_control: TractionControl::new(
                config.traction_control_mode,
                config.desired_slip,
            ),
        }
    }

    fn update(&mut self, config: EngineConfig) {
        println!("Engine Subsystem Config: {:?}", config);

        self.throttle_map.update_mode(config.throttle_map_mode);
        self.traction_control
            .update_mode(config.traction_control_mode);
        self.traction_control
            .update_desired_slip(config.desired_slip);
        self.reset();
    }

    fn reset(&mut self) {
        println!("Engine Subsystem Reset");
        self.traction_control.reset();
    }

    fn run(&mut self, req: EngineRequest) -> EngineResponse {
        println!("Engine Subsystem Response: {:?}", req);
        // reset subsystem back to default if no throttle request (it means we've finished this acceleration cycle)
        if req.throttle_req == Percentage::zero() {
            self.reset();
            return EngineResponse {
                throttle_req: Percentage::zero(),
            };
        }

        // calculate throttle position from map
        let mut desired_throttle = self.throttle_map.run_algo(req.throttle_req);

        // if we have ws info and detected slip run TC
        if let Some(rear_ws) = req.rear_ws {
            if let Some(front_ws) = req.front_ws {
                if rear_ws > front_ws {
                    let slip = Percentage::from_fractional(
                        ((Into::<f32>::into(rear_ws) - Into::<f32>::into(front_ws))
                            / Into::<f32>::into(rear_ws)),
                    );
                    desired_throttle =
                        self.traction_control
                            .run_algo(req.timestamp, slip, desired_throttle);
                }
            }
        }

        println!(
            "Engine Subsystem Response: Desired Throttle: {:?}",
            desired_throttle
        );

        EngineResponse {
            throttle_req: desired_throttle,
        }
    }
}
