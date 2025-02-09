#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::f32::consts::TAU;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::window::CursorGrabMode;
use bevy_fps_controller::controller::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use blenvy::*;
use iyes_perf_ui::prelude::*;
use lightyear::client::input::native::InputSystemSet;
use lightyear::prelude::{client::*, *};

use crate::protocol::*;
use crate::shared::shared_config;

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 10000.0,
        })
        .insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)))
        .insert_resource(DebugMode(false));
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
            NetClient {
                server_addr: std::net::SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    9393,
                )),
            },
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
                toggle_noclip,
                toggle_debug,
                character_spawned,
            ),
        );
    }
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

    commands.connect_client();
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

fn character_spawned(
    mut spawn_reader: EventReader<EntitySpawnEvent>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for spawn in spawn_reader.read() {
        let mut e = commands.entity(spawn.entity());

        e.insert((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                ..Default::default()
            })),
        ));
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

pub struct NetClient {
    pub server_addr: SocketAddr,
}

impl Plugin for NetClient {
    fn build(&self, app: &mut App) {
        let client_config = ClientConfig {
            shared: shared_config(Mode::Separate),
            net: net_config(self.server_addr),
            ..Default::default()
        };
        let client_plugins = ClientPlugins::new(client_config);

        app.add_plugins(client_plugins);

        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystemSet::BufferInputs),
        );
    }
}

fn buffer_input(
    tick_manager: Res<TickManager>,
    mut input_manager: ResMut<InputManager<PlayerInputs>>,
    keypress: Res<ButtonInput<KeyCode>>,
) {
    let tick = tick_manager.tick();
    let mut input = PlayerInputs::default();

    // TODO: don't hardcode keys
    // could use leafwing-input-manager
    if keypress.pressed(KeyCode::KeyW) {
        input.forward = true;
    }
    if keypress.pressed(KeyCode::KeyS) {
        input.backward = true;
    }
    if keypress.pressed(KeyCode::KeyA) {
        input.left = true;
    }
    if keypress.pressed(KeyCode::KeyD) {
        input.right = true;
    }
    if keypress.pressed(KeyCode::Space) {
        input.jump = true;
    }
    if keypress.pressed(KeyCode::ControlLeft) {
        input.crouch = true;
    }
    input_manager.add_input(input, tick);
}
