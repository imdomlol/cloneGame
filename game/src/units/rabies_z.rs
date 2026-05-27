// Sources: vault/infected/rabies_z.md, vault/infected/infected.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{EntityKilledEvent, Health, SimPosition, UnitStats};
use crate::units::infected::{
    ArmorReduction, CanInfectBuildings, DamageBounds, ExperienceReward, HpBounds, InfectedAoeProfile,
    InfectedAttackType, InfectedBundle, InfectedVariant, NextReplicatedUnitId, NoiseGeneration,
    RegenerationPerSecond, ReplicatedUnitId, SpawnInfectedEvent, UsesFriendlyFire, UsesSightOnlyAggro,
    VenomResistance,
};

const RABIES_Z_HP: I32F32 = I32F32::ZERO;
const RABIES_Z_MOVE_SPEED: I32F32 = I32F32::ZERO;
const RABIES_Z_ATTACK_RANGE: I32F32 = I32F32::ZERO;
const RABIES_Z_ATTACK_SPEED: I32F32 = I32F32::ZERO;
const RABIES_Z_ATTACK_DAMAGE: I32F32 = I32F32::ZERO;
const RABIES_Z_WATCH_RANGE: I32F32 = I32F32::ZERO;
const RABIES_Z_ARMOR_REDUCTION: I32F32 = I32F32::ZERO;
const RABIES_Z_NOISE_GENERATION: I32F32 = I32F32::ZERO;
const RABIES_Z_EXPERIENCE_REWARD: I32F32 = I32F32::ZERO;
const RABIES_Z_REGEN_PER_SECOND: I32F32 = I32F32::ZERO;
const RABIES_Z_SPAWN_COUNT_ON_DEATH: u32 = 10;

#[derive(Component, Clone, Copy, Default)]
pub struct RabiesZUnit;

#[derive(Bundle)]
pub struct RabiesZBundle {
    pub rabies_z: RabiesZUnit,
    pub infected: InfectedBundle,
}

impl Default for RabiesZBundle {
    fn default() -> Self {
        let mut infected = InfectedBundle::default();
        infected.health = Health::full(RABIES_Z_HP);
        infected.stats = UnitStats {
            move_speed: RABIES_Z_MOVE_SPEED,
            attack_range: RABIES_Z_ATTACK_RANGE,
            attack_damage: RABIES_Z_ATTACK_DAMAGE,
            attack_speed: RABIES_Z_ATTACK_SPEED,
            watch_range: RABIES_Z_WATCH_RANGE,
        };
        infected.armor_reduction = ArmorReduction(RABIES_Z_ARMOR_REDUCTION);
        infected.regeneration_per_second = RegenerationPerSecond(RABIES_Z_REGEN_PER_SECOND);
        infected.noise_generation = NoiseGeneration(RABIES_Z_NOISE_GENERATION);
        infected.venom_resistance = VenomResistance(I32F32::ONE);
        infected.experience_reward = ExperienceReward(RABIES_Z_EXPERIENCE_REWARD);
        infected.uses_sight_only_aggro = UsesSightOnlyAggro(false);
        infected.can_infect_buildings = CanInfectBuildings(true);
        infected.uses_friendly_fire = UsesFriendlyFire(false);
        infected.hp_bounds = HpBounds {
            min: RABIES_Z_HP,
            max: RABIES_Z_HP,
        };
        infected.damage_bounds = DamageBounds {
            min: RABIES_Z_ATTACK_DAMAGE,
            max: RABIES_Z_ATTACK_DAMAGE,
        };
        infected.attack_type = InfectedAttackType::Melee;
        infected.aoe_profile = InfectedAoeProfile::None;
        infected.variant = InfectedVariant::Walkers;

        Self {
            rabies_z: RabiesZUnit,
            infected,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SpawnRabiesZEvent {
    pub position: SimPosition,
}

fn spawn_rabies_z_system(
    mut commands: Commands,
    mut events: EventReader<SpawnRabiesZEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = RabiesZBundle::default();
        bundle.infected.position = ev.position;
        bundle.infected.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn rabies_z_on_killed_spawn_infected_system(
    mut killed_events: EventReader<EntityKilledEvent>,
    rabies_z_units: Query<&SimPosition, With<RabiesZUnit>>,
    mut spawn_writer: EventWriter<SpawnInfectedEvent>,
) {
    for ev in killed_events.read() {
        let Ok(position) = rabies_z_units.get(ev.entity) else {
            continue;
        };

        for _ in 0..RABIES_Z_SPAWN_COUNT_ON_DEATH {
            spawn_writer.send(SpawnInfectedEvent {
                position: *position,
                variant: InfectedVariant::Walkers,
            });
        }
    }
}

pub struct RabiesZPlugin;

impl Plugin for RabiesZPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnRabiesZEvent>().add_systems(
            FixedUpdate,
            (
                spawn_rabies_z_system,
                rabies_z_on_killed_spawn_infected_system,
            )
                .chain(),
        );
    }
}