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
        dt: 1.0 / 120.0, // 120 Hz
        ..Default::default()
    };

    let physics_hooks = ();
    let event_handler = ();

    // ----------------------
    // Physical parameters
    // ----------------------
    // Wheel
    let wheel_radius: f32 = 0.35;
    let wheel_mass: f32 = 6.0; // kg (wheel + hub)
    // Chassis (bike + rider)
    let chassis_mass: f32 = 80.0; // kg
    // Weight distribution (fraction of total weight on this wheel)
    let weight_fraction_on_wheel: f32 = 0.5;

    // Motor/drivetrain
    let motor_torque_command: f32 = 80.0; // commanded torque (N·m)
    let motor_max_torque: f32 = 120.0; // clamp (N·m)
    let drivetrain_rotational_damping: f32 = 8.0; // N·m·s/rad

    // Tire model (saturating simple model)
    let tire_k: f32 = 2000.0;
    let tire_saturation_a: f32 = 1.0;

    // Friction / normal
    let mu: f32 = 0.9; // friction coefficient
    let g = 9.81_f32;
    let normal_force = (chassis_mass + wheel_mass) * g * weight_fraction_on_wheel; // N
    let tire_f_max = mu * normal_force; // traction cap (N)

    // Chassis drag / rolling resistance (for steady-state)
    let rolling_resistance: f32 = 12.0; // N constant-ish
    let aero_c1: f32 = 2.0; // linear drag coeff (N·s/m)
    let aero_c2: f32 = 0.25; // quadratic drag coeff (N/(m/s)^2)

    // Safety caps
    let max_omega: f32 = 2000.0; // rad/s safety cap
    let v_eps: f32 = 0.5; // minimum denom for slip computation

    // ----------------------
    // Ground
    // ----------------------
    let ground_rb = RigidBodyBuilder::fixed().translation(vector![0.0, 0.0]).build();
    let ground_handle = bodies.insert(ground_rb);
    let ground_col = ColliderBuilder::cuboid(20.0, 0.2)
        .friction(1.0)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(ground_col, ground_handle, &mut bodies);

    // ----------------------
    // Chassis (dynamic body)
    // ----------------------
    let chassis_start_y = 0.9; // above ground
    let chassis_rb = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, chassis_start_y])
        .build();
    let chassis_handle = bodies.insert(chassis_rb);

    // approximate chassis collider and density so mass ~ chassis_mass
    let chassis_hw = 0.6_f32;
    let chassis_hh = 0.2_f32;
    let chassis_area = chassis_hw * 2.0 * chassis_hh * 2.0 / 4.0; // rough 2D "area" heuristic
    // Use safe nonzero area; we only need density to approximate mass in 2D sim
    let chassis_density = (chassis_mass / (2.0 * chassis_hw * 2.0 * chassis_hh)).max(1.0);
    let chassis_col = ColliderBuilder::cuboid(chassis_hw, chassis_hh)
        .density(chassis_density)
        .friction(0.9)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(chassis_col, chassis_handle, &mut bodies);

    // ----------------------
    // Wheel (dynamic body)
    // ----------------------
    // Place wheel below chassis on the right (rear wheel)
    let wheel_start_pos = vector![0.4, 0.35];
    let wheel_rb = RigidBodyBuilder::dynamic()
        .translation(wheel_start_pos)
        .angvel(0.0)
        .build();
    let wheel_handle = bodies.insert(wheel_rb);

    // circle area = pi*r^2 ; density = mass / area (2D)
    let wheel_area = std::f32::consts::PI * wheel_radius.powi(2);
    let wheel_density = (wheel_mass / wheel_area).max(1.0);
    let wheel_col = ColliderBuilder::ball(wheel_radius)
        .density(wheel_density)
        .friction(mu)
        .restitution(0.0)
        .build();
    colliders.insert_with_parent(wheel_col, wheel_handle, &mut bodies);

    // ----------------------
    // Revolute joint (axle) between chassis and wheel
    // ----------------------
    // Joint anchor in world coordinates (wheel axle)
    let axle_world_pos = wheel_start_pos;
    let chassis_local_anchor = bodies[chassis_handle].position().inverse() * Isometry::new(axle_world_pos, 0.0);
    let wheel_local_anchor = bodies[wheel_handle].position().inverse() * Isometry::new(axle_world_pos, 0.0);

    // Use a free revolute joint (hinge)
    let joint = RevoluteJointBuilder::new()
        .local_anchor1(chassis_local_anchor.translation.vector.into())
        .local_anchor2(wheel_local_anchor.translation.vector.into())
        .build();
    joints.insert(chassis_handle, wheel_handle, joint, true);

    // ----------------------
    // Simulation loop
    // ----------------------
    let total_steps = 3_000;
    for step in 0..total_steps {
        // read state
        {
            // mutable access
            let (ch_linvel, wheel_angvel, wheel_linvel_x) = {
                let chassis_rb_ref = bodies.get(chassis_handle).unwrap();
                let wheel_rb_ref = bodies.get(wheel_handle).unwrap();
                (chassis_rb_ref.linvel().clone(), wheel_rb_ref.angvel().clone(), wheel_rb_ref.linvel().x.clone())
            };

            // chassis forward speed (use chassis x velocity as forward speed)
            let v_chassis = ch_linvel.x;

            // wheel angular speed
            let mut wheel_omega = wheel_angvel;

            // compute slip ratio safely
            let v_eff = v_chassis.abs().max(v_eps);
            let v_ideal = wheel_omega * wheel_radius;
            
            let slip = (v_ideal - v_chassis) / v_eff;

            // smooth saturating tire curve (driving direction: positive slip -> positive forward force)
            let f_raw = tire_k * slip / (1.0 + tire_saturation_a * slip.abs());
            let mut f_long = f_raw.clamp(-tire_f_max, tire_f_max);

            // compute rolling + aero drag on chassis (opposes forward motion)
            let drag = -v_chassis.signum() * rolling_resistance
                + -aero_c1 * v_chassis
                + -aero_c2 * v_chassis.abs() * v_chassis;

            // total longitudinal force to apply to chassis
            let f_total_chassis = (f_long + drag) as f32;

            // apply forces:
            // - apply driving/braking force to chassis in +x direction (reaction on chassis)
            // - apply opposite force to wheel (reaction on wheel)
            {
                {
                    // chassis force
                    let mut chassis_rb_mut = bodies.get_mut(chassis_handle).unwrap();
                    chassis_rb_mut.add_force(vector![f_total_chassis, 0.0], true);

                    
                    // optional: small linear damping applied manually to chassis (stabilize solver)
                    let linear_damping_coeff = 0.2;
                    chassis_rb_mut.add_force(-ch_linvel * linear_damping_coeff, true);
                }
                
                {
                    // wheel gets opposite reaction from ground contact (so wheel feels it too)
                    let mut wheel_rb_mut = bodies.get_mut(wheel_handle).unwrap();
                    wheel_rb_mut.add_force(vector![-f_long, 0.0], true);

                    // Motor torque: clamp command and subtract rotational damping
                    let motor_tau = motor_torque_command.clamp(-motor_max_torque, motor_max_torque);
                    let damping_tau = -drivetrain_rotational_damping * wheel_omega;
                    let net_tau = motor_tau + damping_tau;

                    // apply torque (Rapier expects a torque in N·m)
                    wheel_rb_mut.add_torque(net_tau, true);

                    // cap omega to avoid runaway numerical problems
                    if wheel_rb_mut.angvel().abs() > max_omega {
                        wheel_rb_mut.set_angvel(wheel_rb_mut.angvel().signum() * max_omega, true);
                    }

                    // update wheel_omega after possible cap
                    wheel_omega = wheel_rb_mut.angvel();
                }
            }

            if step % 120 == 0 {
                println!(
                    "Step {} | slip={:.3} | v={:.2} m/s | omega={:.2} rad/s | F_long={:.1} N",
                    step, slip, v_chassis, wheel_omega, f_long
                );
            }
        }

        // step physics
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
