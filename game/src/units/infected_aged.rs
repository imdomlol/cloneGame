// Sources: vault/infected/infected_aged.md, vault/infected/infected_decrepit.md, vault/infected/infected_young.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimPosition, UnitStats};
use crate::units::infected::{
    ArmorReduction, CanInfectBuildings, DamageBounds, ExperienceReward, HpBounds, InfectedAoeProfile,
    InfectedAttackType, InfectedBundle, InfectedVariant, NextReplicatedUnitId, NoiseGeneration,
    RegenerationPerSecond, ReplicatedUnitId, UsesFriendlyFire, UsesSightOnlyAggro, VenomResistance,
};

const INFECTED_AGED_HP: I32F32 = I32F32::lit("35");
const INFECTED_AGED_MOVE_SPEED: I32F32 = I32F32::lit("0.4");
const INFECTED_AGED_ATTACK_RANGE: I32F32 = I32F32::lit("0.5");
const INFECTED_AGED_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const INFECTED_AGED_ATTACK_DAMAGE: I32F32 = I32F32::lit("6");
const INFECTED_AGED_WATCH_RANGE: I32F32 = I32F32::lit("5");
const INFECTED_AGED_ARMOR_REDUCTION: I32F32 = I32F32::lit("0");
const INFECTED_AGED_NOISE_GENERATION: I32F32 = I32F32::lit("1");
const INFECTED_AGED_EXPERIENCE_REWARD: I32F32 = I32F32::lit("1");
const INFECTED_AGED_REGEN_PER_SECOND: I32F32 = I32F32::lit("3");

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedAged;

#[derive(Bundle)]
pub struct InfectedAgedBundle {
    pub aged: InfectedAged,
    pub infected: InfectedBundle,
}

impl Default for InfectedAgedBundle {
    fn default() -> Self {
        let mut infected = InfectedBundle::default();
        infected.health = Health::full(INFECTED_AGED_HP);
        infected.stats = UnitStats {
            move_speed: INFECTED_AGED_MOVE_SPEED,
            attack_range: INFECTED_AGED_ATTACK_RANGE,
            attack_damage: INFECTED_AGED_ATTACK_DAMAGE,
            attack_speed: INFECTED_AGED_ATTACK_SPEED,
            watch_range: INFECTED_AGED_WATCH_RANGE,
        };
        infected.armor_reduction = ArmorReduction(INFECTED_AGED_ARMOR_REDUCTION);
        infected.regeneration_per_second = RegenerationPerSecond(INFECTED_AGED_REGEN_PER_SECOND);
        infected.noise_generation = NoiseGeneration(INFECTED_AGED_NOISE_GENERATION);
        infected.venom_resistance = VenomResistance(I32F32::ONE);
        infected.experience_reward = ExperienceReward(INFECTED_AGED_EXPERIENCE_REWARD);
        infected.uses_sight_only_aggro = UsesSightOnlyAggro(false);
        infected.can_infect_buildings = CanInfectBuildings(true);
        infected.uses_friendly_fire = UsesFriendlyFire(false);
        infected.hp_bounds = HpBounds {
            min: INFECTED_AGED_HP,
            max: INFECTED_AGED_HP,
        };
        infected.damage_bounds = DamageBounds {
            min: INFECTED_AGED_ATTACK_DAMAGE,
            max: INFECTED_AGED_ATTACK_DAMAGE,
        };
        infected.attack_type = InfectedAttackType::Melee;
        infected.aoe_profile = InfectedAoeProfile::None;
        infected.variant = InfectedVariant::Walkers;

        Self {
            aged: InfectedAged,
            infected,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedAgedEvent {
    pub position: SimPosition,
}

fn spawn_infected_aged_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedAgedEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedAgedBundle::default();
        bundle.infected.position = ev.position;
        bundle.infected.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

pub struct InfectedAgedPlugin;

impl Plugin for InfectedAgedPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnInfectedAgedEvent>()
            .add_systems(FixedUpdate, (spawn_infected_aged_system,).chain());
    }
}