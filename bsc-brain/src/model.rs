use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

#[derive(Debug)]
pub enum ApiError {
    HostError,
    ArgumentError,
    NotFound,
}

pub trait BrainApi {
    fn drones(&self) -> impl Iterator<Item = impl Drone>;
}

pub trait Drone {
    fn status(&self) -> Result<DroneStatus, ApiError>;
}

#[derive(Debug, Default, IntoBytes, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct DroneStatus {
    pub pos: [f32; 5],
}
