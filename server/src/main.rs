use std::any::TypeId;

use bevy_app::{prelude::*, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut app = App::new();
    app.add_plugins((ScheduleRunnerPlugin::default(), StardustPlugin));
    app.add_channel::<MsgChannel>(ChannelConfiguration {
        consistency: MessageConsistency::ReliableOrdered,
        priority: 0,
    });
    app.add_systems(Startup, setup);
    app.add_systems(Update, (send_words_system, read_words_system));

    app.run();
}

struct MsgChannel;

fn setup() {
    log::info!("Hello from server!");
}

fn send_words_system(
    channels: Channels,
    mut query: Query<(Entity, &mut PeerMessages<Outgoing>), With<Peer>>,
) {
    let channel = channels.id(TypeId::of::<MsgChannel>()).unwrap();

    for (ent, mut outgoing) in query.iter_mut() {
        const MESSAGE: Message = Message::from_static_str("Hello, World!");
        outgoing.push_one(ChannelMessage {
            channel,
            message: MESSAGE,
        });

        log::info!("Message sent to {ent:?}");
    }
}

fn read_words_system(
    channels: Channels,
    query: Query<(Entity, &PeerMessages<Incoming>), With<Peer>>,
) {
    let channel = channels.id(TypeId::of::<MsgChannel>()).unwrap();

    for (ent, incoming) in query.iter() {
        for message in incoming.iter_channel(channel) {
            let string = message.as_str().unwrap();
            log::info!("Received message from {ent:?}: {string:?}");
        }
    }
}
