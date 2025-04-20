mod macros;
mod native;

pub type NativeApi = native::NativeApi;

pub trait Main {
    fn update(&mut self, api: &mut impl BrainApi);
}

pub trait FromApi {
    fn init(api: &mut impl BrainApi) -> Self;
}

impl<T: Default> FromApi for T {
    fn init(_api: &mut impl BrainApi) -> Self {
        T::default()
    }
}

pub trait BrainApi {
    fn drones(&self) -> impl Iterator<Item = impl Drone>;
}

pub trait Drone {
    fn ping(&self) -> i32;
    fn status(&self) -> DroneStatus;
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct DroneStatus {
    pub pos: [f32; 5],
}
