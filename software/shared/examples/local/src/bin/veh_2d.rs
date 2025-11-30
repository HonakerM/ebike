use nalgebra::Unit;
use rapier_testbed2d::{KeyCode, Testbed, TestbedApp};
use rapier2d::prelude::*;

pub fn init_world(testbed: &mut Testbed) {
    /*
     * World
     */
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut impulse_joints = ImpulseJointSet::new();
    let multibody_joints = MultibodyJointSet::new();

    /*
     * Ground - using heightfield for 2D terrain.
     */
    let ground_size = 60.0;
    let nsubdivs = 100;

    // Create a DVector for 2D heightfield (1D vector of heights)
    let heights = DVector::from_fn(nsubdivs + 1, |i, _| {
        -(i as f32 * ground_size / (nsubdivs as f32) / 2.0).cos()
    });

    let collider = ColliderBuilder::heightfield(heights, vector![ground_size, 1.0])
        .translation(vector![-7.0, 0.0])
        .friction(1.0);
    colliders.insert(collider);

    /*
     * Vehicle we will control manually, simulated using joints.
     * Simplified 2D version with two wheels.
     */
    const CAR_GROUP: Group = Group::GROUP_1;

    // Two wheel positions in 2D (x, y coordinates)
    let wheel_params = [
        vector![-0.5, 0.2783],  // Left wheel
        vector![0.5, 0.2783], // Right wheel (shifted in y)
    ];

    let suspension_height = 0.12;
    let drive_strength = 1.0;
    let wheel_radius = 0.28;
    let car_position = point![0.0, wheel_radius + suspension_height];
    let body_position_in_car_space = vector![0.0, 0.4739];

    let body_position = car_position + body_position_in_car_space;

    // Car body as a rectangle in 2D
    let body_co = ColliderBuilder::cuboid(0.9, 0.3)
        .mass(500.0)
        .collision_groups(InteractionGroups::new(
            CAR_GROUP,
            !CAR_GROUP,
            InteractionTestMode::And,
        ));
    let body_rb = RigidBodyBuilder::dynamic()
        .pose(Isometry::new(body_position.coords, 0.0))
        .build();
    let body_handle = bodies.insert(body_rb);
    colliders.insert_with_parent(body_co, body_handle, &mut bodies);

    let mut motor_joints = vec![];

    for wheel_pos_in_car_space in wheel_params.into_iter() {
        let wheel_center = car_position + wheel_pos_in_car_space;

        let axle_mass_props = MassProperties::from_ball(100.0, wheel_radius);
        let axle_rb = RigidBodyBuilder::dynamic()
            .pose(Isometry::new(wheel_center.coords, 0.0))
            .additional_mass_properties(axle_mass_props);
        let axle_handle = bodies.insert(axle_rb);

        // Visual circle for the wheel (sensor so it shows as wireframe)
        let wheel_fake_co = ColliderBuilder::ball(wheel_radius)
            .sensor(true)
            .density(0.0)
            .collision_groups(InteractionGroups::none());

        // Actual wheel collider
        let wheel_co = ColliderBuilder::ball(wheel_radius)
            .density(200.0)
            .collision_groups(InteractionGroups::new(
                CAR_GROUP,
                !CAR_GROUP,
                InteractionTestMode::And,
            ))
            .friction(1.0);
        let wheel_rb = RigidBodyBuilder::dynamic()
            .position(Isometry::new(wheel_center.coords, 0.0));
        let wheel_handle = bodies.insert(wheel_rb);
        colliders.insert_with_parent(wheel_co, wheel_handle, &mut bodies);
        colliders.insert_with_parent(wheel_fake_co, wheel_handle, &mut bodies);

        let suspension_attachment_in_body_space =
            wheel_pos_in_car_space - body_position_in_car_space;

        // Suspension between the body and the axle
        // In 2D: lock X translation and rotation
        let locked_axes = JointAxesMask::LIN_X | JointAxesMask::ANG_X;

        let suspension_joint = GenericJointBuilder::new(locked_axes)
            .limits(JointAxis::LinY, [0.0, suspension_height])
            .motor_position(JointAxis::LinY, 0.0, 1.0e4, 1.0e3)
            .local_anchor1(suspension_attachment_in_body_space.into());

        impulse_joints.insert(body_handle, axle_handle, suspension_joint, true);

        // Joint between the axle and the wheel - revolute joint for rotation
        let wheel_joint = RevoluteJointBuilder::new();
        let wheel_joint_handle =
            impulse_joints.insert(axle_handle, wheel_handle, wheel_joint, true);

        motor_joints.push(wheel_joint_handle);
    }

    /*
     * Callback to control the wheels motors (forward/backward only in 2D).
     */
    testbed.add_callback(move |gfx, physics, _, _| {
        let Some(gfx) = gfx else { return };

        let mut thrust = 0.0;
        let mut boost = 1.0;

        for key in gfx.keys().get_pressed() {
            match *key {
                KeyCode::ArrowUp => {
                    thrust = -drive_strength;
                }
                KeyCode::ArrowDown => {
                    thrust = drive_strength;
                }
                KeyCode::ShiftRight => {
                    boost = 1.5;
                }
                _ => {}
            }
        }

        let should_wake_up = thrust != 0.0;

        // Apply thrust to both wheels equally (rotating them)
        for motor_handle in &motor_joints {
            let motor_joint = physics
                .impulse_joints
                .get_mut(*motor_handle, should_wake_up)
                .unwrap();
            motor_joint.data.set_motor_velocity(
                JointAxis::AngX,
                30.0 * thrust * boost,
                1.0e2,
            );
        }
    });

    /*
     * Set up the testbed.
     */
    testbed.set_world(bodies, colliders, impulse_joints, multibody_joints);
    testbed.look_at(point![10.0, 10.0], 5.0);
}

fn main() {
    let builders: Vec<(_, fn(&mut Testbed))> = vec![
        ("Vehicle2D", init_world),
    ];
    let testbed = TestbedApp::from_builders(builders);
    testbed.run()
}