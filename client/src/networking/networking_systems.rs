use crate::networking::util::get_task;
use async_channel::Receiver;
use bevy::prelude::*;
use std::sync::Arc;

use super::{
    networking::Runtime, networking_events::NetworkingEvent,
    networking_ressource::NetworkingRessource,
};

#[derive(Component, Debug)]
pub struct NetworkingReceiver(Receiver<NetworkingEvent>);

pub(crate) fn request_dispatcher(
    mut res: ResMut<NetworkingRessource>,
    mut commands: Commands,
    runtime: Res<Runtime>,
) {
    let client = Arc::new(res.client.clone());
    for request in res.requests.drain(..) {
        debug!("Spawning task for {:?}", request);
        let task = get_task(&runtime, &client, request);

        // Spawn new entity and add our new task as a component
        commands.spawn(NetworkingReceiver(task));
    }
}

pub(crate) fn request_poller(
    mut commands: Commands,
    mut ev: EventWriter<NetworkingEvent>,
    mut transform_tasks: Query<(Entity, &NetworkingReceiver)>,
) {
    for (entity, receiver) in transform_tasks.iter_mut() {
        if let Ok(event) = receiver.0.try_recv() {
            debug!("Sending networking event {:?}", event);
            commands.entity(entity).despawn_recursive();
            ev.send(event);
        } else if receiver.0.is_closed() {
            warn!("Removing entity {:?} with closed receiver. This could indicate networking requests failing.", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}
