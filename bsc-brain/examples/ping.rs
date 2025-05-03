use bsc_brain::{main, BrainApi, Drone, Main};

#[derive(Default)]
struct PingMain;

impl Main for PingMain {
    fn update(&mut self, api: &mut impl BrainApi) {
        for drone in api.drones() {
            _ = drone.status();
        }
    }
}

main!(PingMain);
