use nalgebra::vector;
use rapier2d::prelude::*;

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
    let chassis_mass: f32 = 80.0;
    let weight_fraction_on_wheel: f32 = 0.5;

    let motor_torque_command: f32 = 50.0;
    let motor_max_torque: f32 = 80.0;
    let drivetrain_rotational_damping: f32 = 15.0;

    let tire_k: f32 = 1500.0;
    let tire_saturation_a: f32 = 0.05;

    let mu: f32 = 0.9;
    let g = 9.81_f32;
    let normal_force = (chassis_mass + wheel_mass) * g * weight_fraction_on_wheel;
    let tire_f_max = mu * normal_force;

    let rolling_resistance: f32 = 12.0;
    let aero_c1: f32 = 2.0;
    let aero_c2: f32 = 0.25;

    let max_omega: f32 = 100.0;
    let v_eps: f32 = 0.5;

    // ----------------------
    // Ground
    // ----------------------
    let ground_rb = RigidBodyBuilder::fixed()
        .translation(vector![0.0, -0.2])
        .build();
    let ground_handle = bodies.insert(ground_rb);
    let ground_col = ColliderBuilder::cuboid(50.0, 0.2)
        .friction(1.0)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(ground_col, ground_handle, &mut bodies);

    // ----------------------
    // Chassis
    // ----------------------
    let chassis_start_y = 0.7;
    let chassis_rb = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, chassis_start_y])
        .linear_damping(1.0)
        .build();
    let chassis_handle = bodies.insert(chassis_rb);

    let chassis_hw = 0.6_f32;
    let chassis_hh = 0.2_f32;
    let chassis_density = chassis_mass / (4.0 * chassis_hw * chassis_hh);
    let chassis_col = ColliderBuilder::cuboid(chassis_hw, chassis_hh)
        .density(chassis_density)
        .friction(0.5)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(chassis_col, chassis_handle, &mut bodies);

    // ----------------------
    // Wheel
    // ----------------------
    let wheel_start_pos = vector![0.4, wheel_radius];
    let wheel_rb = RigidBodyBuilder::dynamic()
        .translation(wheel_start_pos)
        .angvel(0.0)
        .angular_damping(2.0)
        .build();
    let wheel_handle = bodies.insert(wheel_rb);

    let wheel_area = std::f32::consts::PI * wheel_radius.powi(2);
    let wheel_density = wheel_mass / wheel_area;
    let wheel_col = ColliderBuilder::ball(wheel_radius)
        .density(wheel_density)
        .friction(mu)
        .restitution(0.1)
        .build();
    colliders.insert_with_parent(wheel_col, wheel_handle, &mut bodies);

    // ----------------------
    // Revolute joint
    // ----------------------
    let axle_world_pos = wheel_start_pos;
    let chassis_local = bodies[chassis_handle].position().inverse() 
        * Isometry::new(axle_world_pos, 0.0);
    let wheel_local = bodies[wheel_handle].position().inverse() 
        * Isometry::new(axle_world_pos, 0.0);

    let joint = RevoluteJointBuilder::new()
        .local_anchor1(chassis_local.translation.vector.into())
        .local_anchor2(wheel_local.translation.vector.into())
        .build();
    joints.insert(chassis_handle, wheel_handle, joint, true);

    // ----------------------
    // Simulation loop
    // ----------------------
    let total_steps = 3_000;
    
    for step in 0..total_steps {
        // Cap omega at start of step
        {
            let mut wheel_rb = bodies.get_mut(wheel_handle).unwrap();
            let omega = wheel_rb.angvel();
            if !omega.is_finite() || omega.abs() > max_omega {
                wheel_rb.set_angvel(omega.signum() * max_omega.min(omega.abs()), true);
            }
        }

        // Read state
        let (v_chassis, wheel_omega) = {
            let chassis_rb = bodies.get(chassis_handle).unwrap();
            let wheel_rb = bodies.get(wheel_handle).unwrap();
            (chassis_rb.linvel().x, wheel_rb.angvel())
        };

        // Safety check
        if !v_chassis.is_finite() || !wheel_omega.is_finite() {
            println!("NaN detected at step {}, resetting...", step);
            let mut chassis_rb = bodies.get_mut(chassis_handle).unwrap();
            chassis_rb.set_linvel(vector![0.0, 0.0], true);
            chassis_rb.set_angvel(0.0, true);
            let mut wheel_rb = bodies.get_mut(wheel_handle).unwrap();
            wheel_rb.set_linvel(vector![0.0, 0.0], true);
            wheel_rb.set_angvel(0.0, true);
            continue;
        }

        // Slip calculation
        let v_wheel_surface = wheel_omega * wheel_radius;
        let v_ref = v_chassis.abs().max(v_eps);
        let slip = (v_wheel_surface - v_chassis) / v_ref;
        let slip_clamped = slip.clamp(-50.0, 50.0);

        // Tire force at contact patch (ground pushing on wheel)
        let f_tire_raw = tire_k * slip_clamped / (1.0 + tire_saturation_a * slip_clamped.abs());
        let f_tire = f_tire_raw.clamp(-tire_f_max, tire_f_max);

        // Drag on chassis (resists chassis motion)
        let drag = -v_chassis.signum() * rolling_resistance
                   - aero_c1 * v_chassis
                   - aero_c2 * v_chassis * v_chassis.abs();

        // Apply drag directly to chassis
        {
            let mut chassis_rb = bodies.get_mut(chassis_handle).unwrap();
            chassis_rb.add_force(vector![drag, 0.0], true);
        }

        // Apply all torques to wheel
        {
            let mut wheel_rb = bodies.get_mut(wheel_handle).unwrap();
            
            // Motor torque (drives wheel)
            let motor_tau = motor_torque_command.clamp(-motor_max_torque, motor_max_torque);
            
            // Drivetrain damping
            let damping_tau = -drivetrain_rotational_damping * wheel_omega;
            
            // Tire force creates torque at contact patch (radius below axle)
            // If tire pushes forward on wheel (+f_tire), it creates +torque (accelerates rotation)
            let tire_torque = f_tire * wheel_radius;
            
            let total_torque = motor_tau + damping_tau + tire_torque;
            
            wheel_rb.add_torque(total_torque, true);
        }

        if step % 120 == 0 {
            println!(
                "Step {} | slip={:.2} | v={:.2} m/s | ω={:.1} rad/s | F_tire={:.1} N",
                step, slip_clamped, v_chassis, wheel_omega, f_tire
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

    println!("Simulation finished.");
}