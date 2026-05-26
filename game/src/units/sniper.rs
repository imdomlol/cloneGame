// Sources: vault/units/sniper.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimHz,
    SimPosition, UnitStats,
};
use crate::units::Infected;

pub const SNIPER_GOLD_COST: u32 = 300;
pub const SNIPER_FOOD_COST: u32 = 1;
pub const SNIPER_WORKER_COST: u32 = 1;
pub const SNIPER_IRON_COST: u32 = 2;
pub const SNIPER_WOOD_COST: u32 = 2;
pub const SNIPER_MAINTENANCE_GOLD: u32 = 5;
pub const SNIPER_BUILD_TIME_SECONDS: u32 = 33;

pub const SNIPER_HP: I32F32 = I32F32::lit("150");
pub const SNIPER_MOVE_SPEED: I32F32 = I32F32::lit("1.8");
pub const SNIPER_ATTACK_RANGE: I32F32 = I32F32::lit("8");
pub const SNIPER_ATTACK_SPEED: I32F32 = I32F32::lit("0.45");
pub const SNIPER_ATTACK_DAMAGE: I32F32 = I32F32::lit("100");
pub const SNIPER_WATCH_RANGE: I32F32 = I32F32::lit("9");
pub const SNIPER_NOISE_PER_ATTACK: I32F32 = I32F32::lit("10");
pub const SNIPER_EXP_REWARD: I32F32 = I32F32::lit("140");
pub const SNIPER_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.05");

pub const SNIPER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("8");
pub const SNIPER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("0.91");
pub const SNIPER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("110");

pub const SNIPER_CAMPAIGN_HP_MULTIPLIER: I32F32 = I32F32::lit("1.2");
pub const SNIPER_CAMPAIGN_VIEW_RANGE_BONUS: I32F32 = I32F32::lit("1");
pub const SNIPER_CAMPAIGN_ATTACK_RANGE_BONUS: I32F32 = I32F32::lit("1");
pub const SNIPER_CAMPAIGN_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("1.2");
pub const SNIPER_CAMPAIGN_ARMOR_BONUS: I32F32 = I32F32::lit("0.05");
pub const SNIPER_CAMPAIGN_TRAINING_TIME_MULTIPLIER: I32F32 = I32F32::lit("0.9");
pub const SNIPER_CAMPAIGN_VETERAN_EXP_MULTIPLIER: I32F32 = I32F32::lit("0.8");

pub const SNIPER_KNOCKBACK_DISTANCE: I32F32 = I32F32::lit("0.2");

#[derive(Component, Default)]
pub struct Sniper;

#[derive(Component, Clone, Copy)]
pub struct SniperArmor {
    pub standard_reduction: I32F32,
}

