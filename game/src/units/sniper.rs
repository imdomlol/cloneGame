// Sources: vault/units/sniper.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, SimPosition, UnitStats};

const SNIPER_HP: I32F32 = I32F32::lit("150");
const SNIPER_MOVE_SPEED: I32F32 = I32F32::lit("1.8");
const SNIPER_ATTACK_RANGE: I32F32 = I32F32::lit("8");
const SNIPER_ATTACK_SPEED: I32F32 = I32F32::lit("0.45");
const SNIPER_ATTACK_DAMAGE: I32F32 = I32F32::lit("100");
const SNIPER_WATCH_RANGE: I32F32 = I32F32::lit("9");
const SNIPER_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.05");
const SNIPER_ATTACK_NOISE: I32F32 = I32F32::lit("10");
const SNIPER_ATTACK_RANGE_VETERAN: I32F32 = I32F32::lit("8");
const SNIPER_ATTACK_SPEED_VETERAN: I32F32 = I32F32::lit("0.91");
const SNIPER_ATTACK_DAMAGE_VETERAN: I32F32 = I32F32::lit("110");
const SNIPER_EXPERIENCE_REWARD: u32 = 140;
const SNIPER_EXPERIENCE_TO_VETERAN: u32 = 140;

#[derive(Component, Default)]
pub struct Sniper;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct AttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct AppliesKnockback(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct CarryingExplosiveBarrel(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperiencePoints(pub u32);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperienceToVeteran(pub u32);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperienceReward(pub u32);

#[derive(Component, Clone, Copy, Default)]
pub struct IsVeteran(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct CampaignSurvivabilityDamageApplied(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct CampaignTrainingVeteranApplied(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct CampaignGeneralArmorApplied(pub bool);

#[derive(Bundle)]
pub struct SniperBundle {
    pub unit: Sniper,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub attack_noise: AttackNoise,
    pub applies_knockback: AppliesKnockback,
    pub carrying_explosive_barrel: CarryingExplosiveBarrel,
    pub experience_points: ExperiencePoints,
    pub experience_to_veteran: ExperienceToVeteran,
    pub experience_reward: ExperienceReward,
    pub is_veteran: IsVeteran,
    pub campaign_survivability_damage_applied: CampaignSurvivabilityDamageApplied,
    pub campaign_training_veteran_applied: CampaignTrainingVeteranApplied,
    pub campaign_general_armor_applied: CampaignGeneralArmorApplied,
}

impl Default for SniperBundle {
    fn default() -> Self {
        Self {
            unit: Sniper,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(SNIPER_HP),
            stats: UnitStats {
                move_speed: SNIPER_MOVE_SPEED,
                attack_range: SNIPER_ATTACK_RANGE,
                attack_damage: SNIPER_ATTACK_DAMAGE,
                attack_speed: SNIPER_ATTACK_SPEED,
                watch_range: SNIPER_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction(SNIPER_ARMOR_REDUCTION),
            attack_noise: AttackNoise(SNIPER_ATTACK_NOISE),
            applies_knockback: AppliesKnockback(true),
            carrying_explosive_barrel: CarryingExplosiveBarrel(false),
            experience_points: ExperiencePoints(0),
            experience_to_veteran: ExperienceToVeteran(SNIPER_EXPERIENCE_TO_VETERAN),
            experience_reward: ExperienceReward(SNIPER_EXPERIENCE_REWARD),
            is_veteran: IsVeteran(false),
            campaign_survivability_damage_applied: CampaignSurvivabilityDamageApplied(false),
            campaign_training_veteran_applied: CampaignTrainingVeteranApplied(false),
            campaign_general_armor_applied: CampaignGeneralArmorApplied(false),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnSniperEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct SetSniperBarrelCarryEvent {
    pub entity: Entity,
    pub carrying: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SniperCampaignUpgrade {
    SurvivabilityAndDamage,
    TrainingAndVeteranPerks,
    GeneralArmor,
}

#[derive(Event, Clone, Copy)]
pub struct ApplySniperCampaignUpgradeEvent {
    pub entity: Entity,
    pub upgrade: SniperCampaignUpgrade,
}

fn spawn_sniper_system(
    mut commands: Commands,
    mut events: EventReader<SpawnSniperEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = SniperBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn set_sniper_barrel_carry_system(
    mut events: EventReader<SetSniperBarrelCarryEvent>,
    mut snipers: Query<&mut CarryingExplosiveBarrel, With<Sniper>>,
) {
    for ev in events.read() {
        let Ok(mut carrying) = snipers.get_mut(ev.entity) else {
            continue;
        };
        carrying.0 = ev.carrying;
    }
}

fn apply_sniper_campaign_upgrade_system(
    mut events: EventReader<ApplySniperCampaignUpgradeEvent>,
    mut snipers: Query<
        (
            &mut Health,
            &mut UnitStats,
            &mut ArmorReduction,
            &mut ExperienceToVeteran,
            &mut CampaignSurvivabilityDamageApplied,
            &mut CampaignTrainingVeteranApplied,
            &mut CampaignGeneralArmorApplied,
        ),
        With<Sniper>,
    >,
) {
    for ev in events.read() {
        let Ok((
            mut health,
            mut stats,
            mut armor,
            mut exp_to_veteran,
            mut survivability_and_damage_applied,
            mut training_and_veteran_applied,
            mut general_armor_applied,
        )) = snipers.get_mut(ev.entity)
        else {
            continue;
        };

        match ev.upgrade {
            SniperCampaignUpgrade::SurvivabilityAndDamage => {
                if survivability_and_damage_applied.0 {
                    continue;
                }

                health.max = health.max * I32F32::lit("1.20");
                health.current = health.current * I32F32::lit("1.20");
                stats.watch_range = stats.watch_range + I32F32::ONE;
                stats.attack_range = stats.attack_range + I32F32::ONE;
                stats.attack_damage = stats.attack_damage * I32F32::lit("1.20");
                survivability_and_damage_applied.0 = true;
            }
            SniperCampaignUpgrade::TrainingAndVeteranPerks => {
                if training_and_veteran_applied.0 {
                    continue;
                }

                exp_to_veteran.0 = exp_to_veteran.0.saturating_mul(80) / 100;
                training_and_veteran_applied.0 = true;
            }
            SniperCampaignUpgrade::GeneralArmor => {
                if general_armor_applied.0 {
                    continue;
                }

                armor.0 = armor.0 + I32F32::lit("0.05");
                general_armor_applied.0 = true;
            }
        }
    }
}

fn promote_sniper_veteran_system(
    mut snipers: Query<(&ExperiencePoints, &ExperienceToVeteran, &mut UnitStats, &mut IsVeteran), With<Sniper>>,
) {
    for (exp, exp_to_veteran, mut stats, mut veteran) in &mut snipers {
        if veteran.0 || exp.0 < exp_to_veteran.0 {
            continue;
        }

        veteran.0 = true;
        stats.attack_range = SNIPER_ATTACK_RANGE_VETERAN;
        stats.attack_speed = SNIPER_ATTACK_SPEED_VETERAN;
        stats.attack_damage = SNIPER_ATTACK_DAMAGE_VETERAN;
    }
}

fn sniper_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    snipers: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &AttackNoise,
            &AppliesKnockback,
            &CarryingExplosiveBarrel,
            &ExperiencePoints,
            &ExperienceToVeteran,
            &ExperienceReward,
            &IsVeteran,
            &CampaignSurvivabilityDamageApplied,
            &CampaignTrainingVeteranApplied,
            &CampaignGeneralArmorApplied,
        ),
        With<Sniper>,
    >,
) {
    for (
        replicated_id,
        position,
        health,
        stats,
        armor_reduction,
        attack_noise,
        applies_knockback,
        carrying_explosive_barrel,
        experience_points,
        experience_to_veteran,
        experience_reward,
        is_veteran,
        campaign_survivability_damage_applied,
        campaign_training_veteran_applied,
        campaign_general_armor_applied,
    ) in &snipers
    {
        checksum.accumulate(replicated_id.0);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor_reduction.0.to_bits() as u64);
        checksum.accumulate(attack_noise.0.to_bits() as u64);
        checksum.accumulate(u64::from(applies_knockback.0));
        checksum.accumulate(u64::from(carrying_explosive_barrel.0));
        checksum.accumulate(experience_points.0 as u64);
        checksum.accumulate(experience_to_veteran.0 as u64);
        checksum.accumulate(experience_reward.0 as u64);
        checksum.accumulate(u64::from(is_veteran.0));
        checksum.accumulate(u64::from(campaign_survivability_damage_applied.0));
        checksum.accumulate(u64::from(campaign_training_veteran_applied.0));
        checksum.accumulate(u64::from(campaign_general_armor_applied.0));
    }
}

pub struct SniperPlugin;

impl Plugin for SniperPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnSniperEvent>()
            .add_event::<SetSniperBarrelCarryEvent>()
            .add_event::<ApplySniperCampaignUpgradeEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_sniper_system,
                    set_sniper_barrel_carry_system,
                    apply_sniper_campaign_upgrade_system,
                    promote_sniper_veteran_system,
                    sniper_checksum_system,
                )
                    .chain(),
            );
    }
}