pub use ::enum_iterator;
pub use ::protocol_types;
pub mod protocol;

pub mod gods {
    include!(concat!(env!("OUT_DIR"), "/gods.rs"));
}

pub mod characters {
    include!(concat!(env!("OUT_DIR"), "/characters.rs"));
}
