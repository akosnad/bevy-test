use avian3d::prelude::*;
use bevy::prelude::*;
use blenvy::BlenvyPlugin;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::ReplicationTarget;
use lightyear::prelude::*;
use std::time::Duration;

use crate::protocol::{PlayerActions, PlayerId, ProtocolPlugin};

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: Duration::default(),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        mode,
    }
}

pub fn shared_player_movement(mut transform: Mut<Transform>, action: &ActionState<PlayerActions>) {
    const PLAYER_RUN_SPEED: f32 = 0.2;

    // TODO: handle panning camera
    // let Some(look_data) = action.dual_axis_data(&PlayerActions::Look) else {
    //     return;
    // };
    let Some(run_data) = action.dual_axis_data(&PlayerActions::Run) else {
        return;
    };

    // TODO
    transform.translation.x += PLAYER_RUN_SPEED * run_data.pair.x;
    transform.translation.z += PLAYER_RUN_SPEED * run_data.pair.y;
}

fn player_movement(
    mut player_query: Query<
        (&mut Transform, &ActionState<PlayerActions>),
        Or<(With<Predicted>, With<ReplicationTarget>)>,
    >,
) {
    for (transform, action_state) in player_query.iter_mut() {
        shared_player_movement(transform, action_state);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Spawnpoint;

fn respawn(
    spawn: Query<&Transform, With<Spawnpoint>>,
    mut players: Query<
        (&mut Transform, &mut LinearVelocity),
        (With<PlayerId>, Without<Spawnpoint>),
    >,
) {
    let Ok(spawn_transform) = spawn.get_single() else {
        return;
    };

    for (mut transform, mut velocity) in &mut players {
        if transform.translation.y > -50.0 {
            continue;
        }

        let mut zero_vel = LinearVelocity::ZERO;
        std::mem::swap(&mut zero_vel, &mut *velocity);
        transform.translation = spawn_transform.translation;
    }
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
            e.insert(RigidBody::Static);
        } else {
            e.insert(RigidBody::Dynamic);
        }

        if let Some(mass) = props.mass {
            e.insert(Mass(mass));
        }

        match props.shape {
            ColliderInitialShape::Cuboid(ColliderCuboidShape { hx, hy, hz }) => {
                e.insert(Collider::cuboid(hx * 2., hy * 2., hz * 2.));
            }
            ColliderInitialShape::Ball(r) => {
                e.insert(Collider::sphere(r));
            }
            ColliderInitialShape::ComputedTriMesh => {
                let mesh = meshes.get(mesh).unwrap();
                if let Some(collider) = Collider::trimesh_from_mesh(mesh) {
                    e.insert(collider);
                } else {
                    log::error!("Failed to create trimesh collider for entity {:?}", ent);
                }
            }
        };

        e.insert((
            Friction::new(props.friction),
            Restitution::new(props.restitution),
        ));

        e.remove::<ColliderInitialProperties>();
    }
}

fn setup(mut commands: Commands) {
    use blenvy::*;

    commands.spawn(iyes_perf_ui::prelude::PerfUiDefaultEntries::default());

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(12.0, 4.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::TAU / 5.0,
            ..Default::default()
        }),
        bevy::render::camera::Exposure::SUNLIGHT,
    ));

    commands.spawn((
        BlueprintInfo::from_path("levels/playground.glb"),
        SpawnBlueprint,
        HideUntilReady,
        GameWorldTag,
    ));
}

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColliderInitialShape>()
            .register_type::<ColliderInitialProperties>()
            .register_type::<ColliderCuboidShape>()
            .register_type::<Spawnpoint>();
        app.add_plugins((
            ProtocolPlugin,
            PhysicsPlugins::default(),
            BlenvyPlugin::default(),
            // diagnostic
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            iyes_perf_ui::PerfUiPlugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            PhysicsDebugPlugin::default(),
        ));

        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 10000.0,
        });
        app.insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)));

        app.add_systems(FixedUpdate, (respawn, create_colliders, player_movement));
        app.add_systems(Startup, setup);
    }
}
