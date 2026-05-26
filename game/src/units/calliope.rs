// Sources: vault/units/calliope.md, vault/game_mechanics/animation_canceling.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats};
use crate::units::Infected;

const CALLIOPE_BASE_HP: I32F32 = I32F32::lit("60");
const CALLIOPE_BASE_MS: I32F32 = I32F32::lit("3.2");
const CALLIOPE_BASE_AR: I32F32 = I32F32::lit("4.5");
const CALLIOPE_BASE_AS: I32F32 = I32F32::lit("5");
const CALLIOPE_BASE_AD: I32F32 = I32F32::lit("15");
const CALLIOPE_BASE_WR: I32F32 = I32F32::lit("7");
const CALLIOPE_BASE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");
const CALLIOPE_BASE_ATTACK_NOISE: I32F32 = I32F32::lit("1");

#[derive(Component)]
pub struct Calliope;

#[derive(Component, Clone, Copy)]
pub struct CalliopeCombatMods {
    pub armor_reduction: I32F32,
    pub attack_noise: I32F32,
}

impl Default for CalliopeCombatMods {
    fn default() -> Self {
        Self {
            armor_reduction: CALLIOPE_BASE_ARMOR_REDUCTION,
            attack_noise: CALLIOPE_BASE_ATTACK_NOISE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct CalliopeAttackCooldown {
    pub ticks_remaining: I32F32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MissionState {
    InProgress,
    Completed,
    Lost,
    Reloaded,
}

#[derive(Component, Clone, Copy)]
pub struct CalliopePerkState {
    pub perk_points_available: u8,
    pub aim_tier: u8,
    pub protection_tier: u8,
    pub strength_tier: u8,
    pub speed_tier: u8,
    pub dexterity_tier: u8,
    pub silent_tier: u8,
    pub quick_reflexes_tier: u8,
    pub improved_vision_tier: u8,
    pub mission_state: MissionState,
}

impl Default for CalliopePerkState {
    fn default() -> Self {
        Self {
            perk_points_available: 0,
            aim_tier: 0,
            protection_tier: 0,
            strength_tier: 0,
            speed_tier: 0,
            dexterity_tier: 0,
            silent_tier: 0,
            quick_reflexes_tier: 0,
            improved_vision_tier: 0,
            mission_state: MissionState::InProgress,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct CalliopeLoadSpeedMultiplier(pub I32F32);

#[derive(Event, Clone, Copy)]
pub struct CalliopeDamageEvent {
    pub target: Entity,
    pub raw_damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct CalliopePerkPointAwardEvent {
    pub target: Entity,
    pub first_time_scouting_mission_completed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CalliopePerk {
    AimI,
    AimII,
    AimIII,
    AimIV,
    ProtectionI,
    ProtectionII,
    ProtectionIII,
    StrengthI,
    StrengthII,
    StrengthIII,
    StrengthIV,
    SpeedI,
    SpeedII,
    SpeedIII,
    DexterityI,
    DexterityII,
    DexterityIII,
    SilentI,
    SilentII,
    QuickReflexesI,
    QuickReflexesII,
    ImprovedVisionI,
    ImprovedVisionII,
    ImprovedVisionIII,
}

#[derive(Event, Clone, Copy)]
pub struct AllocateCalliopePerkEvent {
    pub target: Entity,
    pub perk: CalliopePerk,
}

#[derive(Event, Clone, Copy)]
pub struct RefundCalliopePerksEvent {
    pub target: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct UpdateCalliopeMissionStateEvent {
    pub target: Entity,
    pub state: MissionState,
}

fn is_center_perk_unlocked(perks: &CalliopePerkState) -> bool {
    perks.aim_tier > 0 || perks.protection_tier > 0 || perks.strength_tier > 0 || perks.speed_tier > 0
}

fn can_unlock(perks: &CalliopePerkState, perk: CalliopePerk) -> bool {
    match perk {
        CalliopePerk::AimI
        | CalliopePerk::ProtectionI
        | CalliopePerk::StrengthI
        | CalliopePerk::SpeedI => true,
        _ => is_center_perk_unlocked(perks),
    }
}

pub fn calliope_attack_tick_system(
    sim_hz: Res<SimHz>,
    mut attackers: Query<
        (
            Entity,
            &SimPosition,
            &mut CalliopeAttackCooldown,
            &UnitStats,
            &CalliopeLoadSpeedMultiplier,
        ),
        With<Calliope>,
    >,
    infected_positions: Query<(Entity, &SimPosition), With<Infected>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
) {
    for (calliope_entity, calliope_pos, mut cooldown, stats, load_mult) in &mut attackers {
        if cooldown.ticks_remaining > I32F32::ZERO {
            cooldown.ticks_remaining -= I32F32::ONE;
            continue;
        }

        let attack_range_sq = stats.attack_range * stats.attack_range;
        let mut best_target: Option<(Entity, SimPosition, I32F32)> = None;

        for (infected_entity, infected_pos) in &infected_positions {
            let dx = infected_pos.x - calliope_pos.x;
            let dy = infected_pos.y - calliope_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > attack_range_sq {
                continue;
            }

            match best_target {
                None => {
                    best_target = Some((infected_entity, *infected_pos, dist_sq));
                }
                Some((_, best_pos, best_dist_sq)) => {
                    if dist_sq < best_dist_sq
                        || (dist_sq == best_dist_sq
                            && (infected_pos.x < best_pos.x
                                || (infected_pos.x == best_pos.x && infected_pos.y < best_pos.y)))
                    {
                        best_target = Some((infected_entity, *infected_pos, dist_sq));
                    }
                }
            }
        }

        let Some((target, _, _)) = best_target else {
            continue;
        };

        outgoing_damage.send(IncomingDamageEvent {
            target,
            raw_amount: stats.attack_damage,
            damage_type: crate::sim::DamageType::Standard,
            source: calliope_entity,
        });

        let load_speed = if load_mult.0 <= I32F32::ZERO {
            I32F32::ONE
        } else {
            load_mult.0
        };
        cooldown.ticks_remaining = (sim_hz.0 / stats.attack_speed) / load_speed;
    }
}

pub fn calliope_receive_damage_system(
    mut events: EventReader<CalliopeDamageEvent>,
    mut units: Query<(&mut Health, &CalliopeCombatMods), With<Calliope>>,
) {
    for ev in events.read() {
        if let Ok((mut hp, mods)) = units.get_mut(ev.target) {
            let applied = ev.raw_damage * (I32F32::ONE - mods.armor_reduction);
            hp.current = (hp.current - applied).max(I32F32::ZERO);
        }
    }
}

pub fn calliope_perk_point_award_system(
    mut events: EventReader<CalliopePerkPointAwardEvent>,
    mut units: Query<&mut CalliopePerkState, With<Calliope>>,
) {
    for ev in events.read() {
        if !ev.first_time_scouting_mission_completed {
            continue;
        }
        if let Ok(mut perks) = units.get_mut(ev.target) {
            perks.perk_points_available = perks.perk_points_available.saturating_add(1);
        }
    }
}

pub fn calliope_update_mission_state_system(
    mut events: EventReader<UpdateCalliopeMissionStateEvent>,
    mut units: Query<&mut CalliopePerkState, With<Calliope>>,
) {
    for ev in events.read() {
        if let Ok(mut perks) = units.get_mut(ev.target) {
            perks.mission_state = ev.state;
        }
    }
}

pub fn calliope_allocate_perk_system(
    mut events: EventReader<AllocateCalliopePerkEvent>,
    mut units: Query<&mut CalliopePerkState, With<Calliope>>,
) {
    for ev in events.read() {
        if let Ok(mut perks) = units.get_mut(ev.target) {
            if perks.perk_points_available == 0 || !can_unlock(&perks, ev.perk) {
                continue;
            }

            let allocated = match ev.perk {
                CalliopePerk::AimI if perks.aim_tier == 0 => {
                    perks.aim_tier = 1;
                    true
                }
                CalliopePerk::AimII if perks.aim_tier == 1 => {
                    perks.aim_tier = 2;
                    true
                }
                CalliopePerk::AimIII if perks.aim_tier == 2 => {
                    perks.aim_tier = 3;
                    true
                }
                CalliopePerk::AimIV if perks.aim_tier == 3 => {
                    perks.aim_tier = 4;
                    true
                }
                CalliopePerk::ProtectionI if perks.protection_tier == 0 => {
                    perks.protection_tier = 1;
                    true
                }
                CalliopePerk::ProtectionII if perks.protection_tier == 1 => {
                    perks.protection_tier = 2;
                    true
                }
                CalliopePerk::ProtectionIII if perks.protection_tier == 2 => {
                    perks.protection_tier = 3;
                    true
                }
                CalliopePerk::StrengthI if perks.strength_tier == 0 => {
                    perks.strength_tier = 1;
                    true
                }
                CalliopePerk::StrengthII if perks.strength_tier == 1 => {
                    perks.strength_tier = 2;
                    true
                }
                CalliopePerk::StrengthIII if perks.strength_tier == 2 => {
                    perks.strength_tier = 3;
                    true
                }
                CalliopePerk::StrengthIV if perks.strength_tier == 3 => {
                    perks.strength_tier = 4;
                    true
                }
                CalliopePerk::SpeedI if perks.speed_tier == 0 => {
                    perks.speed_tier = 1;
                    true
                }
                CalliopePerk::SpeedII if perks.speed_tier == 1 => {
                    perks.speed_tier = 2;
                    true
                }
                CalliopePerk::SpeedIII if perks.speed_tier == 2 => {
                    perks.speed_tier = 3;
                    true
                }
                CalliopePerk::DexterityI if perks.dexterity_tier == 0 => {
                    perks.dexterity_tier = 1;
                    true
                }
                CalliopePerk::DexterityII if perks.dexterity_tier == 1 => {
                    perks.dexterity_tier = 2;
                    true
                }
                CalliopePerk::DexterityIII if perks.dexterity_tier == 2 => {
                    perks.dexterity_tier = 3;
                    true
                }
                CalliopePerk::SilentI if perks.silent_tier == 0 => {
                    perks.silent_tier = 1;
                    true
                }
                CalliopePerk::SilentII if perks.silent_tier == 1 => {
                    perks.silent_tier = 2;
                    true
                }
                CalliopePerk::QuickReflexesI if perks.quick_reflexes_tier == 0 => {
                    perks.quick_reflexes_tier = 1;
                    true
                }
                CalliopePerk::QuickReflexesII if perks.quick_reflexes_tier == 1 => {
                    perks.quick_reflexes_tier = 2;
                    true
                }
                CalliopePerk::ImprovedVisionI if perks.improved_vision_tier == 0 => {
                    perks.improved_vision_tier = 1;
                    true
                }
                CalliopePerk::ImprovedVisionII if perks.improved_vision_tier == 1 => {
                    perks.improved_vision_tier = 2;
                    true
                }
                CalliopePerk::ImprovedVisionIII if perks.improved_vision_tier == 2 => {
                    perks.improved_vision_tier = 3;
                    true
                }
                _ => false,
            };

            if allocated {
                perks.perk_points_available = perks.perk_points_available.saturating_sub(1);
            }
        }
    }
}

pub fn calliope_refund_perks_system(
    mut events: EventReader<RefundCalliopePerksEvent>,
    mut units: Query<(&mut CalliopePerkState, &mut UnitStats, &mut CalliopeCombatMods, &mut Health), With<Calliope>>,
) {
    for ev in events.read() {
        if let Ok((mut perks, mut stats, mut combat_mods, mut hp)) = units.get_mut(ev.target) {
            let can_refund = matches!(perks.mission_state, MissionState::InProgress | MissionState::Lost);
            if !can_refund {
                continue;
            }

            let spent_points = perks.aim_tier
                + perks.protection_tier
                + perks.strength_tier
                + perks.speed_tier
                + perks.dexterity_tier
                + perks.silent_tier
                + perks.quick_reflexes_tier
                + perks.improved_vision_tier;

            perks.perk_points_available = perks.perk_points_available.saturating_add(spent_points);
            perks.aim_tier = 0;
            perks.protection_tier = 0;
            perks.strength_tier = 0;
            perks.speed_tier = 0;
            perks.dexterity_tier = 0;
            perks.silent_tier = 0;
            perks.quick_reflexes_tier = 0;
            perks.improved_vision_tier = 0;

            *stats = UnitStats {
                move_speed: CALLIOPE_BASE_MS,
                attack_range: CALLIOPE_BASE_AR,
                attack_damage: CALLIOPE_BASE_AD,
                attack_speed: CALLIOPE_BASE_AS,
                watch_range: CALLIOPE_BASE_WR,
            };
            *combat_mods = CalliopeCombatMods::default();
            hp.max = CALLIOPE_BASE_HP;
            if hp.current > hp.max {
                hp.current = hp.max;
            }
        }
    }
}

pub fn calliope_apply_perks_system(
    mut units: Query<
        (
            &CalliopePerkState,
            &mut UnitStats,
            &mut CalliopeCombatMods,
            &mut Health,
            &mut CalliopeLoadSpeedMultiplier,
        ),
        With<Calliope>,
    >,
) {
    for (perks, mut stats, mut combat_mods, mut hp, mut load_mult) in &mut units {
        let mut damage_mult = I32F32::ONE;
        if perks.aim_tier >= 1 {
            damage_mult += I32F32::lit("0.30");
        }
        if perks.aim_tier >= 2 {
            damage_mult += I32F32::lit("0.30");
        }
        if perks.aim_tier >= 3 {
            damage_mult += I32F32::lit("0.40");
        }
        if perks.aim_tier >= 4 {
            damage_mult += I32F32::lit("0.50");
        }

        let mut hp_mult = I32F32::ONE;
        if perks.strength_tier >= 1 {
            hp_mult += I32F32::lit("0.30");
        }
        if perks.strength_tier >= 2 {
            hp_mult += I32F32::lit("0.30");
        }
        if perks.strength_tier >= 3 {
            hp_mult += I32F32::lit("0.40");
        }
        if perks.strength_tier >= 4 {
            hp_mult += I32F32::lit("0.50");
        }

        let mut as_mult = I32F32::ONE;
        if perks.dexterity_tier >= 1 {
            as_mult += I32F32::lit("0.20");
        }
        if perks.dexterity_tier >= 2 {
            as_mult += I32F32::lit("0.20");
        }
        if perks.dexterity_tier >= 3 {
            as_mult += I32F32::lit("0.25");
        }

        let mut ls_mult = I32F32::ONE;
        if perks.quick_reflexes_tier >= 1 {
            ls_mult += I32F32::lit("0.40");
        }
        if perks.quick_reflexes_tier >= 2 {
            ls_mult += I32F32::lit("0.40");
        }

        let mut armor = CALLIOPE_BASE_ARMOR_REDUCTION;
        if perks.protection_tier >= 1 {
            armor += I32F32::lit("0.15");
        }
        if perks.protection_tier >= 2 {
            armor += I32F32::lit("0.15");
        }
        if perks.protection_tier >= 3 {
            armor += I32F32::lit("0.15");
        }
        if armor > I32F32::lit("0.95") {
            armor = I32F32::lit("0.95");
        }

        let mut move_mult = I32F32::ONE;
        if perks.speed_tier >= 1 {
            move_mult += I32F32::lit("0.20");
        }
        if perks.speed_tier >= 2 {
            move_mult += I32F32::lit("0.20");
        }
        if perks.speed_tier >= 3 {
            move_mult += I32F32::lit("0.30");
        }

        let range_bonus = I32F32::from_num(perks.improved_vision_tier);

        let mut noise_mult = I32F32::ONE;
        if perks.silent_tier >= 1 {
            noise_mult -= I32F32::lit("0.40");
        }
        if perks.silent_tier >= 2 {
            noise_mult -= I32F32::lit("0.40");
        }
        if noise_mult < I32F32::ZERO {
            noise_mult = I32F32::ZERO;
        }

        stats.move_speed = CALLIOPE_BASE_MS * move_mult;
        stats.attack_range = CALLIOPE_BASE_AR + range_bonus;
        stats.attack_speed = CALLIOPE_BASE_AS * as_mult;
        stats.attack_damage = CALLIOPE_BASE_AD * damage_mult;
        stats.watch_range = CALLIOPE_BASE_WR + range_bonus;

        combat_mods.armor_reduction = armor;
        combat_mods.attack_noise = CALLIOPE_BASE_ATTACK_NOISE * noise_mult;

        hp.max = CALLIOPE_BASE_HP * hp_mult;
        if hp.current > hp.max {
            hp.current = hp.max;
        }

        load_mult.0 = ls_mult;
    }
}

pub fn calliope_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &Health,
            &UnitStats,
            &CalliopeCombatMods,
            &CalliopeAttackCooldown,
            &CalliopePerkState,
            &CalliopeLoadSpeedMultiplier,
        ),
        With<Calliope>,
    >,
) {
    for (hp, stats, combat_mods, cooldown, perks, load_mult) in &units {
        checksum.accumulate(hp.current.to_bits() as u64);
        checksum.accumulate(hp.max.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(combat_mods.armor_reduction.to_bits() as u64);
        checksum.accumulate(combat_mods.attack_noise.to_bits() as u64);

        checksum.accumulate(cooldown.ticks_remaining.to_bits() as u64);
        checksum.accumulate(load_mult.0.to_bits() as u64);

        checksum.accumulate(perks.perk_points_available as u64);
        checksum.accumulate(perks.aim_tier as u64);
        checksum.accumulate(perks.protection_tier as u64);
        checksum.accumulate(perks.strength_tier as u64);
        checksum.accumulate(perks.speed_tier as u64);
        checksum.accumulate(perks.dexterity_tier as u64);
        checksum.accumulate(perks.silent_tier as u64);
        checksum.accumulate(perks.quick_reflexes_tier as u64);
        checksum.accumulate(perks.improved_vision_tier as u64);
        let mission_state_bits = match perks.mission_state {
            MissionState::InProgress => 0_u64,
            MissionState::Completed => 1_u64,
            MissionState::Lost => 2_u64,
            MissionState::Reloaded => 3_u64,
        };
        checksum.accumulate(mission_state_bits);
    }
}

pub struct CalliopePlugin;

impl Plugin for CalliopePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CalliopeDamageEvent>()
            .add_event::<CalliopePerkPointAwardEvent>()
            .add_event::<AllocateCalliopePerkEvent>()
            .add_event::<RefundCalliopePerksEvent>()
            .add_event::<UpdateCalliopeMissionStateEvent>()
            .add_systems(
                FixedUpdate,
                (
                    calliope_attack_tick_system,
                    calliope_receive_damage_system,
                    calliope_perk_point_award_system,
                    calliope_update_mission_state_system,
                    calliope_allocate_perk_system,
                    calliope_refund_perks_system,
                    calliope_apply_perks_system,
                    calliope_checksum_system,
                )
                    .chain(),
            );
    }
}