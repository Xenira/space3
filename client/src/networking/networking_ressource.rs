use std::{fmt::Debug, time::Duration};

use bevy::{
    log::debug,
    prelude::{FromWorld, Resource, World},
};
use reqwest::{header::HeaderMap, Client, ClientBuilder, Method, Request, RequestBuilder, Url};
use serde::Serialize;

#[derive(Resource)]
pub struct ServerUrl(pub Url);

impl From<&str> for ServerUrl {
    fn from(base_url: &str) -> Self {
        ServerUrl(Url::parse(base_url).expect("Base url is invalid"))
    }
}

#[derive(Resource)]
pub struct NetworkingRessource {
    pub client: Client,
    pub polling_client: Client,
    pub requests: Vec<Request>,
    pub base_url: Url,
    pub headers: HeaderMap,
}

impl FromWorld for NetworkingRessource {
    fn from_world(world: &mut World) -> Self {
        let base_url = world
            .get_resource::<ServerUrl>()
            .expect("Base path is missing. ServerUrl resource not found");

        NetworkingRessource::new(base_url.0.clone())
    }
}

impl NetworkingRessource {
    pub fn new(base_url: Url) -> NetworkingRessource {
        #[cfg(not(target_family = "wasm"))]
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        #[cfg(target_family = "wasm")]
        let client = ClientBuilder::new().build().unwrap();

        #[cfg(not(target_family = "wasm"))]
        let polling_client = ClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        #[cfg(target_family = "wasm")]
        let polling_client = ClientBuilder::new().build().unwrap();

        NetworkingRessource {
            client: client.try_into().expect("Failed to construct client"),
            polling_client: polling_client
                .try_into()
                .expect("Failed to construct polling client"),
            requests: vec![],
            base_url,
            headers: HeaderMap::new(),
        }
    }

    pub fn request(&mut self, method: Method, url: &str) {
        self.requests.push(
            self.get_request(method, self.base_url.join(url).unwrap().as_str())
                .build()
                .unwrap(),
        )
    }

    pub fn request_data<T: Serialize + Debug>(&mut self, method: Method, url: &str, data: &T) {
        debug!(
            "[NET] {} Request to \"{}\" with data {:?}",
            method, url, data
        );
        self.requests.push(
            self.get_request(method, self.base_url.join(url).unwrap().as_str())
                .json(data)
                .build()
                .expect("Failed to parse json request body"),
        )
    }

    pub fn get_request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client
            .request(method, self.base_url.join(url).unwrap().as_str())
            .headers(self.headers.clone())
    }
}
