// Sources: vault/infected/v9.md

use bevy::prelude::*;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimPosition, SimTick};
use crate::units::infected::{InfectedVariant, ReplicatedUnitId, SpawnInfectedEvent};

const V9_HARPY_MUTATION_PERCENT: u32 = 5;
const V9_MUTATION_ROLL_MODULUS: u32 = 100;
const V9_MUTATION_SALT: u64 = 0x6D58_2A91_E34C_7B1D;

#[derive(Component, Clone, Copy, Default)]
pub struct V9Infected;

#[derive(Event, Clone, Copy)]
pub struct AdministerV9Event {
    pub target: Entity,
    pub infected_woman: bool,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct V9MutationStats {
    pub administration_attempts: u64,
    pub successful_mutations: u64,
}

fn administer_v9_system(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    mut commands: Commands,
    mut events: EventReader<AdministerV9Event>,
    infected_women: Query<(&SimPosition, &ReplicatedUnitId), With<V9Infected>>,
    mut spawn_writer: EventWriter<SpawnInfectedEvent>,
    mut stats: ResMut<V9MutationStats>,
) {
    for ev in events.read() {
        if !ev.infected_woman {
            continue;
        }

        let Ok((position, replicated_id)) = infected_women.get(ev.target) else {
            continue;
        };

        stats.administration_attempts = stats.administration_attempts.wrapping_add(1);

        let salt = V9_MUTATION_SALT ^ replicated_id.0;
        let mut rng = tick_rng(game_seed.0, sim_tick.0, salt);
        let roll = rng.next_u32() % V9_MUTATION_ROLL_MODULUS;
        if roll < V9_HARPY_MUTATION_PERCENT {
            spawn_writer.send(SpawnInfectedEvent {
                position: *position,
                variant: InfectedVariant::Harpy,
            });
            commands.entity(ev.target).despawn();
            stats.successful_mutations = stats.successful_mutations.wrapping_add(1);
        }
    }
}

fn v9_checksum_system(mut checksum: ResMut<SimChecksumState>, stats: Res<V9MutationStats>) {
    checksum.accumulate(stats.administration_attempts);
    checksum.accumulate(stats.successful_mutations);
}

pub struct V9Plugin;

impl Plugin for V9Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<V9MutationStats>()
            .add_event::<AdministerV9Event>()
            .add_systems(
                FixedUpdate,
                (administer_v9_system, v9_checksum_system).chain(),
            );
    }
}