use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .add_systems(Startup, startup)
        .add_systems(Update, (apply_velocity, apply_speed, move_player, collide))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct Speed(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Anchored;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Player,
        Collider,
        Speed::default(),
        Velocity::default(),
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
            material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                scale: Vec3::new(10.0, 10.0, 10.0),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    let cube_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let cube_color = materials.add(Color::srgb(0.3, 0.3, 0.3));

    // spawn randomly placed rectangles to collide with
    for _ in 0..20 {
        commands.spawn((
            Collider,
            Anchored,
            MaterialMesh2dBundle {
                mesh: cube_mesh.clone().into(),
                material: cube_color.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        (rand::random::<f32>() - 0.5) * 800.0,
                        (rand::random::<f32>() - 0.5) * 600.0,
                        0.0,
                    ),
                    scale: Vec3::new(50.0, 50.0, 1.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
    }
}

fn apply_speed(time: Res<Time>, mut query: Query<(&mut Transform, &Speed)>) {
    for (mut transform, speed) in query.iter_mut() {
        transform.translation.x += speed.0.x * time.delta_seconds();
        transform.translation.y += speed.0.y * time.delta_seconds();
    }
}

fn apply_velocity(mut query: Query<(&mut Speed, &Velocity)>) {
    const FRICTION_COEFF: f32 = 0.85;
    const MAX_SPEED: f32 = 500.0;
    for (mut speed, velocity) in query.iter_mut() {
        speed.0 = (speed.0 + velocity.0).clamp(Vec2::splat(-MAX_SPEED), Vec2::splat(MAX_SPEED));
        speed.0 *= FRICTION_COEFF;
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    const ACCEL_AMOUNT: f32 = 50.0;
    const ACCEL_AMOUNT_SLOW: f32 = 10.0;
    let mut velocity = query.single_mut();

    let mut direction_x = 0.0;
    let mut direction_y = 0.0;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction_y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction_y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction_x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction_x += 1.0;
    }

    let accel_amount = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        ACCEL_AMOUNT_SLOW
    } else {
        ACCEL_AMOUNT
    };

    velocity.0 = Vec2::new(direction_x, direction_y).clamp_length_max(1.0) * accel_amount;
}

fn collide(
    mut query: Query<(&mut Transform, Option<&Anchored>, Option<&mut Speed>), With<Collider>>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(mut a, a_anchored, a_speed), (mut b, b_anchored, b_speed)]) =
        combinations.fetch_next()
    {
        let a_bounding_box = Aabb2d::new(a.translation.truncate(), a.scale.truncate() / 2.);
        let b_bounding_box = Aabb2d::new(b.translation.truncate(), b.scale.truncate() / 2.);

        if !a_bounding_box.intersects(&b_bounding_box) {
            // no collision
            continue;
        }

        if a_anchored.is_some() && b_anchored.is_some() {
            // both are anchored, ignore collision
            continue;
        }

        // calculate the offset to move the colliders apart
        let a_closest = a_bounding_box.closest_point(b_bounding_box.center());
        let b_closest = b_bounding_box.closest_point(a_bounding_box.center());

        let offset = a_closest - b_closest; // this is naive, because it always moves to the edge of the bounding box

        // move the colliders apart
        if a_anchored.is_none() && b_anchored.is_none() {
            let offset = offset / 2.0;
            a.translation -= offset.extend(0.0);
            b.translation += offset.extend(0.0);

            if a_speed.is_some() && b_speed.is_some() {
                let mut a_speed = a_speed.unwrap();
                let mut b_speed = b_speed.unwrap();

                let relative_speed = a_speed.0 - b_speed.0;
                let normal = offset.normalize();
                let impulse = relative_speed.dot(normal) * normal;
                a_speed.0 -= impulse;
                b_speed.0 += impulse;
            }
        } else if a_anchored.is_some() && b_anchored.is_none() {
            b.translation += offset.extend(0.0);

            if let Some(mut b_speed) = b_speed {
                let normal = offset.normalize();
                let impulse = b_speed.0.dot(normal) * normal;
                b_speed.0 -= impulse;
            }
        } else if a_anchored.is_none() && b_anchored.is_some() {
            a.translation -= offset.extend(0.0);

            if let Some(mut a_speed) = a_speed {
                let normal = offset.normalize();
                let impulse = a_speed.0.dot(normal) * normal;
                a_speed.0 -= impulse;
            }
        }
    }
}
