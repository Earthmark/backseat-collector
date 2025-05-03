use bevy::prelude::*;
use brain::{BrainEngine, BrainPlugin, BrainRunner, BrainStats};
use drone::{Drone, DroneBrainLink};

mod brain;
mod drone;

fn spawn_entities(mut c: Commands, engine: Res<BrainEngine>) {
    let wat = include_bytes!("../test_brain.wasm");
    let brain = c
        .spawn((
            BrainRunner::new(&engine, wat).expect("Failed to load test module."),
            BrainStats::default(),
        ))
        .id();

    c.spawn((
        Drone::default(),
        DroneBrainLink::new(brain),
        Transform::default(),
    ));
}

fn main() -> Result<()> {
    App::default()
        .add_plugins((DefaultPlugins, BrainPlugin))
        .add_systems(Startup, spawn_entities)
        .run();
    Ok(())
}
