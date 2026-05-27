// Sources: vault/infected/infected_executive.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimPosition, UnitStats};
use crate::units::infected::{
    ArmorReduction, CanInfectBuildings, DamageBounds, ExperienceReward, HpBounds, InfectedAoeProfile,
    InfectedAttackType, InfectedBundle, InfectedVariant, NextReplicatedUnitId, NoiseGeneration,
    RegenerationPerSecond, ReplicatedUnitId, UsesFriendlyFire, UsesSightOnlyAggro, VenomResistance,
};

const INFECTED_EXECUTIVE_HP: I32F32 = I32F32::lit("80");
const INFECTED_EXECUTIVE_MOVE_SPEED: I32F32 = I32F32::lit("1.75");
const INFECTED_EXECUTIVE_ATTACK_RANGE: I32F32 = I32F32::lit("0.6");
const INFECTED_EXECUTIVE_ATTACK_SPEED: I32F32 = I32F32::lit("1.25");
const INFECTED_EXECUTIVE_ATTACK_DAMAGE: I32F32 = I32F32::lit("12");
const INFECTED_EXECUTIVE_WATCH_RANGE: I32F32 = I32F32::lit("6");
const INFECTED_EXECUTIVE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");
const INFECTED_EXECUTIVE_NOISE_GENERATION: I32F32 = I32F32::lit("3");
const INFECTED_EXECUTIVE_EXPERIENCE_REWARD: I32F32 = I32F32::lit("5");
const INFECTED_EXECUTIVE_REGEN_PER_SECOND: I32F32 = I32F32::lit("7");

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedExecutiveUnit;

#[derive(Bundle)]
pub struct InfectedExecutiveBundle {
    pub executive: InfectedExecutiveUnit,
    pub infected: InfectedBundle,
}

impl Default for InfectedExecutiveBundle {
    fn default() -> Self {
        let mut infected = InfectedBundle::default();
        infected.health = Health::full(INFECTED_EXECUTIVE_HP);
        infected.stats = UnitStats {
            move_speed: INFECTED_EXECUTIVE_MOVE_SPEED,
            attack_range: INFECTED_EXECUTIVE_ATTACK_RANGE,
            attack_damage: INFECTED_EXECUTIVE_ATTACK_DAMAGE,
            attack_speed: INFECTED_EXECUTIVE_ATTACK_SPEED,
            watch_range: INFECTED_EXECUTIVE_WATCH_RANGE,
        };
        infected.armor_reduction = ArmorReduction(INFECTED_EXECUTIVE_ARMOR_REDUCTION);
        infected.regeneration_per_second = RegenerationPerSecond(INFECTED_EXECUTIVE_REGEN_PER_SECOND);
        infected.noise_generation = NoiseGeneration(INFECTED_EXECUTIVE_NOISE_GENERATION);
        infected.venom_resistance = VenomResistance(I32F32::ONE);
        infected.experience_reward = ExperienceReward(INFECTED_EXECUTIVE_EXPERIENCE_REWARD);
        infected.uses_sight_only_aggro = UsesSightOnlyAggro(false);
        infected.can_infect_buildings = CanInfectBuildings(true);
        infected.uses_friendly_fire = UsesFriendlyFire(false);
        infected.hp_bounds = HpBounds {
            min: INFECTED_EXECUTIVE_HP,
            max: INFECTED_EXECUTIVE_HP,
        };
        infected.damage_bounds = DamageBounds {
            min: INFECTED_EXECUTIVE_ATTACK_DAMAGE,
            max: INFECTED_EXECUTIVE_ATTACK_DAMAGE,
        };
        infected.attack_type = InfectedAttackType::Melee;
        infected.aoe_profile = InfectedAoeProfile::None;
        infected.variant = InfectedVariant::Executive;

        Self {
            executive: InfectedExecutiveUnit,
            infected,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedExecutiveEvent {
    pub position: SimPosition,
}

fn spawn_infected_executive_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedExecutiveEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedExecutiveBundle::default();
        bundle.infected.position = ev.position;
        bundle.infected.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

pub struct InfectedExecutivePlugin;

impl Plugin for InfectedExecutivePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnInfectedExecutiveEvent>()
            .add_systems(FixedUpdate, (spawn_infected_executive_system,).chain());
    }
}