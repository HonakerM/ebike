use nalgebra::vector;
use rapier2d::prelude::*;
use std::fs::File;
use std::io::Write;

fn main() {
    // ----------------------
    // World / solver setup
    // ----------------------
    let gravity = vector![0.0, -9.81];
    let mut pipeline = PhysicsPipeline::new();
    let mut islands = IslandManager::new();
    let mut broad_phase = BroadPhaseBvh::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut impulse_joints = ImpulseJointSet::new();
    let mut joints = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();

    let integration_params = IntegrationParameters {
        dt: 1.0 / 120.0,
        ..Default::default()
    };

    let physics_hooks = ();
    let event_handler = ();

    // ----------------------
    // Physical parameters
    // ----------------------
    let wheel_radius: f32 = 0.35;
    let wheel_mass: f32 = 6.0;
    let chassis_mass: f32 = 100.0;

    // Motor parameters
    let motor_torque: f32 = 40.0; // N·m

    // Tire slip model parameters - much gentler
    let slip_stiffness: f32 = 100.0; // N per unit slip ratio
    
    // Friction limit
    let mu: f32 = 0.9;
    let g = 9.81_f32;
    let weight_per_wheel = (chassis_mass + wheel_mass) * g / 2.0;
    let max_tire_force = mu * weight_per_wheel;

    // Drag coefficients
    let rolling_resistance: f32 = 10.0; // N
    let aero_drag: f32 = 0.3; // N/(m/s)^2

    // ----------------------
    // Ground
    // ----------------------
    let ground_rb = RigidBodyBuilder::fixed()
        .translation(vector![0.0, -0.2])
        .build();
    let ground_handle = bodies.insert(ground_rb);
    let ground_col = ColliderBuilder::cuboid(100000.0, 0.2)
        .friction(1.0)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(ground_col, ground_handle, &mut bodies);

    // ----------------------
    // Chassis
    // ----------------------
    let chassis_width = 0.8;
    let chassis_y = (2.0*wheel_radius) + 0.15; // Bottom of chassis just above wheel tops
    let chassis_rb = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, chassis_y])
        .linear_damping(0.1)
        .angular_damping(1.0)
        .build();
    let chassis_handle = bodies.insert(chassis_rb);

    let chassis_col = ColliderBuilder::cuboid(chassis_width, 0.15)
        .density(chassis_mass / (1.6 * 0.3))
        .build();
    colliders.insert_with_parent(chassis_col, chassis_handle, &mut bodies);

    // ----------------------
    // Rear Wheel (driven)
    // ----------------------
    let rear_x = -(chassis_width/2.0+0.1);
    let rear_pos = vector![rear_x, wheel_radius];
    let rear_rb = RigidBodyBuilder::dynamic()
        .translation(rear_pos)
        .angular_damping(0.0)
        .build();
    let rear_handle = bodies.insert(rear_rb);

    let wheel_density = wheel_mass / (std::f32::consts::PI * wheel_radius.powi(2));
    let rear_col = ColliderBuilder::ball(wheel_radius)
        .density(wheel_density)
        .friction(mu)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(rear_col, rear_handle, &mut bodies);

    // ----------------------
    // Front Wheel (free)
    // ----------------------
    let front_x = -(rear_x);
    let front_pos = vector![front_x, wheel_radius];
    let front_rb = RigidBodyBuilder::dynamic()
        .translation(front_pos)
        .angular_damping(0.0)
        .build();
    let front_handle = bodies.insert(front_rb);

    let front_col = ColliderBuilder::ball(wheel_radius)
        .density(wheel_density)
        .friction(mu)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(front_col, front_handle, &mut bodies);

    // ----------------------
    // Joints - use actual world positions
    // ----------------------
    // Rear joint: chassis at (0, chassis_y), rear wheel at (rear_x, wheel_radius)
    // Joint position is at wheel center
    let rear_joint = RevoluteJointBuilder::new()
        .local_anchor1(vector![rear_x, wheel_radius - chassis_y].into()) // In chassis frame
        .local_anchor2(vector![0.0, 0.0].into()) // At wheel center
        .build();
    joints.insert(chassis_handle, rear_handle, rear_joint, true);

    // Front joint: chassis at (0, chassis_y), front wheel at (front_x, wheel_radius)
    let front_joint = RevoluteJointBuilder::new()
        .local_anchor1(vector![front_x, wheel_radius - chassis_y].into()) // In chassis frame
        .local_anchor2(vector![0.0, 0.0].into()) // At wheel center
        .build();
    joints.insert(chassis_handle, front_handle, front_joint, true);

    // ----------------------
    // Simulation loop
    // ----------------------
    let total_steps = 6_000;
    let dt = integration_params.dt;
    
    for step in 0..total_steps {
        let time = step as f32 * dt;

        // Read state
        let v_chassis = bodies.get(chassis_handle).unwrap().linvel().x;
        let omega_rear = bodies.get(rear_handle).unwrap().angvel();
        let omega_front = bodies.get(front_handle).unwrap().angvel();

        // Calculate slip ratio for rear wheel
        // slip = (wheel_surface_speed - vehicle_speed) / |vehicle_speed|
        let wheel_surface_speed = omega_rear * wheel_radius;
        let v_ref = v_chassis.abs().max(0.1); // Avoid division by small numbers
        let slip = (wheel_surface_speed - v_chassis) / v_ref;
        
        // Tire force from slip (linear with saturation)
        //let tire_force_desired = slip_stiffness * slip;
        //let tire_force = tire_force_desired.clamp(-max_tire_force, max_tire_force);
        let tire_force = 0.0;

        // Drag forces
        let drag = -rolling_resistance * v_chassis.signum() 
                   - aero_drag * v_chassis * v_chassis.abs();

        // Apply forces to chassis
        {
            let mut chassis_rb = bodies.get_mut(chassis_handle).unwrap();
            //chassis_rb.add_force(vector![tire_force + drag, 0.0], true);
        }

        // Apply torques to rear wheel
        let tire_torque = -tire_force * wheel_radius; // Tire force opposes wheel spin
        {
            let mut rear_rb = bodies.get_mut(rear_handle).unwrap();
            //rear_rb.add_torque(motor_torque + tire_torque, true);
        }

        // Console output
        if step % 30 == 0 {
            println!(
                "t={:.1}s | v={:.2} m/s | slip={:.3} | ω_r={:.1} | ω_f={:.1} | d_w_f={:.1} | F_tire={:.1} N",
                time, v_chassis, slip, omega_rear,omega_front,v_chassis/ wheel_radius, tire_force
            );
        }

        // Step physics
        pipeline.step(
            &gravity,
            &integration_params,
            &mut islands,
            &mut broad_phase,
            &mut narrow_phase,
            &mut bodies,
            &mut colliders,
            &mut impulse_joints,
            &mut joints,
            &mut ccd_solver,
            &physics_hooks,
            &event_handler,
        );
    }
}