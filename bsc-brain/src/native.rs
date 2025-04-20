use crate::{BrainApi, Drone, DroneStatus};

#[link(wasm_import_module = "bsc_brain")]
unsafe extern "C" {
    fn drone_count() -> i32;
    fn drone_id(index: i32) -> i32;
    fn drone_ping(drone: i32) -> i32;
    fn drone_status(drone: i32, status: &mut DroneStatus) -> i32;
}

#[derive(Default)]
pub struct NativeApi;

impl NativeApi {
    pub fn new() -> Self {
        Self
    }
}

impl BrainApi for NativeApi {
    fn drones(&self) -> impl Iterator<Item = impl Drone> {
        let max = unsafe { drone_count() };
        (0..max).map(|idx| NativeDrone {
            id: unsafe { drone_id(idx) },
        })
    }
}

struct NativeDrone {
    id: i32,
}

impl Drone for NativeDrone {
    fn ping(&self) -> i32 {
        unsafe { drone_ping(self.id) }
    }

    fn status(&self) -> DroneStatus {
        let mut status = DroneStatus::default();
        unsafe { drone_status(self.id, &mut status) };
        status
    }
}


