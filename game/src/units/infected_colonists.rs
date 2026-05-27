// Sources: vault/infected/infected_colonists.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimPosition, UnitStats};
use crate::units::infected::{
    ArmorReduction, CanInfectBuildings, DamageBounds, ExperienceReward, HpBounds, InfectedAoeProfile,
    InfectedAttackType, InfectedBundle, InfectedVariant, NextReplicatedUnitId, NoiseGeneration,
    RegenerationPerSecond, ReplicatedUnitId, UsesFriendlyFire, UsesSightOnlyAggro, VenomResistance,
};

const INFECTED_COLONIST_HP: I32F32 = I32F32::lit("45");
const INFECTED_COLONIST_MOVE_SPEED: I32F32 = I32F32::lit("1.75");
const INFECTED_COLONIST_ATTACK_RANGE: I32F32 = I32F32::lit("0.6");
const INFECTED_COLONIST_ATTACK_SPEED: I32F32 = I32F32::lit("1.11");
const INFECTED_COLONIST_ATTACK_DAMAGE: I32F32 = I32F32::lit("9");
const INFECTED_COLONIST_WATCH_RANGE: I32F32 = I32F32::lit("6");
const INFECTED_COLONIST_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.05");
const INFECTED_COLONIST_NOISE_GENERATION: I32F32 = I32F32::lit("1");
const INFECTED_COLONIST_EXPERIENCE_REWARD: I32F32 = I32F32::lit("2");
const INFECTED_COLONIST_REGEN_PER_SECOND: I32F32 = I32F32::ZERO;

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedColonistsUnit;

#[derive(Bundle)]
pub struct InfectedColonistsBundle {
    pub colonists: InfectedColonistsUnit,
    pub infected: InfectedBundle,
}

impl Default for InfectedColonistsBundle {
    fn default() -> Self {
        let mut infected = InfectedBundle::default();
        infected.health = Health::full(INFECTED_COLONIST_HP);
        infected.stats = UnitStats {
            move_speed: INFECTED_COLONIST_MOVE_SPEED,
            attack_range: INFECTED_COLONIST_ATTACK_RANGE,
            attack_damage: INFECTED_COLONIST_ATTACK_DAMAGE,
            attack_speed: INFECTED_COLONIST_ATTACK_SPEED,
            watch_range: INFECTED_COLONIST_WATCH_RANGE,
        };
        infected.armor_reduction = ArmorReduction(INFECTED_COLONIST_ARMOR_REDUCTION);
        infected.regeneration_per_second = RegenerationPerSecond(INFECTED_COLONIST_REGEN_PER_SECOND);
        infected.noise_generation = NoiseGeneration(INFECTED_COLONIST_NOISE_GENERATION);
        infected.venom_resistance = VenomResistance(I32F32::ONE);
        infected.experience_reward = ExperienceReward(INFECTED_COLONIST_EXPERIENCE_REWARD);
        infected.uses_sight_only_aggro = UsesSightOnlyAggro(false);
        infected.can_infect_buildings = CanInfectBuildings(true);
        infected.uses_friendly_fire = UsesFriendlyFire(false);
        infected.hp_bounds = HpBounds {
            min: INFECTED_COLONIST_HP,
            max: INFECTED_COLONIST_HP,
        };
        infected.damage_bounds = DamageBounds {
            min: INFECTED_COLONIST_ATTACK_DAMAGE,
            max: INFECTED_COLONIST_ATTACK_DAMAGE,
        };
        infected.attack_type = InfectedAttackType::Melee;
        infected.aoe_profile = InfectedAoeProfile::None;
        infected.variant = InfectedVariant::Colonist;

        Self {
            colonists: InfectedColonistsUnit,
            infected,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedColonistsEvent {
    pub position: SimPosition,
}

fn spawn_infected_colonists_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedColonistsEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedColonistsBundle::default();
        bundle.infected.position = ev.position;
        bundle.infected.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

pub struct InfectedColonistsPlugin;

impl Plugin for InfectedColonistsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnInfectedColonistsEvent>()
            .add_systems(FixedUpdate, (spawn_infected_colonists_system,).chain());
    }
}