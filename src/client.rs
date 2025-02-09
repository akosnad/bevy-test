use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use lightyear::prelude::{client::*, *};

use crate::protocol::PlayerId;
use crate::shared::shared_config;

#[derive(Component)]
struct RenderPlayer {
    logical_entity: Entity,
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    // ass: Res<AssetServer>,
) {
    commands.connect_client();
}

fn add_player_input_map(
    trigger: Trigger<OnAdd, PlayerId>,
    mut commands: Commands,
    query: Query<(), With<Predicted>>,
) {
    if query.get(trigger.entity()).is_ok() {
        commands
            .entity(trigger.entity())
            .insert(crate::protocol::PlayerActions::default_input_map());
    }
}

fn handle_predicted_spawn(
    mut predicted: Query<(Entity, &Transform), Added<Predicted>>,
    mut camera: Query<Entity, With<Camera3d>>,
    mut commands: Commands,
) {
    let camera_entity = camera.single_mut();
    for (player_entity, transform) in predicted.iter_mut() {
        commands.entity(camera_entity).insert((
            transform.clone(),
            RenderPlayer {
                logical_entity: player_entity,
            },
        ));

        log::info!(
            "attached camera {:?} to player {:?}",
            camera_entity,
            player_entity
        );
    }
}

fn render_player(
    mut render_query: Query<(&mut Transform, &RenderPlayer), Without<Predicted>>,
    player_query: Query<&Transform, With<Predicted>>,
) {
    for (mut render_transform, render_player) in render_query.iter_mut() {
        if let Ok(logical_transform) = player_query.get(render_player.logical_entity) {
            render_transform.translation = logical_transform.translation;
            render_transform.rotation = logical_transform.rotation;
        }
    }
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

fn net_config(addr: SocketAddr) -> NetConfig {
    use rand::prelude::*;
    let random_id = rand::rng().random::<u64>();

    NetConfig::Netcode {
        auth: Authentication::Manual {
            server_addr: addr,
            client_id: random_id,
            private_key: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            protocol_id: 0,
        },
        config: NetcodeConfig {
            token_expire_secs: -1,
            ..Default::default()
        },
        io: Default::default(),
    }
}

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let client_config = ClientConfig {
            shared: shared_config(Mode::Separate),
            net: net_config(std::net::SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 0, 1),
                9393,
            ))),
            ..Default::default()
        };
        let client_plugins = ClientPlugins::new(client_config);

        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin::default())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::Immediate,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            client_plugins,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (cursor_grab_sys, handle_predicted_spawn, render_player),
        );
        app.add_observer(add_player_input_map);
    }
}
