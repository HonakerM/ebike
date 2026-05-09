use bevy::{diagnostic::FrameCount, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier2d::prelude::*;
use nalgebra::Unit;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_world)
        .add_systems(Update, (control_vehicle, ui_system))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .run();
}

#[derive(Component)]
struct MotorJoint(Entity);

#[derive(Resource)]
struct VehicleControl {
    motor_joints: Vec<Entity>,
    drive_strength: f32,
}

fn setup_world(mut commands: Commands) {
    // Camera
    commands.spawn((Camera2d::default(), Transform::from_xyz(10.0, 10.0, 0.0).with_scale(Vec3::splat(0.2))));

    /*
     * Ground - using heightfield for 2D terrain.
     */
    let ground_size = 60.0;
    let nsubdivs = 100;

    // Create heights for heightfield
    let heights: Vec<f32> = (0..=nsubdivs)
        .map(|i| -(i as f32 * ground_size / (nsubdivs as f32) / 2.0).cos())
        .collect();

    commands.spawn((
        Collider::heightfield(heights, Vec2::new(ground_size, 1.0)),
        Transform::from_xyz(-7.0, 0.0, 0.0),
        Friction::coefficient(1.0),
    ));

    /*
     * Vehicle we will control manually, simulated using joints.
     */
    const CAR_GROUP: Group = Group::GROUP_1;

    let wheel_params = [
        Vec2::new(-0.5, 0.2783),  // Left wheel
        Vec2::new(0.5, 0.2783),   // Right wheel
    ];

    let suspension_height = 0.12;
    let drive_strength = 1.0;
    let wheel_radius = 0.28;
    let car_position = Vec2::new(0.0, wheel_radius + suspension_height);
    let body_position_in_car_space = Vec2::new(0.0, 0.4739);
    let body_position = car_position + body_position_in_car_space;

    // Car body
    let body_entity = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(0.9, 0.3),
            ColliderMassProperties::Mass(500.0),
            Transform::from_xyz(
                body_position.x,
                body_position.y,
                0.0,
            ),
            CollisionGroups::new(CAR_GROUP, Group::all().difference(CAR_GROUP)),
            Velocity::default(),
        ))
        .id();

    let mut motor_joints = vec![];

    for wheel_pos_in_car_space in wheel_params.into_iter() {
        let wheel_center = car_position + wheel_pos_in_car_space;

        // Axle (invisible mass point)
        let axle_entity = commands
            .spawn((
                RigidBody::Dynamic,
                Transform::from_xyz(
                    wheel_center.x,
                    wheel_center.y,
                    0.0,
                ),
                AdditionalMassProperties::Mass(100.0),
                Velocity::default(),
            ))
            .id();

        // Wheel
        let wheel_entity = commands
            .spawn((
                RigidBody::Dynamic,
                Collider::ball(wheel_radius),
                ColliderMassProperties::Density(200.0),
                Transform::from_xyz(
                    wheel_center.x,
                    wheel_center.y,
                    0.0,
                ),
                CollisionGroups::new(CAR_GROUP, Group::all().difference(CAR_GROUP)),
                Friction::coefficient(1.0),
                Velocity::default(),
            ))
            .id();

        let suspension_attachment_in_body_space =
            wheel_pos_in_car_space - body_position_in_car_space;

        // Suspension joint between body and axle
        let locked_axes = JointAxesMask::LIN_X | JointAxesMask::ANG_X;
        
        let suspension_joint = PrismaticJointBuilder::new(Vec2::Y)
        .limits([0.0, suspension_height])
        .motor_position(0.0, 1.0e4, 1.0e3)
        .local_anchor1(Vec2::new(suspension_attachment_in_body_space.x, suspension_attachment_in_body_space.y));


        commands.entity(body_entity).with_children(|parent| {
            parent.spawn(ImpulseJoint::new(axle_entity, suspension_joint));
        });

        // Revolute joint between axle and wheel
        let wheel_joint = RevoluteJointBuilder::new()
            .motor_velocity(0.0, 1.0e2).build();
        
        let joint_entity = commands
            .spawn(ImpulseJoint::new(wheel_entity, wheel_joint))
            .id();

        commands.entity(axle_entity).add_child(joint_entity);

        motor_joints.push(joint_entity);
    }

    // Insert vehicle control resource
    commands.insert_resource(VehicleControl {
        motor_joints,
        drive_strength,
    });
}

fn control_vehicle(
    keyboard: Res<ButtonInput<KeyCode>>,
    vehicle_control: Res<VehicleControl>,
    mut joints: Query<&mut ImpulseJoint>,
) {
    let mut thrust = 0.0;
    let mut boost = 1.0;

    if keyboard.pressed(KeyCode::ArrowUp) {
        thrust = -vehicle_control.drive_strength;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        thrust = vehicle_control.drive_strength;
    }
    if keyboard.pressed(KeyCode::ShiftRight) {
        boost = 1.5;
    }

    // Apply thrust to both wheels
    for motor_entity in &vehicle_control.motor_joints {
        if let Ok(mut joint) = joints.get_mut(*motor_entity) {
            if let TypedJoint::RevoluteJoint(mut revolute) = joint.data {
                revolute.set_motor_velocity(30.0 * thrust * boost, 1.0e2);
            }
        }
    }
}

fn ui_system(mut contexts: EguiContexts, frame_count: Res<FrameCount>) {
    if frame_count.0 == 0 {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut() {

    egui::Window::new("Controls").show(ctx, |ui| {
        ui.label("Arrow Up: Forward");
        ui.label("Arrow Down: Backward");
        ui.label("Right Shift: Boost");
    });
    }
}