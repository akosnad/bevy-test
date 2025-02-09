use std::{
    collections::HashMap,
    f32::consts::TAU,
    net::{Ipv4Addr, SocketAddr},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use blenvy::*;
use lightyear::prelude::{server::*, *};

use crate::{protocol::*, shared::shared_config};

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(AssetPlugin::default()),
            NetServer { listen_port: 9393 },
            RapierDebugRenderPlugin {
                default_collider_debug: ColliderDebug::AlwaysRender,
                enabled: true,
                ..Default::default()
            },
            // FpsControllerPlugin,
        ));
        app.add_systems(Startup, setup);
        // app.add_systems(Update, handle_connections);
        app.add_systems(
            FixedUpdate,
            (handle_connections, handle_disconnects, movement),
        );

        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 10000.0,
        });
        app.insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)));
    }
}

fn setup(mut commands: Commands) {
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
            fov: TAU / 5.0,
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

    commands.start_server();

    log::info!("Hello from server!");
}

fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    mut global: ResMut<Global>,
    mut commands: Commands,
) {
    for connection in connections.read() {
        let client_id = connection.client_id;

        let replicate = Replicate::default();
        let entity = commands.spawn((
            PlayerBundle::new(client_id, Transform::default(), "unknown".to_string()),
            replicate,
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
            GravityScale(1.0),
            Ccd { enabled: true },
            Transform::from_xyz(0.0, 12., 0.0),
        ));

        global.client_id_to_entity_id.insert(client_id, entity.id());

        log::info!("spawned entity for new client");
    }
}

fn handle_disconnects(
    mut disconnects: EventReader<DisconnectEvent>,
    mut global: ResMut<Global>,
    mut commands: Commands,
) {
    for disconnect in disconnects.read() {
        let client_id = disconnect.client_id;

        if let Some(entity) = global.client_id_to_entity_id.remove(&client_id) {
            let e = commands.entity(entity);
            e.despawn_recursive();
        }
    }
}

fn shared_movement_behaviour(mut position: Mut<PlayerPosition>, _input: &PlayerInputs) {
    // TODO
    position.translation.x += 2.0;
}

fn movement(
    mut position_query: Query<&mut PlayerPosition>,
    mut input_reader: EventReader<InputEvent<PlayerInputs>>,
    global: Res<Global>,
) {
    for input in input_reader.read() {
        let client_id = input.from();
        if let Some(input) = input.input() {
            if let Some(player_entity) = global.client_id_to_entity_id.get(&client_id) {
                if let Ok(position) = position_query.get_mut(*player_entity) {
                    shared_movement_behaviour(position, input);
                }
            }
        }
    }
}

fn net_configs(listen_port: u16) -> Vec<NetConfig> {
    let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), listen_port);

    vec![NetConfig::Netcode {
        config: NetcodeConfig {
            protocol_id: 0,
            private_key: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            ..Default::default()
        },
        io: IoConfig::from_transport(ServerTransport::UdpSocket(server_addr)),
    }]
}

struct NetServer {
    pub listen_port: u16,
}

impl Plugin for NetServer {
    fn build(&self, app: &mut App) {
        let server_config = ServerConfig {
            shared: shared_config(Mode::Separate),
            net: net_configs(self.listen_port),
            ..Default::default()
        };
        let server_plugins = ServerPlugins::new(server_config);
        app.add_plugins(server_plugins);

        app.insert_resource(Global {
            client_id_to_entity_id: HashMap::new(),
        });
    }
}

#[derive(Resource)]
struct Global {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}
