// pub trait

use std::sync::{Arc, Mutex};

use crate::model::polling::Channel;

pub struct DataManager<T> {
    pub data: Option<T>,
}

pub struct Synced<T> {
    pub data: Option<Arc<Mutex<T>>>,
    channel: Channel,
}

// impl<T> Synced<T> {
//     pub fn new(data: T, channel: Channel) -> Self {
//         Self {
//             data: Some(Arc::new(Mutex::new(data))),
//             channel,
//         }
//     }

//     pub fn get(&self) -> &T {
//         self.data.as_ref().unwrap().lock().unwrap()
//     }

//     pub fn with_mut(&mut self) -> &mut T {
//         self.data.as_mut().unwrap()
//     }
// }
