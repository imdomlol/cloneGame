// Sources: vault/infected/infected_harpy.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimPosition, UnitStats};
use crate::units::infected::{
    ArmorReduction, CanInfectBuildings, DamageBounds, ExperienceReward, HpBounds, InfectedAoeProfile,
    InfectedAttackType, InfectedBundle, InfectedVariant, NextReplicatedUnitId, NoiseGeneration,
    RegenerationPerSecond, ReplicatedUnitId, UsesFriendlyFire, UsesSightOnlyAggro, VenomResistance,
};

const INFECTED_HARPY_HP: I32F32 = I32F32::lit("120");
const INFECTED_HARPY_MOVE_SPEED: I32F32 = I32F32::lit("5");
const INFECTED_HARPY_ATTACK_RANGE: I32F32 = I32F32::lit("0.8");
const INFECTED_HARPY_ATTACK_SPEED: I32F32 = I32F32::lit("3.33");
const INFECTED_HARPY_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
const INFECTED_HARPY_WATCH_RANGE: I32F32 = I32F32::lit("9");
const INFECTED_HARPY_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");
const INFECTED_HARPY_NOISE_GENERATION: I32F32 = I32F32::lit("10");
const INFECTED_HARPY_EXPERIENCE_REWARD: I32F32 = I32F32::lit("10");
const INFECTED_HARPY_REGEN_PER_SECOND: I32F32 = I32F32::lit("10");

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedHarpyUnit;

#[derive(Bundle)]
pub struct InfectedHarpyBundle {
    pub harpy: InfectedHarpyUnit,
    pub infected: InfectedBundle,
}

impl Default for InfectedHarpyBundle {
    fn default() -> Self {
        let mut infected = InfectedBundle::default();
        infected.health = Health::full(INFECTED_HARPY_HP);
        infected.stats = UnitStats {
            move_speed: INFECTED_HARPY_MOVE_SPEED,
            attack_range: INFECTED_HARPY_ATTACK_RANGE,
            attack_damage: INFECTED_HARPY_ATTACK_DAMAGE,
            attack_speed: INFECTED_HARPY_ATTACK_SPEED,
            watch_range: INFECTED_HARPY_WATCH_RANGE,
        };
        infected.armor_reduction = ArmorReduction(INFECTED_HARPY_ARMOR_REDUCTION);
        infected.regeneration_per_second = RegenerationPerSecond(INFECTED_HARPY_REGEN_PER_SECOND);
        infected.noise_generation = NoiseGeneration(INFECTED_HARPY_NOISE_GENERATION);
        infected.venom_resistance = VenomResistance(I32F32::ONE);
        infected.experience_reward = ExperienceReward(INFECTED_HARPY_EXPERIENCE_REWARD);
        infected.uses_sight_only_aggro = UsesSightOnlyAggro(false);
        infected.can_infect_buildings = CanInfectBuildings(true);
        infected.uses_friendly_fire = UsesFriendlyFire(false);
        infected.hp_bounds = HpBounds {
            min: INFECTED_HARPY_HP,
            max: INFECTED_HARPY_HP,
        };
        infected.damage_bounds = DamageBounds {
            min: INFECTED_HARPY_ATTACK_DAMAGE,
            max: INFECTED_HARPY_ATTACK_DAMAGE,
        };
        infected.attack_type = InfectedAttackType::Melee;
        infected.aoe_profile = InfectedAoeProfile::None;
        infected.variant = InfectedVariant::Harpy;

        Self {
            harpy: InfectedHarpyUnit,
            infected,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedHarpyEvent {
    pub position: SimPosition,
}

fn spawn_infected_harpy_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedHarpyEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedHarpyBundle::default();
        bundle.infected.position = ev.position;
        bundle.infected.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

pub struct InfectedHarpyPlugin;

impl Plugin for InfectedHarpyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnInfectedHarpyEvent>()
            .add_systems(FixedUpdate, (spawn_infected_harpy_system,).chain());
    }
}