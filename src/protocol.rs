use std::ops::{Add, Mul};

use bevy::{ecs::entity::MapEntities, prelude::*};
use client::ComponentSyncMode;
use lightyear::prelude::*;

#[derive(Bundle)]
pub struct PlayerBundle {
    id: PlayerId,
    position: PlayerPosition,
    name: PlayerName,
}

impl PlayerBundle {
    pub fn new(id: ClientId, transform: Transform, name: String) -> Self {
        Self {
            id: PlayerId(id),
            position: PlayerPosition {
                translation: transform.translation,
                rotation: transform.rotation,
            },
            name: PlayerName(name),
        }
    }
}

// Components

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(ClientId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerPosition {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl Add for PlayerPosition {
    type Output = PlayerPosition;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        PlayerPosition {
            translation: self.translation + rhs.translation,
            rotation: self.rotation + rhs.rotation,
        }
    }
}

impl Mul<f32> for &PlayerPosition {
    type Output = PlayerPosition;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        PlayerPosition {
            translation: self.translation * rhs,
            rotation: self.rotation * rhs,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct PlayerName(pub String);

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerParent(Entity);

impl MapEntities for PlayerParent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

// Channels

// #[derive(Channel)]
// pub struct Channel1;

// Messages

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub struct Message1(pub usize);

// Inputs

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct PlayerInputs {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub crouch: bool,
}

// Protocol

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // messages
        // app.register_message::<Message1>(ChannelDirection::Bidirectional);

        // inputs
        app.add_plugins(InputPlugin::<PlayerInputs>::default());

        // components
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<PlayerPosition>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<PlayerName>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        // channels
        // app.add_channel::<Channel1>(ChannelSettings {
        //     mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        //     ..Default::default()
        // });
    }
}
