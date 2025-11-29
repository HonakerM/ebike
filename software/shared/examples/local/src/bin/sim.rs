use nalgebra::{vector, Point2, Vector2};
use rapier2d::prelude::*;

fn main() {
    // --- Build physics world ---
    let gravity = vector![0.0, -9.81];
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut islands = IslandManager::new();
    let mut broad_phase = BroadPhaseBvh::new();
    let mut narrow_phase = NarrowPhase::new();

    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut impulse_joints = ImpulseJointSet::new();
    let mut joints = MultibodyJointSet::new();

    let integration_params = IntegrationParameters {
        dt: 1.0 / 120.0,
        ..IntegrationParameters::default()
    };
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    // --- Create ground plane ---
    let ground = RigidBodyBuilder::fixed().build();
    let ground_handle = bodies.insert(ground);

    let ground_collider = ColliderBuilder::cuboid(5.0, 0.2).build();
    colliders.insert_with_parent(ground_collider, ground_handle, &mut bodies);

    // --- Create wheel body ---
    let wheel = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 1.0])
        .linvel(vector![0.0, 0.0])
        .angvel(0.0)
        .build();
    let wheel_handle = bodies.insert(wheel);

    let wheel_collider = ColliderBuilder::ball(0.35)
        .friction(1.0) // baseline friction
        .build();
    colliders.insert_with_parent(wheel_collider, wheel_handle, &mut bodies);

    // ---------------------------------------------------------------------
    // Tunable parameters
    // ---------------------------------------------------------------------
    let wheel_radius = 0.35;
    let motor_torque = 30.0; // Nm constant torque motor
    let tire_stiffness = 5000.0; // N (Pacejka-like simplified stiffness)
    let slip_damping = 20.0;
    let max_tire_force = 400.0; // real bicycle tire ≈ 150–300 N

    // ---------------------------------------------------------------------
    // Main simulation loop
    // ---------------------------------------------------------------------
    for step in 0..3000 {
        // -----------------------------------------------------------------
        // Custom Tire Slip Model
        // -----------------------------------------------------------------
        {
            let (tire_force, slip, v_forward, wheel_vel, wheel_omega) = {
                let wheel_rb = bodies.get(wheel_handle).unwrap();
                let wheel_vel = wheel_rb.linvel();
                let wheel_omega = wheel_rb.angvel();

                // Forward velocity at wheel contact patch
                let v_forward = wheel_vel.x;

                // Ideal rolling velocity
                let v_ideal = wheel_omega * wheel_radius;

                // Slip ratio (simple)
                let slip = if v_forward.abs() < 0.5 {
                    (v_ideal - v_forward).signum() * 1.0  // clamp slip to ±1
                } else {
                    (v_ideal - v_forward) / v_forward
                };
                let slip = slip.clamp(-2.0, 2.0);  // or even (-1, +1)



                // Tire force = stiffness * slip
                let tire_force = -slip * tire_stiffness;

                let tire_force = (-slip * tire_stiffness).clamp(-max_tire_force, max_tire_force);


                (tire_force.clone(), slip.clone(), v_forward.clone(), wheel_vel.clone(), wheel_omega.clone())
            };

            { 
                let wheel_rb_mut = bodies.get_mut(wheel_handle).unwrap();

                // Apply forward friction force at wheel center
                wheel_rb_mut.add_force(vector![tire_force, 0.0], true);

                // Motor torque applied to wheel
                wheel_rb_mut.apply_torque_impulse(motor_torque, true);

                // Additional slip damping for stability
                wheel_rb_mut.add_force(vector![-wheel_vel.x * slip_damping, 0.0], true);

                if step % 120 == 0 {
                    println!(
                        "Step {} | slip={:.3} | v={:.2} m/s | omega={:.2} rad/s",
                        step,
                        slip,
                        v_forward,
                        wheel_omega
                    );
                }
            };
        }

        // -----------------------------------------------------------------
        // Run physics step
        // -----------------------------------------------------------------
        physics_pipeline.step(
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
