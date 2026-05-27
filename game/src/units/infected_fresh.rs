// Sources: vault/infected/infected_fresh.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimPosition, UnitStats};
use crate::units::infected::{
    ArmorReduction, CanInfectBuildings, DamageBounds, ExperienceReward, HpBounds, InfectedAoeProfile,
    InfectedAttackType, InfectedBundle, InfectedVariant, NextReplicatedUnitId, NoiseGeneration,
    RegenerationPerSecond, ReplicatedUnitId, UsesFriendlyFire, UsesSightOnlyAggro, VenomResistance,
};

const INFECTED_FRESH_HP: I32F32 = I32F32::lit("45");
const INFECTED_FRESH_MOVE_SPEED: I32F32 = I32F32::lit("1.75");
const INFECTED_FRESH_ATTACK_RANGE: I32F32 = I32F32::lit("0.6");
const INFECTED_FRESH_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const INFECTED_FRESH_ATTACK_DAMAGE: I32F32 = I32F32::lit("10");
const INFECTED_FRESH_WATCH_RANGE: I32F32 = I32F32::lit("6");
const INFECTED_FRESH_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.05");
const INFECTED_FRESH_NOISE_GENERATION: I32F32 = I32F32::lit("2");
const INFECTED_FRESH_EXPERIENCE_REWARD: I32F32 = I32F32::lit("2");
const INFECTED_FRESH_REGEN_PER_SECOND: I32F32 = I32F32::ZERO;

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedFreshUnit;

#[derive(Bundle)]
pub struct InfectedFreshBundle {
    pub fresh: InfectedFreshUnit,
    pub infected: InfectedBundle,
}

impl Default for InfectedFreshBundle {
    fn default() -> Self {
        let mut infected = InfectedBundle::default();
        infected.health = Health::full(INFECTED_FRESH_HP);
        infected.stats = UnitStats {
            move_speed: INFECTED_FRESH_MOVE_SPEED,
            attack_range: INFECTED_FRESH_ATTACK_RANGE,
            attack_damage: INFECTED_FRESH_ATTACK_DAMAGE,
            attack_speed: INFECTED_FRESH_ATTACK_SPEED,
            watch_range: INFECTED_FRESH_WATCH_RANGE,
        };
        infected.armor_reduction = ArmorReduction(INFECTED_FRESH_ARMOR_REDUCTION);
        infected.regeneration_per_second = RegenerationPerSecond(INFECTED_FRESH_REGEN_PER_SECOND);
        infected.noise_generation = NoiseGeneration(INFECTED_FRESH_NOISE_GENERATION);
        infected.venom_resistance = VenomResistance(I32F32::ONE);
        infected.experience_reward = ExperienceReward(INFECTED_FRESH_EXPERIENCE_REWARD);
        infected.uses_sight_only_aggro = UsesSightOnlyAggro(false);
        infected.can_infect_buildings = CanInfectBuildings(true);
        infected.uses_friendly_fire = UsesFriendlyFire(false);
        infected.hp_bounds = HpBounds {
            min: INFECTED_FRESH_HP,
            max: INFECTED_FRESH_HP,
        };
        infected.damage_bounds = DamageBounds {
            min: INFECTED_FRESH_ATTACK_DAMAGE,
            max: INFECTED_FRESH_ATTACK_DAMAGE,
        };
        infected.attack_type = InfectedAttackType::Melee;
        infected.aoe_profile = InfectedAoeProfile::None;
        infected.variant = InfectedVariant::Fresh;

        Self {
            fresh: InfectedFreshUnit,
            infected,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedFreshEvent {
    pub position: SimPosition,
}

fn spawn_infected_fresh_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedFreshEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedFreshBundle::default();
        bundle.infected.position = ev.position;
        bundle.infected.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

pub struct InfectedFreshPlugin;

impl Plugin for InfectedFreshPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnInfectedFreshEvent>()
            .add_systems(FixedUpdate, (spawn_infected_fresh_system,).chain());
    }
}