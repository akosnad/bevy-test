use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
};

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{server::*, *};
use lightyear_avian::prelude::*;

use crate::{protocol::*, shared::*};

fn setup(mut commands: Commands) {
    commands.start_server();

    log::info!("Hello from server!");
}

fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    spawnpoint: Query<&Transform, With<Spawnpoint>>,
    mut global: ResMut<Global>,
    mut commands: Commands,
) {
    for connection in connections.read() {
        let client_id = connection.client_id;

        let transform = spawnpoint
            .get_single()
            .cloned()
            .unwrap_or(Transform::from_xyz(0., 12., 0.));

        let entity = commands.spawn((
            Replicate {
                sync: SyncTarget {
                    prediction: NetworkTarget::Single(client_id),
                    interpolation: NetworkTarget::AllExceptSingle(client_id),
                },
                controlled_by: ControlledBy {
                    target: NetworkTarget::Single(client_id),
                    ..Default::default()
                },
                group: ReplicationGroup::new_id(client_id.to_bits()),
                ..Default::default()
            },
            PlayerId(client_id),
            ActionState::<PlayerActions>::default(),
            PlayerName(client_id.to_string()),
            Collider::capsule(1.0, 1.0),
            Friction::new(0.0),
            Restitution::new(0.0),
            LinearVelocity::ZERO,
            RigidBody::Dynamic,
            SleepingDisabled,
            LockedAxes::ROTATION_LOCKED,
            Mass(1.0),
            GravityScale(1.0),
            SweptCcd::new_with_mode(SweepMode::Linear),
            transform,
        ));

        global.client_id_to_entity_id.insert(client_id, entity.id());

        log::info!("spawned entity for new client");
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

#[derive(Resource)]
struct Global {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let server_config = ServerConfig {
            shared: shared_config(Mode::Separate),
            net: net_configs(9393),
            ..Default::default()
        };
        let server_plugins = ServerPlugins::new(server_config);

        app.insert_resource(Global {
            client_id_to_entity_id: HashMap::new(),
        });

        app.add_plugins((
            DefaultPlugins.set(AssetPlugin::default()),
            server_plugins,
            LagCompensationPlugin,
        ));
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, handle_connections);
    }
}
