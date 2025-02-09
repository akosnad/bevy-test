use bevy::prelude::*;
use bevy_fps_controller::controller::LogicalPlayer;
use bevy_rapier3d::prelude::*;
use blenvy::BlenvyPlugin;
use lightyear::prelude::*;
use std::time::Duration;

use crate::protocol::ProtocolPlugin;

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: Duration::default(),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        mode,
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Spawnpoint;

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

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColliderInitialShape>()
            .register_type::<ColliderInitialProperties>()
            .register_type::<Spawnpoint>()
            .add_plugins((
                ProtocolPlugin,
                RapierPhysicsPlugin::<NoUserData>::default(),
                BlenvyPlugin::default(),
            ))
            .add_systems(Update, (respawn, create_colliders));
    }
}
