// Sources: vault/infected/infected_young.md, vault/infected/infected_decrepit.md, vault/infected/infected_aged.md

use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimPosition};
use crate::units::infected::{InfectedVariant, SpawnInfectedEvent};

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedYoung;

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedYoungEvent {
    pub position: SimPosition,
}

fn spawn_infected_young_system(
    mut events: EventReader<SpawnInfectedYoungEvent>,
    mut spawn_writer: EventWriter<SpawnInfectedEvent>,
) {
    for ev in events.read() {
        spawn_writer.send(SpawnInfectedEvent {
            position: ev.position,
            variant: InfectedVariant::Walkers,
        });
    }
}

fn infected_young_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    infected_young_units: Query<&SimPosition, With<InfectedYoung>>,
) {
    for position in &infected_young_units {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
    }
}

pub struct InfectedYoungPlugin;

impl Plugin for InfectedYoungPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnInfectedYoungEvent>()
            .add_systems(
                FixedUpdate,
                (spawn_infected_young_system, infected_young_checksum_system).chain(),
            );
    }
}