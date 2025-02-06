#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::window::CursorGrabMode;
use bevy_fps_controller::controller::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use blenvy::*;
use iyes_perf_ui::prelude::*;

fn main() {
    App::new()
        .register_type::<ColliderInitialShape>()
        .register_type::<ColliderInitialProperties>()
        .register_type::<Spawnpoint>()
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
            BlenvyPlugin::default(),
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
                toggle_noclip,
                respawn,
                toggle_debug,
                create_colliders,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    // ass: Res<AssetServer>,
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
    // let playground = ass.load("level2.glb");
    // let small = ass.load("level1.glb");

    // commands.insert_resource(Scenes {
    //     scenes: vec![small, playground],
    //     current: 0,
    //     current_loaded: false,
    // });
    commands.spawn((
        BlueprintInfo::from_path("levels/playground.glb"),
        SpawnBlueprint,
        HideUntilReady,
        GameWorldTag,
    ));
}

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

fn toggle_noclip(
    mut query: Query<(Entity, &mut FpsControllerInput)>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.pressed(KeyCode::AltLeft) {
        let (_, mut controller_input) = query.get_single_mut().unwrap();
        controller_input.fly = true;
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Spawnpoint;

fn respawn(
    spawn: Query<&Transform, With<Spawnpoint>>,
    mut players: Query<(&mut Transform, &mut Velocity), (With<LogicalPlayer>, Without<Spawnpoint>)>,
) {
    let Ok(spawn_transform) = spawn.get_single() else {
        return;
    };

    for (mut transform, mut velocity) in &mut players {
        if transform.translation.y > -50.0 {
            continue;
        }

        velocity.linvel = Vec3::ZERO;
        transform.translation = spawn_transform.translation;
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

#[derive(Reflect)]
#[reflect(Default)]
struct ColliderCuboidShape {
    hx: f32,
    hy: f32,
    hz: f32,
}
impl Default for ColliderCuboidShape {
    fn default() -> Self {
        Self {
            hx: 1.0,
            hy: 1.0,
            hz: 1.0,
        }
    }
}

#[derive(Reflect, Default)]
enum ColliderInitialShape {
    #[default]
    ComputedTriMesh,
    Cuboid(ColliderCuboidShape),
    Ball(f32),
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[reflect(Default)]
struct ColliderInitialProperties {
    shape: ColliderInitialShape,
    mass: Option<f32>,
    fixed: bool,
    friction: f32,
    restitution: f32,
}
impl Default for ColliderInitialProperties {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            mass: None,
            fixed: true,
            friction: 0.7,
            restitution: 0.3,
        }
    }
}

fn create_colliders(
    mut query: Query<(Entity, &ColliderInitialProperties, &Mesh3d)>,
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
) {
    for (ent, props, mesh) in query.iter_mut() {
        let mut e = commands.entity(ent);

        if props.fixed {
            e.insert((RigidBody::Fixed, ActiveCollisionTypes::all()));
        } else {
            e.insert(RigidBody::Dynamic);
        };

        match props.mass {
            Some(mass) => e.insert(ColliderMassProperties::Mass(mass)),
            // fall back to computed
            None => e.insert(ColliderMassProperties::default()),
        };

        match props.shape {
            ColliderInitialShape::Ball(radius) => {
                e.insert(Collider::ball(radius));
            }
            ColliderInitialShape::Cuboid(ColliderCuboidShape { hx, hy, hz }) => {
                e.insert(Collider::cuboid(hx, hy, hz));
            }
            ColliderInitialShape::ComputedTriMesh => {
                let mesh = meshes.get(mesh).unwrap();
                if let Some(collider) = Collider::from_bevy_mesh(
                    mesh,
                    &ComputedColliderShape::TriMesh(TriMeshFlags::all()),
                ) {
                    e.insert(collider);
                } else {
                    log::error!("Failed to create trimesh collider for entity {:?}", ent);
                }
            }
        };

        e.insert((
            Friction {
                coefficient: props.friction,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: props.restitution,
                combine_rule: CoefficientCombineRule::Min,
            },
        ));

        e.remove::<ColliderInitialProperties>();
    }
}
