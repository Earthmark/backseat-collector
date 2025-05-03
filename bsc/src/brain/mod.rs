mod api;
mod runtime;

use std::sync::Mutex;

use bevy::prelude::*;
use runtime::{BrainCtx, BrainRuntime};

use crate::Drone;

pub use runtime::BrainEngine;

pub struct BrainPlugin;

impl Plugin for BrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BrainEngine>()
            .add_systems(Update, run_brains);
    }
}

#[derive(Component)]
#[relationship_target(relationship = crate::drone::DroneBrainLink)]
pub struct BrainDroneLinks(Vec<Entity>);

#[derive(Component)]
pub struct BrainRunner(Mutex<BrainRuntime>);

impl BrainRunner {
    pub fn new(engine: &BrainEngine, module: impl AsRef<[u8]>) -> Result<Self> {
        Ok(BrainRunner(Mutex::new(BrainRuntime::new(engine, module)?)))
    }
}

#[derive(Component, Default)]
pub struct BrainStats {
    _message: Option<String>,
    _wall_clock_time: std::time::Duration,
    _gas_consumed: u64,
}

fn run_brains(
    mut brains: Query<(&mut BrainRunner, &mut BrainStats, Option<&BrainDroneLinks>)>,
    drone_pos: Query<&Drone>,
) {
    brains
        .par_iter_mut()
        .for_each(|(mut brain, mut stats, links)| {
            let mut ctx = BrainCtx::new(&mut *stats, links, drone_pos);

            brain
                .0
                .get_mut()
                .expect("Failed to acquire brain mutex")
                .run(&mut ctx)
        });
}
