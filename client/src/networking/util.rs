use super::{networking::Runtime, networking_events::NetworkingEvent};
use async_channel::Receiver;
use bevy::prelude::*;
use protocol::protocol::{Error, Protocol};
use reqwest::{Client, Request, Response, StatusCode};
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
pub(crate) use wasm_bindgen_futures::spawn_local as spawn;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn get_task(
    runtime: &Runtime,
    client: &Client,
    request: Request,
) -> Receiver<NetworkingEvent> {
    let c = Arc::new(client.clone());
    let (s, r) = async_channel::unbounded();

    runtime.0.spawn(async move {
        debug!("Executing request {:?} with client {:?}", request, c);
        let url = request.url().clone();
        let respones = c.execute(request).await;
        let protocol = match respones {
            Ok(response) => decode_response(response, url.path()).await,
            Err(err) => Error::new_protocol(
                err.status().unwrap_or(StatusCode::IM_A_TEAPOT).into(),
                err.to_string(),
            ),
        };
        if let Err(err) = s.send(NetworkingEvent(protocol)).await {
            error!("Failed sending networking event {:?}", err);
        }
    });

    r
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn get_task(
    runtime: &Runtime,
    client: &Client,
    request: Request,
) -> Receiver<NetworkingEvent> {
    let c = Arc::new(client.clone());
    let (s, r) = async_channel::unbounded();

    spawn(async move {
        debug!("Executing request {:?} with client {:?}", request, c);
        let url = request.url().clone();
        let respones = c.execute(request).await;
        let protocol = match respones {
            Ok(response) => decode_response(response, url.path()).await,
            Err(err) => Error::new_protocol(
                err.status().unwrap_or(StatusCode::IM_A_TEAPOT).into(),
                err.to_string(),
            ),
        };
        if let Err(err) = s.send(NetworkingEvent(protocol)).await {
            error!("Failed sending networking event {:?}", err);
        }
    });

    r
}

async fn decode_response(res: Response, path: &str) -> Protocol {
    debug!("Decoding response {:?}", res);

    match res.status() {
        StatusCode::NO_CONTENT => Protocol::EMPTY(path.to_string()),
        StatusCode::UNAUTHORIZED => {
            Error::new_protocol(res.status().into(), "Unauthorized".to_string())
        }
        _ => match res.json().await {
            Ok(protocol) => protocol,
            Err(err) => Error::new_protocol(
                err.status().unwrap_or(StatusCode::IM_A_TEAPOT).into(),
                err.to_string(),
            ),
        },
    }
}
