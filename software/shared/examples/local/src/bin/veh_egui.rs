use bevy::{diagnostic::FrameCount, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_physics)
        .add_systems(Update, (control_vehicle,camera_controls, ui_system))
        .run();
}

#[derive(Component)]
struct WheelMotor {
    joint_entity: Entity,
}

#[derive(Resource)]
struct VehicleControl {
    motor_joints: Vec<Entity>,
    drive_strength: f32,
}

fn setup_physics(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);
    
    // Ground - using heightfield for 2D terrain
    let ground_size = 60.0;
    let nsubdivs = 100;

    let heights: Vec<f32> = (0..=nsubdivs)
        .map(|i| -(i as f32 * ground_size / (nsubdivs as f32) / 2.0).cos())
        .collect();

    commands.spawn((
        Collider::heightfield(heights, Vec2::new(ground_size, 1.0)),
        Transform::from_xyz(-7.0, 0.0, 0.0),
        Friction::coefficient(1.0),
    ));

    // Vehicle setup
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
            CollisionGroups::new(CAR_GROUP, Group::all().difference(CAR_GROUP)),
            Transform::from_xyz(body_position.x, body_position.y, 0.0),
        ))
        .id();

    let mut motor_joints = vec![];

    for wheel_pos_in_car_space in wheel_params.into_iter() {
        let wheel_center = car_position + wheel_pos_in_car_space;

        // Axle
        let axle_entity = commands
            .spawn((
                RigidBody::Dynamic,
                AdditionalMassProperties::Mass(100.0),
                Transform::from_xyz(wheel_center.x, wheel_center.y, 0.0),
            ))
            .id();

        // Wheel
        let wheel_entity = commands
            .spawn((
                RigidBody::Dynamic,
                Collider::ball(wheel_radius),
                ColliderMassProperties::Density(200.0),
                CollisionGroups::new(CAR_GROUP, Group::all().difference(CAR_GROUP)),
                Friction::coefficient(1.0),
                Transform::from_xyz(wheel_center.x, wheel_center.y, 0.0),
            ))
            .with_children(|parent| {
                // Visual wheel outline (sensor)
                parent.spawn((
                    Collider::ball(wheel_radius),
                    Sensor,
                    ColliderMassProperties::Density(0.0),
                    CollisionGroups::new(Group::NONE, Group::NONE),
                ));
            })
            .id();

        let suspension_attachment = wheel_pos_in_car_space - body_position_in_car_space;

        // Suspension joint between body and axle
        let suspension_joint = PrismaticJointBuilder::new(Vec2::Y)
            .limits([0.0, suspension_height])
            .motor_position(0.0, 1.0e4, 1.0e3)
            .local_anchor1(Vec2::new(suspension_attachment.x, suspension_attachment.y));

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
    vehicle: Res<VehicleControl>,
    mut joints: Query<&mut ImpulseJoint>,
) {
    let mut thrust = 0.0;
    let mut boost = 1.0;

    if keyboard.pressed(KeyCode::ArrowUp) {
        thrust = -vehicle.drive_strength;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        thrust = vehicle.drive_strength;
    }
    if keyboard.pressed(KeyCode::ShiftRight) {
        boost = 1.5;
    }

    let target_velocity = 30.0 * thrust * boost;

    // Apply thrust to both wheels
    for joint_entity in &vehicle.motor_joints {
        if let Ok(mut joint) = joints.get_mut(*joint_entity) {
            if let TypedJoint::RevoluteJoint(mut revolute) = joint.data {
                revolute.set_motor_velocity(target_velocity, 1.0e2);
            }
        }
    }
}


fn camera_controls(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut scroll_events: MessageReader<bevy::input::mouse::MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut camera_transform = camera_query.single_mut().unwrap();
    
    // Zoom with mouse wheel
    for event in scroll_events.read() {
        let zoom_delta = event.y * 0.1;
        camera_transform.scale *= 1.0 - zoom_delta;
        // Clamp zoom level
        camera_transform.scale.x = camera_transform.scale.x.clamp(0.05, 2.0);
        camera_transform.scale.y = camera_transform.scale.y.clamp(0.05, 2.0);
    }
    
    // Pan with WASD or arrow keys (when not controlling vehicle)
    let pan_speed = 500.0 * camera_transform.scale.x * time.delta_secs();
    
    if keyboard.pressed(KeyCode::KeyW) {
        camera_transform.translation.y += pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        camera_transform.translation.y -= pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        camera_transform.translation.x -= pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        camera_transform.translation.x += pan_speed;
    }
    
    // Zoom with +/- keys
    if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
        camera_transform.scale *= 1.0 - 2.0 * time.delta_secs();
        camera_transform.scale.x = camera_transform.scale.x.max(0.05);
        camera_transform.scale.y = camera_transform.scale.y.max(0.05);
    }
    if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        camera_transform.scale *= 1.0 + 2.0 * time.delta_secs();
        camera_transform.scale.x = camera_transform.scale.x.min(2.0);
        camera_transform.scale.y = camera_transform.scale.y.min(2.0);
    }
    
    // Reset camera with R
    if keyboard.just_pressed(KeyCode::KeyR) {
        camera_transform.translation = Vec3::new(10.0, 10.0, 0.0);
        camera_transform.scale = Vec3::splat(0.2);
    }
}


fn ui_system(mut contexts: EguiContexts, frame_count: Res<FrameCount>) {
    if frame_count.0 == 0 {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Vehicle Controls")
        .default_pos([10.0, 10.0])
        .show(ctx, |ui| {
            ui.heading("2D Vehicle Simulator");
            ui.separator();
            ui.label("Controls:");
            ui.label("↑ Arrow Up - Move Forward");
            ui.label("↓ Arrow Down - Move Backward");
            ui.label("Right Shift - Boost");
            ui.separator();
            ui.label("Physics: Rapier2D");
            ui.label("Suspension with motor-driven wheels");
        });
    }

}