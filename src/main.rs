#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::f32::consts::TAU;

use bevy::gltf::{GltfMesh, GltfNode};
use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::window::CursorGrabMode;
use bevy_fps_controller::controller::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use iyes_perf_ui::prelude::*;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 10000.0,
        })
        .insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)))
        .insert_resource(DebugMode(false))
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin::default())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::Immediate,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            FpsControllerPlugin,
            // diagnostic
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            PerfUiPlugin,
            WorldInspectorPlugin::new().run_if(|debug_mode: Res<DebugMode>| debug_mode.0),
            RapierDebugRenderPlugin {
                enabled: false,
                ..Default::default()
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                cursor_grab_sys,
                load_level,
                toggle_noclip,
                respawn,
                change_level,
                toggle_debug,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    ass: Res<AssetServer>,
) {
    commands.spawn(PerfUiDefaultEntries::default());

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let logical_entity = commands
        .spawn((
            Collider::capsule_y(1.0, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true },
            Transform::from_xyz(0.0, 12., 0.0),
            LogicalPlayer,
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..Default::default()
            },
            FpsController {
                air_acceleration: 80.,
                ..Default::default()
            },
        ))
        .insert(CameraConfig {
            height_offset: -0.5,
        })
        .id();

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: TAU / 5.0,
            ..Default::default()
        }),
        Exposure::SUNLIGHT,
        RenderPlayer { logical_entity },
    ));
    // commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, -25., 0.)));

    // commands.spawn(SceneRoot(ass.load("scenes.glb#Scene0")));
    let playground = ass.load("playground.glb");
    let small = ass.load("small.glb");

    commands.insert_resource(Scenes {
        scenes: vec![small, playground],
        current: 0,
        current_loaded: false,
    });
}

#[derive(Resource)]
struct Scenes {
    scenes: Vec<Handle<Gltf>>,
    current: usize,
    current_loaded: bool,
}

#[derive(Component)]
struct CurrentScene;

fn cursor_grab_sys(mut windows: Query<&mut Window>, key: Res<ButtonInput<KeyCode>>) {
    let mut window = windows.single_mut();
    if key.just_pressed(KeyCode::Escape) {
        if window.cursor_options.visible {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

fn load_level(
    mut commands: Commands,
    mut scenes: ResMut<Scenes>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
    mut player: Query<(&LogicalPlayer, &mut Transform, &mut Velocity)>,
) {
    if scenes.current_loaded {
        return;
    }

    let gltf = gltf_assets.get(&scenes.scenes[scenes.current]);

    if let Some(gltf) = gltf {
        let scene = gltf.scenes.first().unwrap().clone();
        commands.spawn((SceneRoot(scene), CurrentScene));

        let (_, mut player_transform, mut player_velocity) = player.get_single_mut().unwrap();

        for node in &gltf.nodes {
            let node = gltf_node_assets.get(node).unwrap();

            if node.name == "spawnpoint" {
                player_transform.translation = node.transform.translation;
                player_transform.rotation = node.transform.rotation;
                player_velocity.linvel = Vec3::ZERO;
                player_velocity.angvel = Vec3::ZERO;
            }

            if let Some(gltf_mesh) = node.mesh.clone() {
                let gltf_mesh = gltf_mesh_assets.get(&gltf_mesh).unwrap();
                for mesh_primitive in &gltf_mesh.primitives {
                    let mesh = mesh_assets.get(&mesh_primitive.mesh).unwrap();
                    commands.spawn((
                        Collider::from_bevy_mesh(
                            mesh,
                            &ComputedColliderShape::TriMesh(TriMeshFlags::all()),
                        )
                        .unwrap(),
                        RigidBody::Fixed,
                        node.transform,
                        CurrentScene,
                    ));
                }
            }
        }
        scenes.current_loaded = true;
    }
}

fn toggle_noclip(
    mut query: Query<(Entity, &mut FpsControllerInput)>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.pressed(KeyCode::AltLeft) {
        let (_, mut controller_input) = query.get_single_mut().unwrap();
        controller_input.fly = true;
    }
}

fn respawn(mut query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y > -50.0 {
            continue;
        }

        velocity.linvel = Vec3::ZERO;
        transform.translation = Vec3::new(0.0, 12.0, 0.0);
    }
}

fn change_level(
    mut scenes: ResMut<Scenes>,
    key: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<CurrentScene>>,
    mut commands: Commands,
) {
    if !scenes.current_loaded {
        return;
    }

    if key.just_pressed(KeyCode::KeyN) {
        scenes.current += 1;
        if scenes.current == scenes.scenes.len() {
            scenes.current = 0;
        }
        scenes.current_loaded = false;

        for e in query.iter() {
            commands.entity(e).despawn_recursive();
        }
    }
}

#[derive(Resource)]
struct DebugMode(bool);

fn toggle_debug(
    key: Res<ButtonInput<KeyCode>>,
    mut debug_mode: ResMut<DebugMode>,
    mut debug_rendering: ResMut<DebugRenderContext>,
) {
    if !key.just_pressed(KeyCode::KeyP) {
        return;
    }

    debug_mode.0 = !debug_mode.0;

    debug_rendering.enabled = debug_mode.0;
}
