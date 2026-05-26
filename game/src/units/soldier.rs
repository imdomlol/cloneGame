// Sources: vault/units/soldier.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState,
    SimHz, SimPosition, UnitStats,
};

pub const SOLDIER_GOLD_COST: u32 = 240;
pub const SOLDIER_BUILD_TIME_SECONDS: u32 = 30;
pub const SOLDIER_FOOD_COST: u32 = 1;
pub const SOLDIER_WORKER_COST: u32 = 1;
pub const SOLDIER_IRON_COST: u32 = 2;
pub const SOLDIER_MAINTENANCE_GOLD: u32 = 3;

pub const SOLDIER_HP: I32F32 = I32F32::lit("120");
pub const SOLDIER_MOVE_SPEED: I32F32 = I32F32::lit("2.4");
pub const SOLDIER_ATTACK_RANGE: I32F32 = I32F32::lit("5");
pub const SOLDIER_ATTACK_SPEED: I32F32 = I32F32::lit("2");
pub const SOLDIER_ATTACK_DAMAGE: I32F32 = I32F32::lit("16");
pub const SOLDIER_WATCH_RANGE: I32F32 = I32F32::lit("6");
pub const SOLDIER_NOISE_PER_ATTACK: I32F32 = I32F32::lit("3");
pub const SOLDIER_EXP_REWARD: I32F32 = I32F32::lit("90");

pub const SOLDIER_STANDARD_REDUCTION: I32F32 = I32F32::lit("0.4");
pub const SOLDIER_FIRE_REDUCTION: I32F32 = I32F32::lit("0.5");
pub const SOLDIER_VENOM_REDUCTION: I32F32 = I32F32::lit("0.5");

pub const SOLDIER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("5.5");
pub const SOLDIER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("2.5");
pub const SOLDIER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("26");

pub const SOLDIER_EXP_GAIN_BASIC: I32F32 = I32F32::lit("0.0134");
pub const SOLDIER_EXP_GAIN_MEDIUM: I32F32 = I32F32::lit("0.0268");
pub const SOLDIER_EXP_GAIN_HARD: I32F32 = I32F32::lit("0.067");
pub const SOLDIER_EXP_GAIN_ELITE: I32F32 = I32F32::lit("0.134");

#[derive(Component, Default)]
pub struct Soldier;

#[derive(Component, Clone, Copy)]
pub struct SoldierArmor {
    pub standard_reduction: I32F32,
    pub fire_reduction: I32F32,
    pub venom_reduction: I32F32,
}

