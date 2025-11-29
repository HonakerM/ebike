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
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut impulse_joints = ImpulseJointSet::new();
    let mut joints = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();

    // More stable integration
    let integration_params = IntegrationParameters {
        dt: 1.0 / 120.0, // Slower timestep for stability
        ..Default::default()
    };

    let physics_hooks = ();
    let event_handler = ();

    // ----------------------
    // Physical parameters
    // ----------------------
    let wheel_radius: f32 = 0.35;
    let wheel_mass: f32 = 2.0; // Lighter wheels
    let chassis_mass: f32 = 80.0;

    // Much gentler motor
    let motor_torque: f32 = 15.0; // N·m

    // Friction
    let mu: f32 = 0.8;

    // Drag
    let rolling_resistance: f32 = 8.0;
    let aero_drag: f32 = 0.2;

    // ----------------------
    // Ground
    // ----------------------
    let ground_rb = RigidBodyBuilder::fixed()
        .translation(vector![0.0, 0.0])
        .build();
    let ground_handle = bodies.insert(ground_rb);
    let ground_col = ColliderBuilder::cuboid(50.0, 0.5)
        .friction(mu)
        .build();
    colliders.insert_with_parent(ground_col, ground_handle, &mut bodies);

    // ----------------------
    // Chassis - simple box
    // ----------------------
    let wheelbase = 1.0; // Distance between wheels
    let chassis_height = 0.3;
    let chassis_y = wheel_radius + 0.5 + chassis_height / 2.0;
    
    let chassis_rb = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, chassis_y])
        .linear_damping(0.2)
        .angular_damping(1.0)
        .can_sleep(false)
        .build();
    let chassis_handle = bodies.insert(chassis_rb);

    let chassis_col = ColliderBuilder::cuboid(wheelbase / 2.0, chassis_height / 2.0)
        .density(chassis_mass / (wheelbase * chassis_height))
        .friction(0.3)
        .build();
    colliders.insert_with_parent(chassis_col, chassis_handle, &mut bodies);

    // ----------------------
    // Rear Wheel (driven)
    // ----------------------
    let rear_x = wheelbase / 4.0;
    let rear_y = wheel_radius + 0.5;
    
    let rear_rb = RigidBodyBuilder::dynamic()
        .translation(vector![rear_x, rear_y])
        .angular_damping(0.5)
        .can_sleep(false)
        .build();
    let rear_handle = bodies.insert(rear_rb);

    let wheel_density = wheel_mass / (std::f32::consts::PI * wheel_radius.powi(2));
    let rear_col = ColliderBuilder::ball(wheel_radius)
        .density(wheel_density)
        .friction(mu)
        .restitution(0.1)
        .build();
    colliders.insert_with_parent(rear_col, rear_handle, &mut bodies);

    // ----------------------
    // Front Wheel (free)
    // ----------------------
    let front_x = -wheelbase / 4.0;
    let front_y = wheel_radius + 0.5;
    
    let front_rb = RigidBodyBuilder::dynamic()
        .translation(vector![front_x, front_y])
        .angular_damping(0.5)
        .can_sleep(false)
        .build();
    let front_handle = bodies.insert(front_rb);

    let front_col = ColliderBuilder::ball(wheel_radius)
        .density(wheel_density)
        .friction(mu)
        .restitution(0.1)
        .build();
    colliders.insert_with_parent(front_col, front_handle, &mut bodies);

    // ----------------------
    // Simple revolute joints
    // ----------------------
    let rear_joint = RevoluteJointBuilder::new()
        .local_anchor1(vector![rear_x, rear_y - chassis_y].into())
        .local_anchor2(vector![0.0, 0.0].into())
        .build();
    impulse_joints.insert(chassis_handle, rear_handle, rear_joint, true);

    let front_joint = RevoluteJointBuilder::new()
        .local_anchor1(vector![front_x, front_y - chassis_y].into())
        .local_anchor2(vector![0.0, 0.0].into())
        .build();
    impulse_joints.insert(chassis_handle, front_handle, front_joint, true);

    // ----------------------
    // Data logging
    // ----------------------
    let mut log_file = File::create("simulation.csv").unwrap();
    writeln!(log_file, "time,v_chassis,omega_rear,omega_front,slip_rear,slip_front").unwrap();

    // ----------------------
    // Simulation loop
    // ----------------------
    let total_steps = 3_000;
    let dt = integration_params.dt;
    
    for step in 0..total_steps {
        let time = step as f32 * dt;

        // Read state
        let v_chassis = bodies.get(chassis_handle).unwrap().linvel().x;
        let omega_rear = bodies.get(rear_handle).unwrap().angvel();
        let omega_front = bodies.get(front_handle).unwrap().angvel();

        // Safety check
        if !v_chassis.is_finite() || !omega_rear.is_finite() || !omega_front.is_finite() {
            println!("NaN detected at t={:.2}s, stopping simulation", time);
            break;
        }

        // Velocity caps for safety
        {
            let mut chassis_rb = bodies.get_mut(chassis_handle).unwrap();
            let vel = chassis_rb.linvel();
            if vel.x.abs() > 50.0 {
                chassis_rb.set_linvel(vector![vel.x.signum() * 50.0, vel.y], true);
            }
        }

        {
            let mut rear_rb = bodies.get_mut(rear_handle).unwrap();
            if rear_rb.angvel().abs() > 200.0 {
                rear_rb.set_angvel(rear_rb.angvel().signum() * 200.0, true);
            }
        }

        // Calculate slip ratios
        let v_ref = v_chassis.abs().max(0.1);
        let slip_rear = (omega_rear * wheel_radius - v_chassis) / v_ref;
        let slip_front = (omega_front * wheel_radius - v_chassis) / v_ref;

        // Apply drag to chassis
        let drag = -rolling_resistance * v_chassis.signum() 
                   - aero_drag * v_chassis * v_chassis.abs();
        {
            let mut chassis_rb = bodies.get_mut(chassis_handle).unwrap();
            chassis_rb.add_force(vector![drag, 0.0], true);
        }

        // Apply motor torque to rear wheel only
        {
            let mut rear_rb = bodies.get_mut(rear_handle).unwrap();
            rear_rb.add_torque(motor_torque, true);
        }

        // Log data
        if step % 5 == 0 {
            writeln!(
                log_file,
                "{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
                time, v_chassis, omega_rear, omega_front, slip_rear, slip_front
            ).unwrap();
        }

        // Console output
        if step % 60 == 0 {
            let ideal_omega = v_chassis / wheel_radius;
            println!(
                "t={:.1}s | v={:.2} m/s | slip_r={:.3} | slip_f={:.3} | ω_r={:.1} | ω_f={:.1} | ideal={:.1}",
                time, v_chassis, slip_rear, slip_front, omega_rear, omega_front, ideal_omega
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

    println!("\nSimulation finished. Data saved to simulation.csv");
}