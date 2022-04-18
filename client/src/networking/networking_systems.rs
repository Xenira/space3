use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use protocol::protocol::{Error, Protocol};
use surf::{Client, Request, Response};

use super::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource};

pub(crate) fn request_dispatcher(
    mut res: ResMut<NetworkingRessource>,
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
) {
    let client = Arc::new(res.client.clone());
    for request in res.requests.drain(..) {
        debug!("Spawning task for {:?}", request);
        let task = get_task(&client, request, &thread_pool);

        // Spawn new entity and add our new task as a component
        commands.spawn().insert(task);
    }
}

fn get_task(
    client: &Client,
    request: Request,
    thread_pool: &Res<AsyncComputeTaskPool>,
) -> Task<NetworkingEvent> {
    let c = Arc::new(client.clone());
    thread_pool.spawn(async move {
        debug!("Executing request {:?} with client {:?}", request, c);
        NetworkingEvent(match c.send(request).await {
            Ok(mut response) => decode_response(&mut response).await,
            Err(err) => Error::new_protocol(err.status().into(), err.to_string()),
        })
    })
}

async fn decode_response(res: &mut Response) -> Protocol {
    debug!("Decoding response {:?}", res);
    match res.body_json().await {
        Ok(protocol) => protocol,
        Err(err) => Error::new_protocol(err.status().into(), err.to_string()),
    }
}

pub(crate) fn request_poller(
    mut commands: Commands,
    mut ev: EventWriter<NetworkingEvent>,
    mut transform_tasks: Query<(Entity, &mut Task<NetworkingEvent>)>,
) {
    for (entity, mut task) in transform_tasks.iter_mut() {
        if let Some(event) = future::block_on(future::poll_once(&mut *task)) {
            ev.send(event);
            commands.entity(entity).despawn_recursive();
        }
    }
}