impl Default for SoldierArmor {
    fn default() -> Self {
        Self {
            standard_reduction: SOLDIER_STANDARD_REDUCTION,
            fire_reduction: SOLDIER_FIRE_REDUCTION,
            venom_reduction: SOLDIER_VENOM_REDUCTION,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct SoldierAttackCooldown {
    pub ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct SoldierVeteran {
    pub is_veteran: bool,
}

#[derive(Component, Clone, Copy)]
pub struct SoldierExperience {
    pub accumulated: I32F32,
}

impl Default for SoldierExperience {
    fn default() -> Self {
        Self {
            accumulated: I32F32::ZERO,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct SoldierProductionCost {
    pub gold: u32,
    pub food: u32,
    pub workers: u32,
    pub iron: u32,
    pub maintenance_gold: u32,
    pub build_time_seconds: u32,
}

impl Default for SoldierProductionCost {
    fn default() -> Self {
        Self {
            gold: SOLDIER_GOLD_COST,
            food: SOLDIER_FOOD_COST,
            workers: SOLDIER_WORKER_COST,
            iron: SOLDIER_IRON_COST,
            maintenance_gold: SOLDIER_MAINTENANCE_GOLD,
            build_time_seconds: SOLDIER_BUILD_TIME_SECONDS,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SoldierAttackEvent {
    pub soldier: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct PromoteSoldierEvent {
    pub soldier: Entity,
}

pub fn soldier_base_health() -> Health {
    Health::full(SOLDIER_HP)
}

pub fn soldier_base_stats() -> UnitStats {
    UnitStats {
        move_speed: SOLDIER_MOVE_SPEED,
        attack_range: SOLDIER_ATTACK_RANGE,
        attack_damage: SOLDIER_ATTACK_DAMAGE,
        attack_speed: SOLDIER_ATTACK_SPEED,
        watch_range: SOLDIER_WATCH_RANGE,
    }
}

pub fn spawn_soldier(commands: &mut Commands, position: SimPosition) -> Entity {
    commands
        .spawn((
            Soldier,
            position,
            soldier_base_health(),
            soldier_base_stats(),
            SoldierArmor::default(),
            SoldierAttackCooldown::default(),
            SoldierVeteran::default(),
            SoldierExperience::default(),
            SoldierProductionCost::default(),
        ))
        .id()
}

fn soldier_attack_system(
    sim_hz: Res<SimHz>,
    mut soldiers: Query<(Entity, &SimPosition, &UnitStats, &mut SoldierAttackCooldown), With<Soldier>>,
    mut attack_events: EventWriter<SoldierAttackEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
) {
    for (entity, position, stats, mut cooldown) in &mut soldiers {
        if cooldown.ticks_remaining > 0 {
            cooldown.ticks_remaining -= 1;
            continue;
        }

        attack_events.send(SoldierAttackEvent { soldier: entity });
        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: SOLDIER_NOISE_PER_ATTACK,
        });

        let mut ticks = (sim_hz.0 / stats.attack_speed).to_num::<u32>();
        if ticks == 0 {
            ticks = 1;
        }
        cooldown.ticks_remaining = ticks;
    }
}

fn soldier_apply_damage_system(
    mut incoming_damage: EventReader<IncomingDamageEvent>,
    mut soldiers: Query<(&mut Health, &SoldierArmor), With<Soldier>>,
) {
    for event in incoming_damage.read() {
        if let Ok((mut health, armor)) = soldiers.get_mut(event.target) {
            let reduction = match event.damage_type {
                DamageType::Standard => armor.standard_reduction,
                DamageType::Fire => armor.fire_reduction,
                DamageType::Venom => armor.venom_reduction,
            };
            let effective_damage = event.raw_amount * (I32F32::ONE - reduction);
            health.current = (health.current - effective_damage).max(I32F32::ZERO);
        }
    }
}

fn soldier_experience_system(
    mut kills: EventReader<EntityKilledEvent>,
    mut soldiers: Query<&mut SoldierExperience, With<Soldier>>,
) {
    for event in kills.read() {
        if let Ok(mut experience) = soldiers.get_mut(event.killer) {
            let gain_rate = match event.difficulty_tier {
                0 => SOLDIER_EXP_GAIN_BASIC,
                1 => SOLDIER_EXP_GAIN_MEDIUM,
                2 => SOLDIER_EXP_GAIN_HARD,
                _ => SOLDIER_EXP_GAIN_ELITE,
            };
            experience.accumulated += event.exp_reward * gain_rate;
        }
    }
}

fn soldier_promotion_system(
    mut promotions: EventReader<PromoteSoldierEvent>,
    mut soldiers: Query<(&mut UnitStats, &mut SoldierVeteran), With<Soldier>>,
) {
    for event in promotions.read() {
        if let Ok((mut stats, mut veteran)) = soldiers.get_mut(event.soldier) {
            if veteran.is_veteran {
                continue;
            }
            veteran.is_veteran = true;
            stats.attack_range = SOLDIER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = SOLDIER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = SOLDIER_VETERAN_ATTACK_DAMAGE;
        }
    }
}

fn soldier_auto_promote_system(
    mut soldiers: Query<(&SoldierExperience, &mut UnitStats, &mut SoldierVeteran), With<Soldier>>,
) {
    for (experience, mut stats, mut veteran) in &mut soldiers {
        if veteran.is_veteran || experience.accumulated < SOLDIER_EXP_REWARD {
            continue;
        }

        veteran.is_veteran = true;
        stats.attack_range = SOLDIER_VETERAN_ATTACK_RANGE;
        stats.attack_speed = SOLDIER_VETERAN_ATTACK_SPEED;
        stats.attack_damage = SOLDIER_VETERAN_ATTACK_DAMAGE;
    }
}

fn soldier_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    soldiers: Query<
        (
            &Health,
            &UnitStats,
            &SoldierArmor,
            &SoldierAttackCooldown,
            &SoldierVeteran,
            &SoldierExperience,
            &SoldierProductionCost,
            &SimPosition,
        ),
        With<Soldier>,
    >,
) {
    for (health, stats, armor, cooldown, veteran, experience, production_cost, position) in &soldiers {
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(armor.standard_reduction.to_bits() as u64);
        checksum.accumulate(armor.fire_reduction.to_bits() as u64);
        checksum.accumulate(armor.venom_reduction.to_bits() as u64);

        checksum.accumulate(cooldown.ticks_remaining as u64);
        checksum.accumulate(veteran.is_veteran as u64);
        checksum.accumulate(experience.accumulated.to_bits() as u64);

        checksum.accumulate(production_cost.gold as u64);
        checksum.accumulate(production_cost.food as u64);
        checksum.accumulate(production_cost.workers as u64);
        checksum.accumulate(production_cost.iron as u64);
        checksum.accumulate(production_cost.maintenance_gold as u64);
        checksum.accumulate(production_cost.build_time_seconds as u64);

        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
    }
}

pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoldierAttackEvent>()
            .add_event::<PromoteSoldierEvent>()
            .add_systems(
                FixedUpdate,
                (
                    soldier_attack_system,
                    soldier_apply_damage_system,
                    soldier_experience_system,
                    soldier_promotion_system,
                    soldier_auto_promote_system,
                    soldier_checksum_system,
                )
                    .chain(),
            );
    }
}