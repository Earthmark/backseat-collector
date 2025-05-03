use crate::{internal::{DroneID, StatusCode}, ApiError, BrainApi, Drone, DroneStatus};

#[link(wasm_import_module = "bsc_brain")]
unsafe extern "C" {
    fn drone_count() -> u32;
    fn drone_id(index: u32, id: &mut NativeDrone) -> StatusCode;
    fn drone_status(drone: NativeDrone, status: &mut DroneStatus) -> StatusCode;
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
        (0..max).filter_map(|idx| {
            let mut id = NativeDrone::default();
            unsafe { drone_id(idx, &mut id) }
                .to_result()
                .ok()
                .map(|_| id)
        })
    }
}

#[derive(Default, Clone)]
#[repr(transparent)]
struct NativeDrone {
    id: DroneID,
}

impl Drone for NativeDrone {
    fn status(&self) -> Result<DroneStatus, ApiError> {
        let mut status = DroneStatus::default();
        unsafe { drone_status(self.clone(), &mut status) }.to_result()?;
        Ok(status)
    }
}
