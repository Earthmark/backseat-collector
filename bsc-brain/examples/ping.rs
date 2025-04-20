use bsc_brain::{BrainApi, Drone, Main, main};

#[derive(Default)]
struct PingMain;

impl Main for PingMain {
    fn update(&mut self, api: &mut impl BrainApi) {
        for drone in api.drones() {
            println!("Status is: {:?}", drone.status());
        }
    }
}

main!(PingMain);
