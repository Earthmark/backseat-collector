use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Drone {
    pub id: u32,
    pub pos: [f32; 5],
}

#[derive(Component)]
#[relationship(relationship_target = crate::brain::BrainDroneLinks)]
pub struct DroneBrainLink(Entity);

impl DroneBrainLink {
    pub fn new(brain: Entity) -> Self {
        Self(brain)
    }
}
