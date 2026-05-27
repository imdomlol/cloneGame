// Sources: vault/infected/infected_chubby.md

use fixed::types::I32F32;

use crate::sim::{Health, SimPosition, UnitStats};
use crate::units::infected::{
    ArmorReduction, DamageBounds, ExperienceReward, HpBounds, InfectedAoeProfile, InfectedAttackType,
    InfectedBundle, InfectedVariant, NoiseGeneration, RegenerationPerSecond, ReplicatedUnitId,
};

const INFECTED_CHUBBY_HP: I32F32 = I32F32::lit("500");
const INFECTED_CHUBBY_MOVE_SPEED: I32F32 = I32F32::lit("1.75");
const INFECTED_CHUBBY_ATTACK_RANGE: I32F32 = I32F32::lit("0.7");
const INFECTED_CHUBBY_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const INFECTED_CHUBBY_ATTACK_DAMAGE: I32F32 = I32F32::lit("40");
const INFECTED_CHUBBY_WATCH_RANGE: I32F32 = I32F32::lit("5");
const INFECTED_CHUBBY_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.15");
const INFECTED_CHUBBY_NOISE: I32F32 = I32F32::lit("10");
const INFECTED_CHUBBY_EXPERIENCE: I32F32 = I32F32::lit("10");

pub fn infected_chubby_bundle(replicated_id: u64, position: SimPosition) -> InfectedBundle {
    let mut bundle = InfectedBundle::default();
    bundle.replicated_id = ReplicatedUnitId(replicated_id);
    bundle.position = position;
    bundle.health = Health::full(INFECTED_CHUBBY_HP);
    bundle.stats = UnitStats {
        move_speed: INFECTED_CHUBBY_MOVE_SPEED,
        attack_range: INFECTED_CHUBBY_ATTACK_RANGE,
        attack_damage: INFECTED_CHUBBY_ATTACK_DAMAGE,
        attack_speed: INFECTED_CHUBBY_ATTACK_SPEED,
        watch_range: INFECTED_CHUBBY_WATCH_RANGE,
    };
    bundle.armor_reduction = ArmorReduction(INFECTED_CHUBBY_ARMOR_REDUCTION);
    bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("20"));
    bundle.noise_generation = NoiseGeneration(INFECTED_CHUBBY_NOISE);
    bundle.experience_reward = ExperienceReward(INFECTED_CHUBBY_EXPERIENCE);
    bundle.attack_type = InfectedAttackType::Melee;
    bundle.aoe_profile = InfectedAoeProfile::None;
    bundle.hp_bounds = HpBounds {
        min: INFECTED_CHUBBY_HP,
        max: INFECTED_CHUBBY_HP,
    };
    bundle.damage_bounds = DamageBounds {
        min: INFECTED_CHUBBY_ATTACK_DAMAGE,
        max: INFECTED_CHUBBY_ATTACK_DAMAGE,
    };
    bundle.variant = InfectedVariant::Chubby;
    bundle
}