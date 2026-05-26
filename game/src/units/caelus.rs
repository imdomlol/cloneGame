// Sources: vault/units/caelus.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};
use crate::units::Infected;

const CAELUS_BASE_HP: I32F32 = I32F32::lit("120");
const CAELUS_BASE_MS: I32F32 = I32F32::lit("2");
const CAELUS_BASE_AR: I32F32 = I32F32::lit("6");
const CAELUS_BASE_AS: I32F32 = I32F32::lit("0.71");
const CAELUS_BASE_AD: I32F32 = I32F32::lit("50");
const CAELUS_BASE_WR: I32F32 = I32F32::lit("7");
const CAELUS_BASE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");
const CAELUS_BASE_EFFECT_RADIUS: I32F32 = I32F32::lit("1.0");
const CAELUS_REISSUE_ATTACK_FACTOR: I32F32 = I32F32::lit("0.85");

#[derive(Component, Default)]
pub struct Caelus;

#[derive(Component, Clone, Copy)]
pub struct CaelusCombatMods {
    pub armor_reduction: I32F32,
    pub effect_radius: I32F32,
}

impl Default for CaelusCombatMods {
    fn default() -> Self {
        Self {
            armor_reduction: CAELUS_BASE_ARMOR_REDUCTION,
            effect_radius: CAELUS_BASE_EFFECT_RADIUS,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct CaelusAttackCooldown {
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
pub struct CaelusPerkState {
    pub perk_points_available: u8,
    pub aim_tier: u8,
    pub strength_tier: u8,
    pub dexterity_tier: u8,
    pub quick_reflexes_tier: u8,
    pub protection_tier: u8,
    pub improved_vision_i: bool,
    pub improved_vision_iii: bool,
    pub speed_tier: u8,
    pub effect_radius_tier: u8,
    pub mission_state: MissionState,
}

impl Default for CaelusPerkState {
    fn default() -> Self {
        Self {
            perk_points_available: 0,
            aim_tier: 0,
            strength_tier: 0,
            dexterity_tier: 0,
            quick_reflexes_tier: 0,
            protection_tier: 0,
            improved_vision_i: false,
            improved_vision_iii: false,
            speed_tier: 0,
            effect_radius_tier: 0,
            mission_state: MissionState::InProgress,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct CaelusLoadSpeedMultiplier(pub I32F32);

#[derive(Event, Clone, Copy)]
pub struct CaelusDamageEvent {
    pub target: Entity,
    pub raw_damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct CaelusPerkPointAwardEvent {
    pub target: Entity,
    pub first_time_scouting_mission_completed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CaelusPerk {
    AimI,
    AimII,
    AimIII,
    StrengthI,
    StrengthII,
    StrengthIII,
    StrengthIV,
    DexterityI,
    DexterityII,
    DexterityIII,
    QuickReflexesI,
    QuickReflexesII,
    QuickReflexesIII,
    ProtectionI,
    ProtectionII,
    ProtectionIII,
    ImprovedVisionI,
    ImprovedVisionIII,
    SpeedI,
    SpeedII,
    SpeedIII,
    EffectRadiusI,
    EffectRadiusII,
}

#[derive(Event, Clone, Copy)]
pub struct AllocateCaelusPerkEvent {
    pub target: Entity,
    pub perk: CaelusPerk,
}

#[derive(Event, Clone, Copy)]
pub struct RefundCaelusPerksEvent {
    pub target: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct UpdateCaelusMissionStateEvent {
    pub target: Entity,
    pub state: MissionState,
}

#[derive(Event, Clone, Copy)]
pub struct ReissueCaelusAttackCommandEvent {
    pub target: Entity,
}

pub fn caelus_base_health() -> Health {
    Health::full(CAELUS_BASE_HP)
}

pub fn caelus_base_stats() -> UnitStats {
    UnitStats {
        move_speed: CAELUS_BASE_MS,
        attack_range: CAELUS_BASE_AR,
        attack_damage: CAELUS_BASE_AD,
        attack_speed: CAELUS_BASE_AS,
        watch_range: CAELUS_BASE_WR,
    }
}

pub fn spawn_caelus(commands: &mut Commands, position: SimPosition) -> Entity {
    commands
        .spawn((
            Caelus,
            position,
            caelus_base_health(),
            caelus_base_stats(),
            CaelusCombatMods::default(),
            CaelusAttackCooldown::default(),
            CaelusPerkState::default(),
            CaelusLoadSpeedMultiplier::default(),
        ))
        .id()
}

fn is_center_perk_unlocked(perks: &CaelusPerkState) -> bool {
    perks.aim_tier > 0 || perks.strength_tier > 0 || perks.protection_tier > 0 || perks.speed_tier > 0
}

fn can_unlock(perks: &CaelusPerkState, perk: CaelusPerk) -> bool {
    match perk {
        CaelusPerk::AimI | CaelusPerk::StrengthI | CaelusPerk::ProtectionI | CaelusPerk::SpeedI => {
            true
        }
        _ => is_center_perk_unlocked(perks),
    }
}

pub fn caelus_attack_tick_system(
    sim_hz: Res<SimHz>,
    mut attackers: Query<
        (
            Entity,
            &SimPosition,
            &mut CaelusAttackCooldown,
            &UnitStats,
            &CaelusCombatMods,
            &CaelusLoadSpeedMultiplier,
        ),
        With<Caelus>,
    >,
    infected_positions: Query<(Entity, &SimPosition), With<Infected>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
) {
    for (caelus_entity, caelus_pos, mut cooldown, stats, mods, load_mult) in &mut attackers {
        if cooldown.ticks_remaining > I32F32::ZERO {
            cooldown.ticks_remaining -= I32F32::ONE;
            continue;
        }

        let attack_range_sq = stats.attack_range * stats.attack_range;
        let mut best_target: Option<(Entity, SimPosition, I32F32)> = None;

        for (infected_entity, infected_pos) in &infected_positions {
            let dx = infected_pos.x - caelus_pos.x;
            let dy = infected_pos.y - caelus_pos.y;
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

        let Some((_, impact_pos, _)) = best_target else {
            continue;
        };

        let effect_radius_sq = mods.effect_radius * mods.effect_radius;
        for (infected_entity, infected_pos) in &infected_positions {
            let dx = infected_pos.x - impact_pos.x;
            let dy = infected_pos.y - impact_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= effect_radius_sq {
                outgoing_damage.send(IncomingDamageEvent {
                    target: infected_entity,
                    raw_amount: stats.attack_damage,
                    damage_type: DamageType::Standard,
                    source: caelus_entity,
                });
            }
        }

        let load_speed = if load_mult.0 <= I32F32::ZERO {
            I32F32::ONE
        } else {
            load_mult.0
        };

        cooldown.ticks_remaining = (sim_hz.0 / stats.attack_speed) / load_speed;
    }
}

pub fn caelus_reissue_attack_command_system(
    mut events: EventReader<ReissueCaelusAttackCommandEvent>,
    mut units: Query<&mut CaelusAttackCooldown, With<Caelus>>,
) {
    for ev in events.read() {
        if let Ok(mut cooldown) = units.get_mut(ev.target) {
            if cooldown.ticks_remaining > I32F32::ZERO {
                cooldown.ticks_remaining *= CAELUS_REISSUE_ATTACK_FACTOR;
            }
        }
    }
}

pub fn caelus_receive_damage_system(
    mut events: EventReader<CaelusDamageEvent>,
    mut units: Query<(&mut Health, &CaelusCombatMods), With<Caelus>>,
) {
    for ev in events.read() {
        if let Ok((mut hp, mods)) = units.get_mut(ev.target) {
            let applied = ev.raw_damage * (I32F32::ONE - mods.armor_reduction);
            hp.current = (hp.current - applied).max(I32F32::ZERO);
        }
    }
}

pub fn caelus_perk_point_award_system(
    mut events: EventReader<CaelusPerkPointAwardEvent>,
    mut units: Query<&mut CaelusPerkState, With<Caelus>>,
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

pub fn caelus_update_mission_state_system(
    mut events: EventReader<UpdateCaelusMissionStateEvent>,
    mut units: Query<&mut CaelusPerkState, With<Caelus>>,
) {
    for ev in events.read() {
        if let Ok(mut perks) = units.get_mut(ev.target) {
            perks.mission_state = ev.state;
        }
    }
}

pub fn caelus_allocate_perk_system(
    mut events: EventReader<AllocateCaelusPerkEvent>,
    mut units: Query<&mut CaelusPerkState, With<Caelus>>,
) {
    for ev in events.read() {
        if let Ok(mut perks) = units.get_mut(ev.target) {
            if perks.perk_points_available == 0 || !can_unlock(&perks, ev.perk) {
                continue;
            }

            let allocated = match ev.perk {
                CaelusPerk::AimI if perks.aim_tier == 0 => {
                    perks.aim_tier = 1;
                    true
                }
                CaelusPerk::AimII if perks.aim_tier == 1 => {
                    perks.aim_tier = 2;
                    true
                }
                CaelusPerk::AimIII if perks.aim_tier == 2 => {
                    perks.aim_tier = 3;
                    true
                }
                CaelusPerk::StrengthI if perks.strength_tier == 0 => {
                    perks.strength_tier = 1;
                    true
                }
                CaelusPerk::StrengthII if perks.strength_tier == 1 => {
                    perks.strength_tier = 2;
                    true
                }
                CaelusPerk::StrengthIII if perks.strength_tier == 2 => {
                    perks.strength_tier = 3;
                    true
                }
                CaelusPerk::StrengthIV if perks.strength_tier == 3 => {
                    perks.strength_tier = 4;
                    true
                }
                CaelusPerk::DexterityI if perks.dexterity_tier == 0 => {
                    perks.dexterity_tier = 1;
                    true
                }
                CaelusPerk::DexterityII if perks.dexterity_tier == 1 => {
                    perks.dexterity_tier = 2;
                    true
                }
                CaelusPerk::DexterityIII if perks.dexterity_tier == 2 => {
                    perks.dexterity_tier = 3;
                    true
                }
                CaelusPerk::QuickReflexesI if perks.quick_reflexes_tier == 0 => {
                    perks.quick_reflexes_tier = 1;
                    true
                }
                CaelusPerk::QuickReflexesII if perks.quick_reflexes_tier == 1 => {
                    perks.quick_reflexes_tier = 2;
                    true
                }
                CaelusPerk::QuickReflexesIII if perks.quick_reflexes_tier == 2 => {
                    perks.quick_reflexes_tier = 3;
                    true
                }
                CaelusPerk::ProtectionI if perks.protection_tier == 0 => {
                    perks.protection_tier = 1;
                    true
                }
                CaelusPerk::ProtectionII if perks.protection_tier == 1 => {
                    perks.protection_tier = 2;
                    true
                }
                CaelusPerk::ProtectionIII if perks.protection_tier == 2 => {
                    perks.protection_tier = 3;
                    true
                }
                CaelusPerk::ImprovedVisionI if !perks.improved_vision_i => {
                    perks.improved_vision_i = true;
                    true
                }
                CaelusPerk::ImprovedVisionIII if !perks.improved_vision_iii => {
                    perks.improved_vision_iii = true;
                    true
                }
                CaelusPerk::SpeedI if perks.speed_tier == 0 => {
                    perks.speed_tier = 1;
                    true
                }
                CaelusPerk::SpeedII if perks.speed_tier == 1 => {
                    perks.speed_tier = 2;
                    true
                }
                CaelusPerk::SpeedIII if perks.speed_tier == 2 => {
                    perks.speed_tier = 3;
                    true
                }
                CaelusPerk::EffectRadiusI if perks.effect_radius_tier == 0 => {
                    perks.effect_radius_tier = 1;
                    true
                }
                CaelusPerk::EffectRadiusII if perks.effect_radius_tier == 1 => {
                    perks.effect_radius_tier = 2;
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

pub fn caelus_refund_perks_system(
    mut events: EventReader<RefundCaelusPerksEvent>,
    mut units: Query<
        (
            &mut CaelusPerkState,
            &mut UnitStats,
            &mut CaelusCombatMods,
            &mut Health,
            &mut CaelusLoadSpeedMultiplier,
        ),
        With<Caelus>,
    >,
) {
    for ev in events.read() {
        if let Ok((mut perks, mut stats, mut combat_mods, mut hp, mut load_mult)) = units.get_mut(ev.target) {
            let can_refund = matches!(perks.mission_state, MissionState::InProgress | MissionState::Lost);
            if !can_refund {
                continue;
            }

            let spent_points = perks.aim_tier
                + perks.strength_tier
                + perks.dexterity_tier
                + perks.quick_reflexes_tier
                + perks.protection_tier
                + perks.speed_tier
                + perks.effect_radius_tier
                + u8::from(perks.improved_vision_i)
                + u8::from(perks.improved_vision_iii);

            perks.perk_points_available = perks.perk_points_available.saturating_add(spent_points);
            perks.aim_tier = 0;
            perks.strength_tier = 0;
            perks.dexterity_tier = 0;
            perks.quick_reflexes_tier = 0;
            perks.protection_tier = 0;
            perks.improved_vision_i = false;
            perks.improved_vision_iii = false;
            perks.speed_tier = 0;
            perks.effect_radius_tier = 0;

            *stats = caelus_base_stats();
            *combat_mods = CaelusCombatMods::default();
            hp.max = CAELUS_BASE_HP;
            if hp.current > hp.max {
                hp.current = hp.max;
            }
            load_mult.0 = I32F32::ONE;
        }
    }
}

pub fn caelus_apply_perks_system(
    mut units: Query<
        (
            &CaelusPerkState,
            &mut UnitStats,
            &mut CaelusCombatMods,
            &mut Health,
            &mut CaelusLoadSpeedMultiplier,
        ),
        With<Caelus>,
    >,
) {
    for (perks, mut stats, mut combat_mods, mut hp, mut load_mult) in &mut units {
        let mut damage_mult = I32F32::ONE;
        if perks.aim_tier >= 1 {
            damage_mult += I32F32::lit("0.40");
        }
        if perks.aim_tier >= 2 {
            damage_mult += I32F32::lit("0.50");
        }
        if perks.aim_tier >= 3 {
            damage_mult += I32F32::lit("0.70");
        }

        let mut hp_mult = I32F32::ONE;
        if perks.strength_tier >= 1 {
            hp_mult += I32F32::lit("0.40");
        }
        if perks.strength_tier >= 2 {
            hp_mult += I32F32::lit("0.40");
        }
        if perks.strength_tier >= 3 {
            hp_mult += I32F32::lit("0.50");
        }
        if perks.strength_tier >= 4 {
            hp_mult += I32F32::lit("0.60");
        }

        let mut as_mult = I32F32::ONE;
        if perks.dexterity_tier >= 1 {
            as_mult += I32F32::lit("0.20");
        }
        if perks.dexterity_tier >= 2 {
            as_mult += I32F32::lit("0.20");
        }
        if perks.dexterity_tier >= 3 {
            as_mult += I32F32::lit("0.20");
        }

        let mut ls_mult = I32F32::ONE;
        if perks.quick_reflexes_tier >= 1 {
            ls_mult += I32F32::lit("0.25");
        }
        if perks.quick_reflexes_tier >= 2 {
            ls_mult += I32F32::lit("0.25");
        }
        if perks.quick_reflexes_tier >= 3 {
            ls_mult += I32F32::lit("0.30");
        }

        let mut armor = CAELUS_BASE_ARMOR_REDUCTION;
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

        let mut range_bonus = I32F32::ZERO;
        if perks.improved_vision_i {
            range_bonus += I32F32::ONE;
        }
        if perks.improved_vision_iii {
            range_bonus += I32F32::ONE;
        }

        let mut radius_bonus = I32F32::ZERO;
        if perks.effect_radius_tier >= 1 {
            radius_bonus += I32F32::lit("0.2");
        }
        if perks.effect_radius_tier >= 2 {
            radius_bonus += I32F32::lit("0.2");
        }

        stats.move_speed = CAELUS_BASE_MS * move_mult;
        stats.attack_range = CAELUS_BASE_AR + range_bonus;
        stats.attack_speed = CAELUS_BASE_AS * as_mult;
        stats.attack_damage = CAELUS_BASE_AD * damage_mult;
        stats.watch_range = CAELUS_BASE_WR + range_bonus;

        combat_mods.armor_reduction = armor;
        combat_mods.effect_radius = CAELUS_BASE_EFFECT_RADIUS + radius_bonus;

        hp.max = CAELUS_BASE_HP * hp_mult;
        if hp.current > hp.max {
            hp.current = hp.max;
        }

        load_mult.0 = ls_mult;
    }
}

pub fn caelus_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &Health,
            &UnitStats,
            &CaelusCombatMods,
            &CaelusAttackCooldown,
            &CaelusPerkState,
            &CaelusLoadSpeedMultiplier,
        ),
        With<Caelus>,
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
        checksum.accumulate(combat_mods.effect_radius.to_bits() as u64);

        checksum.accumulate(cooldown.ticks_remaining.to_bits() as u64);
        checksum.accumulate(load_mult.0.to_bits() as u64);

        checksum.accumulate(perks.perk_points_available as u64);
        checksum.accumulate(perks.aim_tier as u64);
        checksum.accumulate(perks.strength_tier as u64);
        checksum.accumulate(perks.dexterity_tier as u64);
        checksum.accumulate(perks.quick_reflexes_tier as u64);
        checksum.accumulate(perks.protection_tier as u64);
        checksum.accumulate(u64::from(perks.improved_vision_i));
        checksum.accumulate(u64::from(perks.improved_vision_iii));
        checksum.accumulate(perks.speed_tier as u64);
        checksum.accumulate(perks.effect_radius_tier as u64);

        let mission_state_bits = match perks.mission_state {
            MissionState::InProgress => 0_u64,
            MissionState::Completed => 1_u64,
            MissionState::Lost => 2_u64,
            MissionState::Reloaded => 3_u64,
        };
        checksum.accumulate(mission_state_bits);
    }
}

pub struct CaelusPlugin;

impl Plugin for CaelusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CaelusDamageEvent>()
            .add_event::<CaelusPerkPointAwardEvent>()
            .add_event::<AllocateCaelusPerkEvent>()
            .add_event::<RefundCaelusPerksEvent>()
            .add_event::<UpdateCaelusMissionStateEvent>()
            .add_event::<ReissueCaelusAttackCommandEvent>()
            .add_systems(
                FixedUpdate,
                (
                    caelus_attack_tick_system,
                    caelus_reissue_attack_command_system,
                    caelus_receive_damage_system,
                    caelus_perk_point_award_system,
                    caelus_update_mission_state_system,
                    caelus_allocate_perk_system,
                    caelus_refund_perks_system,
                    caelus_apply_perks_system,
                    caelus_checksum_system,
                )
                    .chain(),
            );
    }
}