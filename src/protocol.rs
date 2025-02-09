use avian3d::prelude::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use client::LerpFn as _;
use leafwing_input_manager::prelude::*;
use lightyear::client::components::ComponentSyncMode;
use lightyear::prelude::client::LeafwingInputConfig;
use lightyear::prelude::*;
use lightyear::utils::bevy::TransformLinearInterpolation;

// Components

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct PlayerId(pub ClientId);

#[derive(
    Component, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Deref, DerefMut, Reflect,
)]
#[reflect(Component)]
pub struct PlayerName(pub String);

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerParent(Entity);

impl MapEntities for PlayerParent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

// Channels

#[derive(Channel)]
pub struct Channel1;

// Messages

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub struct Message1(pub usize);

// Inputs

#[derive(Actionlike, Deserialize, Serialize, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerActions {
    #[actionlike(DualAxis)]
    Run,
    #[actionlike(DualAxis)]
    Look,
    Walk,
    Jump,
    Crouch,
}

impl PlayerActions {
    pub fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert_dual_axis(Self::Run, VirtualDPad::wasd());
        input_map.insert_dual_axis(Self::Look, MouseMove::default());
        input_map.insert(Self::Walk, KeyCode::ShiftLeft);
        input_map.insert(Self::Jump, KeyCode::Space);
        input_map.insert(Self::Crouch, KeyCode::ControlLeft);

        // TODO: gamepad map

        input_map
    }
}

// Protocol

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // types
        app.register_type::<PlayerId>()
            .register_type::<PlayerName>()
            .register_type::<PlayerParent>()
            .register_type::<PlayerActions>();

        // messages
        // app.register_message::<Message1>(ChannelDirection::Bidirectional);

        // inputs
        app.add_plugins(LeafwingInputPlugin::<PlayerActions> {
            config: LeafwingInputConfig::<PlayerActions> {
                lag_compensation: true,
                ..Default::default()
            },
        });

        // components
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<Transform>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_interpolation_fn(TransformLinearInterpolation::lerp);

        app.register_component::<RigidBody>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<PlayerName>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        // channels
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..Default::default()
        });
    }
}