impl Default for SniperArmor {
    fn default() -> Self {
        Self {
            standard_reduction: SNIPER_ARMOR_REDUCTION,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct SniperAttackCooldown {
    pub ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct SniperVeteran {
    pub is_veteran: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct SniperCampaignUpgrades {
    pub applied: bool,
}

#[derive(Component, Clone, Copy)]
pub struct SniperEconomy {
    pub gold: u32,
    pub food: u32,
    pub workers: u32,
    pub iron: u32,
    pub wood: u32,
    pub maintenance_gold: u32,
    pub build_time_seconds: u32,
    pub build_time_multiplier: I32F32,
    pub veteran_exp_multiplier: I32F32,
}

impl Default for SniperEconomy {
    fn default() -> Self {
        Self {
            gold: SNIPER_GOLD_COST,
            food: SNIPER_FOOD_COST,
            workers: SNIPER_WORKER_COST,
            iron: SNIPER_IRON_COST,
            wood: SNIPER_WOOD_COST,
            maintenance_gold: SNIPER_MAINTENANCE_GOLD,
            build_time_seconds: SNIPER_BUILD_TIME_SECONDS,
            build_time_multiplier: I32F32::ONE,
            veteran_exp_multiplier: I32F32::ONE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct SniperBarrelCarrier {
    pub carrying_explosive_barrel: bool,
}

#[derive(Component, Clone, Copy)]
pub struct SniperExperience {
    pub reward: I32F32,
}

impl Default for SniperExperience {
    fn default() -> Self {
        Self {
            reward: SNIPER_EXP_REWARD,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct PromoteSniperEvent {
    pub sniper: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct ApplySniperCampaignUpgradesEvent {
    pub sniper: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct SetSniperBarrelCarryEvent {
    pub sniper: Entity,
    pub carrying: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SniperFiredEvent {
    pub sniper: Entity,
}

pub fn sniper_base_health() -> Health {
    Health::full(SNIPER_HP)
}

pub fn sniper_base_stats() -> UnitStats {
    UnitStats {
        move_speed: SNIPER_MOVE_SPEED,
        attack_range: SNIPER_ATTACK_RANGE,
        attack_damage: SNIPER_ATTACK_DAMAGE,
        attack_speed: SNIPER_ATTACK_SPEED,
        watch_range: SNIPER_WATCH_RANGE,
    }
}

pub fn spawn_sniper(commands: &mut Commands, position: SimPosition) -> Entity {
    commands
        .spawn((
            Sniper,
            position,
            sniper_base_health(),
            sniper_base_stats(),
            SniperArmor::default(),
            SniperAttackCooldown::default(),
            SniperVeteran::default(),
            SniperCampaignUpgrades::default(),
            SniperEconomy::default(),
            SniperBarrelCarrier::default(),
            SniperExperience::default(),
        ))
        .id()
}

fn sniper_attack_system(
    sim_hz: Res<SimHz>,
    mut snipers: Query<
        (Entity, &SimPosition, &UnitStats, &mut SniperAttackCooldown),
        (With<Sniper>, Without<Infected>),
    >,
    mut infected: Query<(Entity, &mut SimPosition), With<Infected>>,
    mut incoming_damage: EventWriter<IncomingDamageEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut fired_events: EventWriter<SniperFiredEvent>,
) {
    for (sniper_entity, sniper_pos, stats, mut cooldown) in &mut snipers {
        if cooldown.ticks_remaining > 0 {
            cooldown.ticks_remaining -= 1;
            continue;
        }

        let range_sq = stats.attack_range * stats.attack_range;
        let mut selected: Option<(Entity, I32F32, I32F32, I32F32)> = None;

        for (infected_entity, infected_pos) in &mut infected {
            let dx = infected_pos.x - sniper_pos.x;
            let dy = infected_pos.y - sniper_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > range_sq {
                continue;
            }

            let candidate = (infected_entity, dist_sq, infected_pos.x, infected_pos.y);
            selected = match selected {
                None => Some(candidate),
                Some(best) => {
                    if candidate.1 < best.1
                        || (candidate.1 == best.1
                            && (candidate.2 < best.2
                                || (candidate.2 == best.2 && candidate.3 < best.3)))
                    {
                        Some(candidate)
                    } else {
                        Some(best)
                    }
                }
            };
        }

        if let Some((target_entity, _dist_sq, target_x, target_y)) = selected {
            incoming_damage.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: stats.attack_damage,
                damage_type: DamageType::Standard,
                source: sniper_entity,
            });

            noise_events.send(NoiseEmittedEvent {
                source: sniper_entity,
                position: *sniper_pos,
                amount: SNIPER_NOISE_PER_ATTACK,
            });

            fired_events.send(SniperFiredEvent {
                sniper: sniper_entity,
            });

            if let Ok((_target_entity, mut target_pos)) = infected.get_mut(target_entity) {
                let dx = target_x - sniper_pos.x;
                let dy = target_y - sniper_pos.y;
                if dx != I32F32::ZERO || dy != I32F32::ZERO {
                    if dx.abs() >= dy.abs() {
                        if dx > I32F32::ZERO {
                            target_pos.x += SNIPER_KNOCKBACK_DISTANCE;
                        } else {
                            target_pos.x -= SNIPER_KNOCKBACK_DISTANCE;
                        }
                    } else if dy > I32F32::ZERO {
                        target_pos.y += SNIPER_KNOCKBACK_DISTANCE;
                    } else {
                        target_pos.y -= SNIPER_KNOCKBACK_DISTANCE;
                    }
                }
            }

            let mut ticks = (sim_hz.0 / stats.attack_speed).to_num::<u32>();
            if ticks == 0 {
                ticks = 1;
            }
            cooldown.ticks_remaining = ticks;
        }
    }
}

fn sniper_apply_damage_system(
    mut incoming_damage: EventReader<IncomingDamageEvent>,
    mut snipers: Query<(&mut Health, &SniperArmor), With<Sniper>>,
) {
    for event in incoming_damage.read() {
        if let Ok((mut health, armor)) = snipers.get_mut(event.target) {
            let effective_damage = event.raw_amount * (I32F32::ONE - armor.standard_reduction);
            health.current = (health.current - effective_damage).max(I32F32::ZERO);
        }
    }
}

fn sniper_promotion_system(
    mut promotions: EventReader<PromoteSniperEvent>,
    mut snipers: Query<(&mut UnitStats, &mut SniperVeteran), With<Sniper>>,
) {
    for event in promotions.read() {
        if let Ok((mut stats, mut veteran)) = snipers.get_mut(event.sniper) {
            if veteran.is_veteran {
                continue;
            }
            veteran.is_veteran = true;
            stats.attack_range = SNIPER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = SNIPER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = SNIPER_VETERAN_ATTACK_DAMAGE;
        }
    }
}

fn sniper_apply_campaign_upgrades_system(
    mut upgrades: EventReader<ApplySniperCampaignUpgradesEvent>,
    mut snipers: Query<
        (
            &mut Health,
            &mut UnitStats,
            &mut SniperArmor,
            &mut SniperEconomy,
            &mut SniperCampaignUpgrades,
        ),
        With<Sniper>,
    >,
) {
    for event in upgrades.read() {
        if let Ok((mut health, mut stats, mut armor, mut economy, mut applied)) =
            snipers.get_mut(event.sniper)
        {
            if applied.applied {
                continue;
            }

            health.max *= SNIPER_CAMPAIGN_HP_MULTIPLIER;
            health.current *= SNIPER_CAMPAIGN_HP_MULTIPLIER;
            stats.watch_range += SNIPER_CAMPAIGN_VIEW_RANGE_BONUS;
            stats.attack_range += SNIPER_CAMPAIGN_ATTACK_RANGE_BONUS;
            stats.attack_damage *= SNIPER_CAMPAIGN_DAMAGE_MULTIPLIER;
            armor.standard_reduction += SNIPER_CAMPAIGN_ARMOR_BONUS;
            economy.build_time_multiplier = SNIPER_CAMPAIGN_TRAINING_TIME_MULTIPLIER;
            economy.veteran_exp_multiplier = SNIPER_CAMPAIGN_VETERAN_EXP_MULTIPLIER;

            applied.applied = true;
        }
    }
}

fn sniper_barrel_carry_system(
    mut carry_events: EventReader<SetSniperBarrelCarryEvent>,
    mut snipers: Query<&mut SniperBarrelCarrier, With<Sniper>>,
) {
    for event in carry_events.read() {
        if let Ok(mut carry) = snipers.get_mut(event.sniper) {
            carry.carrying_explosive_barrel = event.carrying;
        }
    }
}

fn sniper_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    snipers: Query<
        (
            &Health,
            &UnitStats,
            &SniperArmor,
            &SniperAttackCooldown,
            &SniperVeteran,
            &SniperCampaignUpgrades,
            &SniperEconomy,
            &SniperBarrelCarrier,
            &SniperExperience,
            &SimPosition,
        ),
        With<Sniper>,
    >,
) {
    for (health, stats, armor, cooldown, veteran, upgrades, economy, barrel, exp, position) in
        &snipers
    {
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(armor.standard_reduction.to_bits() as u64);
        checksum.accumulate(cooldown.ticks_remaining as u64);
        checksum.accumulate(veteran.is_veteran as u64);
        checksum.accumulate(upgrades.applied as u64);

        checksum.accumulate(economy.gold as u64);
        checksum.accumulate(economy.food as u64);
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.iron as u64);
        checksum.accumulate(economy.wood as u64);
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.build_time_seconds as u64);
        checksum.accumulate(economy.build_time_multiplier.to_bits() as u64);
        checksum.accumulate(economy.veteran_exp_multiplier.to_bits() as u64);

        checksum.accumulate(barrel.carrying_explosive_barrel as u64);
        checksum.accumulate(exp.reward.to_bits() as u64);

        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
    }
}

pub struct SniperPlugin;

impl Plugin for SniperPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PromoteSniperEvent>()
            .add_event::<ApplySniperCampaignUpgradesEvent>()
            .add_event::<SetSniperBarrelCarryEvent>()
            .add_event::<SniperFiredEvent>()
            .add_systems(
                FixedUpdate,
                (
                    sniper_attack_system,
                    sniper_apply_damage_system,
                    sniper_promotion_system,
                    sniper_apply_campaign_upgrades_system,
                    sniper_barrel_carry_system,
                    sniper_checksum_system,
                )
                    .chain(),
            );
    }
}