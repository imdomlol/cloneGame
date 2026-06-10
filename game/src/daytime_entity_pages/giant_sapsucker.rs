// Sources: vault/daytime_entity_pages/giant_sapsucker.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
    UnitStats,
};

pub const GIANT_SAPSUCKER_ID: &str = "giant_sapsucker";
pub const GIANT_SAPSUCKER_NAME: &str = "Giant Sapsucker";
pub const GIANT_SAPSUCKER_SCIENTIFIC_NAME: &str = "Sphyrapicus cursus";
pub const GIANT_SAPSUCKER_INTERNAL_NAME: &str = "Giant Kiwi";
pub const GIANT_SAPSUCKER_DWELLS: &str = "Outdoors";
pub const GIANT_SAPSUCKER_POWER_LEVEL: u8 = 4;
pub const GIANT_SAPSUCKER_MAX_SPAWNED: usize = 1;
pub const GIANT_SAPSUCKER_HP: I32F32 = I32F32::lit("28");
pub const GIANT_SAPSUCKER_PLAYER_ATTACK_DAMAGE: I32F32 = I32F32::lit("10");
pub const GIANT_SAPSUCKER_ENTITY_ATTACK_DAMAGE: I32F32 = I32F32::lit("4");
pub const GIANT_SAPSUCKER_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.5");
pub const GIANT_SAPSUCKER_SPAWN_DELAY_SECONDS: u16 = 15;
pub const GIANT_SAPSUCKER_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("1.6");
pub const GIANT_SAPSUCKER_DANGER_MIN_PERCENT: I32F32 = I32F32::lit("10");
pub const GIANT_SAPSUCKER_DANGER_MAX_PERCENT: I32F32 = I32F32::lit("80");
pub const GIANT_SAPSUCKER_LEAVE_MINUTE_OF_DAY: u16 = 1437;

pub struct GiantSapsuckerPlugin;

impl Plugin for GiantSapsuckerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnGiantSapsuckerEvent>()
            .add_event::<GiantSapsuckerEggStolenEvent>()
            .add_event::<GiantSapsuckerEggDroppedEvent>()
            .add_event::<GiantSapsuckerStunAppliedEvent>()
            .add_event::<GiantSapsuckerStunResolvedEvent>()
            .add_event::<GiantSapsuckerZapGunTargetedEvent>()
            .add_event::<GiantSapsuckerZapGunDifficultyEvent>()
            .add_event::<GiantSapsuckerShipDoorPriedEvent>()
            .add_event::<GiantSapsuckerEggRetrievedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_giant_sapsuckers,
                    giant_sapsucker_passive_roam_forage_rest,
                    giant_sapsucker_guard_nest_approach,
                    giant_sapsucker_aggravate_when_touched_or_too_close,
                    giant_sapsucker_pursue_egg_thief,
                    giant_sapsucker_enter_berserk_when_attacked,
                    giant_sapsucker_enrage_at_baboon_hawk,
                    giant_sapsucker_retrieve_dropped_egg,
                    giant_sapsucker_remember_seen_target,
                    giant_sapsucker_grow_attack_phase,
                    giant_sapsucker_attack_targets,
                    giant_sapsucker_return_after_attack_phase,
                    giant_sapsucker_apply_stun_multiplier,
                    giant_sapsucker_report_zap_gun_difficulty,
                    giant_sapsucker_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GiantSapsucker;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GiantSapsuckerNest;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GiantSapsuckerEgg;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GiantSapsuckerNestAnchor {
    pub position: SimPosition,
    pub guard_radius: I32F32,
    pub too_close_radius: I32F32,
    pub return_radius: I32F32,
    pub egg_retrieve_radius: I32F32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GiantSapsuckerAttackPhase {
    pub ticks_in_phase: u32,
    pub ticks_until_attack: u32,
    pub return_after_ticks: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GiantSapsuckerAttackGrowth {
    pub base_move_speed: I32F32,
    pub base_attack_speed: I32F32,
    pub move_speed_per_tick: I32F32,
    pub attack_speed_per_tick: I32F32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GiantSapsuckerMemory {
    pub has_target: bool,
    pub target_stable_id: u64,
    pub locked_location: SimPosition,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GiantSapsuckerTarget {
    pub stable_id: u64,
    pub kind: GiantSapsuckerTargetKind,
    pub is_touching_sapsucker: bool,
    pub is_holding_egg: bool,
    pub targets_eggs: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GiantSapsuckerTargetKind {
    #[default]
    Employee,
    Entity,
    BaboonHawk,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GiantSapsuckerState {
    #[default]
    Roaming,
    Foraging,
    Resting,
    Guard,
    Aggravated,
    ChasingEggThief,
    Berserk,
    RetrievingEgg,
    ReturningEggToNest,
    ReturningToNest,
}

#[derive(Bundle)]
pub struct GiantSapsuckerBundle {
    pub name: Name,
    pub sapsucker: GiantSapsucker,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub nest_anchor: GiantSapsuckerNestAnchor,
    pub state: GiantSapsuckerState,
    pub attack_phase: GiantSapsuckerAttackPhase,
    pub attack_growth: GiantSapsuckerAttackGrowth,
    pub memory: GiantSapsuckerMemory,
}

impl GiantSapsuckerBundle {
    pub fn new(event: SpawnGiantSapsuckerEvent) -> Self {
        Self {
            name: Name::new(GIANT_SAPSUCKER_NAME),
            sapsucker: GiantSapsucker,
            position: event.position,
            health: Health::full(GIANT_SAPSUCKER_HP),
            stats: UnitStats {
                move_speed: event.move_speed,
                attack_range: event.attack_range,
                attack_damage: GIANT_SAPSUCKER_PLAYER_ATTACK_DAMAGE,
                attack_speed: event.base_attack_speed,
                watch_range: event.watch_range,
            },
            nest_anchor: GiantSapsuckerNestAnchor {
                position: event.nest_position,
                guard_radius: event.guard_radius,
                too_close_radius: event.too_close_radius,
                return_radius: event.return_radius,
                egg_retrieve_radius: event.egg_retrieve_radius,
            },
            state: GiantSapsuckerState::Roaming,
            attack_phase: GiantSapsuckerAttackPhase {
                ticks_in_phase: 0,
                ticks_until_attack: 0,
                return_after_ticks: event.return_after_ticks,
            },
            attack_growth: GiantSapsuckerAttackGrowth {
                base_move_speed: event.move_speed,
                base_attack_speed: event.base_attack_speed,
                move_speed_per_tick: event.move_speed_growth_per_tick,
                attack_speed_per_tick: event.attack_speed_growth_per_tick,
            },
            memory: GiantSapsuckerMemory {
                has_target: false,
                target_stable_id: 0,
                locked_location: event.nest_position,
            },
        }
    }
}

#[derive(Bundle)]
pub struct GiantSapsuckerNestBundle {
    pub name: Name,
    pub nest: GiantSapsuckerNest,
    pub position: SimPosition,
}

impl GiantSapsuckerNestBundle {
    pub fn new(position: SimPosition) -> Self {
        Self {
            name: Name::new("Giant Sapsucker Nest"),
            nest: GiantSapsuckerNest,
            position,
        }
    }
}

#[derive(Bundle)]
pub struct GiantSapsuckerEggBundle {
    pub name: Name,
    pub egg: GiantSapsuckerEgg,
    pub position: SimPosition,
}

impl GiantSapsuckerEggBundle {
    pub fn new(position: SimPosition) -> Self {
        Self {
            name: Name::new("Giant Sapsucker Egg"),
            egg: GiantSapsuckerEgg,
            position,
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnGiantSapsuckerEvent {
    pub position: SimPosition,
    pub nest_position: SimPosition,
    pub guard_radius: I32F32,
    pub too_close_radius: I32F32,
    pub return_radius: I32F32,
    pub egg_retrieve_radius: I32F32,
    pub attack_range: I32F32,
    pub watch_range: I32F32,
    pub move_speed: I32F32,
    pub base_attack_speed: I32F32,
    pub move_speed_growth_per_tick: I32F32,
    pub attack_speed_growth_per_tick: I32F32,
    pub return_after_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerEggStolenEvent {
    pub thief: Entity,
    pub thief_stable_id: u64,
    pub thief_position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerEggDroppedEvent {
    pub egg: Entity,
    pub thief_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerStunAppliedEvent {
    pub sapsucker: Entity,
    pub raw_ticks: I32F32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerStunResolvedEvent {
    pub sapsucker: Entity,
    pub effective_ticks: I32F32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerZapGunTargetedEvent {
    pub sapsucker: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerZapGunDifficultyEvent {
    pub sapsucker: Entity,
    pub difficulty: I32F32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerShipDoorPriedEvent {
    pub sapsucker: Entity,
    pub target: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct GiantSapsuckerEggRetrievedEvent {
    pub sapsucker: Entity,
    pub egg: Entity,
}

fn spawn_giant_sapsuckers(
    mut commands: Commands,
    mut events: EventReader<SpawnGiantSapsuckerEvent>,
    sapsuckers: Query<(), With<GiantSapsucker>>,
) {
    let mut spawned_count = sapsuckers.iter().count();

    for event in events.read() {
        if spawned_count >= GIANT_SAPSUCKER_MAX_SPAWNED {
            break;
        }

        commands.spawn(GiantSapsuckerBundle::new(*event));
        spawned_count += 1;
    }
}

fn giant_sapsucker_passive_roam_forage_rest(
    tick: Res<SimTick>,
    mut sapsuckers: Query<&mut GiantSapsuckerState, With<GiantSapsucker>>,
) {
    for mut state in sapsuckers.iter_mut() {
        if !is_passive_state(*state) {
            continue;
        }

        if tick.0 % 12 == 0 {
            *state = GiantSapsuckerState::Resting;
        } else if tick.0 % 3 == 0 {
            *state = GiantSapsuckerState::Foraging;
        } else if *state != GiantSapsuckerState::Resting {
            *state = GiantSapsuckerState::Roaming;
        }
    }
}

fn giant_sapsucker_guard_nest_approach(
    mut sapsuckers: Query<(&mut GiantSapsuckerState, &GiantSapsuckerNestAnchor), With<GiantSapsucker>>,
    targets: Query<&SimPosition, With<GiantSapsuckerTarget>>,
) {
    for (mut state, nest_anchor) in sapsuckers.iter_mut() {
        if !is_passive_state(*state) {
            continue;
        }

        let guard_radius_squared = nest_anchor.guard_radius * nest_anchor.guard_radius;

        for target_position in targets.iter() {
            if distance_squared(*target_position, nest_anchor.position) <= guard_radius_squared {
                *state = GiantSapsuckerState::Guard;
                break;
            }
        }
    }
}

fn giant_sapsucker_aggravate_when_touched_or_too_close(
    mut sapsuckers: Query<
        (
            &mut GiantSapsuckerState,
            &GiantSapsuckerNestAnchor,
            &mut GiantSapsuckerMemory,
        ),
        With<GiantSapsucker>,
    >,
    targets: Query<(&SimPosition, &GiantSapsuckerTarget)>,
) {
    for (mut state, nest_anchor, mut memory) in sapsuckers.iter_mut() {
        if !is_passive_state(*state) && *state != GiantSapsuckerState::Guard {
            continue;
        }

        let too_close_radius_squared = nest_anchor.too_close_radius * nest_anchor.too_close_radius;
        let mut selected_target: Option<(u64, SimPosition)> = None;

        for (target_position, target) in targets.iter() {
            let too_close =
                distance_squared(*target_position, nest_anchor.position) <= too_close_radius_squared;

            if too_close || target.is_touching_sapsucker {
                selected_target = choose_lowest_stable_id(
                    selected_target,
                    target.stable_id,
                    *target_position,
                );
            }
        }

        if let Some((stable_id, position)) = selected_target {
            *state = GiantSapsuckerState::Aggravated;
            memory.has_target = true;
            memory.target_stable_id = stable_id;
            memory.locked_location = position;
        }
    }
}

fn giant_sapsucker_pursue_egg_thief(
    mut stolen_events: EventReader<GiantSapsuckerEggStolenEvent>,
    mut door_events: EventWriter<GiantSapsuckerShipDoorPriedEvent>,
    mut sapsuckers: Query<(Entity, &mut GiantSapsuckerState, &mut GiantSapsuckerMemory), With<GiantSapsucker>>,
) {
    for event in stolen_events.read() {
        for (sapsucker_entity, mut state, mut memory) in sapsuckers.iter_mut() {
            *state = GiantSapsuckerState::ChasingEggThief;
            memory.has_target = true;
            memory.target_stable_id = event.thief_stable_id;
            memory.locked_location = event.thief_position;

            door_events.send(GiantSapsuckerShipDoorPriedEvent {
                sapsucker: sapsucker_entity,
                target: event.thief,
            });
        }
    }
}

fn giant_sapsucker_enter_berserk_when_attacked(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut sapsuckers: Query<(Entity, &mut GiantSapsuckerState), With<GiantSapsucker>>,
) {
    for event in damage_events.read() {
        for (sapsucker_entity, mut state) in sapsuckers.iter_mut() {
            if event.target == sapsucker_entity {
                *state = GiantSapsuckerState::Berserk;
            }
        }
    }
}

fn giant_sapsucker_enrage_at_baboon_hawk(
    mut sapsuckers: Query<
        (&mut GiantSapsuckerState, &GiantSapsuckerNestAnchor, &mut GiantSapsuckerMemory),
        With<GiantSapsucker>,
    >,
    targets: Query<(&SimPosition, &GiantSapsuckerTarget)>,
) {
    for (mut state, nest_anchor, mut memory) in sapsuckers.iter_mut() {
        let guard_radius_squared = nest_anchor.guard_radius * nest_anchor.guard_radius;
        let mut selected_target: Option<(u64, SimPosition)> = None;

        for (target_position, target) in targets.iter() {
            if target.kind != GiantSapsuckerTargetKind::BaboonHawk || !target.targets_eggs {
                continue;
            }

            if distance_squared(*target_position, nest_anchor.position) <= guard_radius_squared {
                selected_target = choose_lowest_stable_id(
                    selected_target,
                    target.stable_id,
                    *target_position,
                );
            }
        }

        if let Some((stable_id, position)) = selected_target {
            *state = GiantSapsuckerState::Berserk;
            memory.has_target = true;
            memory.target_stable_id = stable_id;
            memory.locked_location = position;
        }
    }
}

fn giant_sapsucker_retrieve_dropped_egg(
    mut dropped_events: EventReader<GiantSapsuckerEggDroppedEvent>,
    mut retrieved_events: EventWriter<GiantSapsuckerEggRetrievedEvent>,
    mut sapsuckers: Query<
        (
            Entity,
            &SimPosition,
            &mut GiantSapsuckerState,
            &GiantSapsuckerNestAnchor,
            &mut GiantSapsuckerMemory,
        ),
        With<GiantSapsucker>,
    >,
) {
    for event in dropped_events.read() {
        for (sapsucker_entity, sapsucker_position, mut state, nest_anchor, mut memory) in
            sapsuckers.iter_mut()
        {
            if *state != GiantSapsuckerState::ChasingEggThief
                || memory.target_stable_id != event.thief_stable_id
            {
                continue;
            }

            memory.locked_location = event.position;
            *state = GiantSapsuckerState::RetrievingEgg;

            let retrieve_radius_squared =
                nest_anchor.egg_retrieve_radius * nest_anchor.egg_retrieve_radius;

            if distance_squared(*sapsucker_position, event.position) <= retrieve_radius_squared {
                *state = GiantSapsuckerState::ReturningEggToNest;
                retrieved_events.send(GiantSapsuckerEggRetrievedEvent {
                    sapsucker: sapsucker_entity,
                    egg: event.egg,
                });
            }
        }
    }
}

fn giant_sapsucker_remember_seen_target(
    mut sapsuckers: Query<
        (
            &mut GiantSapsuckerState,
            &SimPosition,
            &UnitStats,
            &GiantSapsuckerMemory,
        ),
        With<GiantSapsucker>,
    >,
    targets: Query<(&SimPosition, &GiantSapsuckerTarget)>,
) {
    for (mut state, sapsucker_position, stats, memory) in sapsuckers.iter_mut() {
        if !memory.has_target || is_attack_phase(*state) {
            continue;
        }

        let watch_range_squared = stats.watch_range * stats.watch_range;

        for (target_position, target) in targets.iter() {
            if target.stable_id != memory.target_stable_id {
                continue;
            }

            if distance_squared(*sapsucker_position, *target_position) <= watch_range_squared {
                *state = GiantSapsuckerState::Aggravated;
                break;
            }
        }
    }
}

fn giant_sapsucker_grow_attack_phase(
    mut sapsuckers: Query<
        (
            &GiantSapsuckerState,
            &mut GiantSapsuckerAttackPhase,
            &GiantSapsuckerAttackGrowth,
            &mut UnitStats,
        ),
        With<GiantSapsucker>,
    >,
) {
    for (state, mut attack_phase, attack_growth, mut stats) in sapsuckers.iter_mut() {
        if is_attack_phase(*state) {
            attack_phase.ticks_in_phase = attack_phase.ticks_in_phase.saturating_add(1);
            let ticks = I32F32::from_num(attack_phase.ticks_in_phase);
            stats.move_speed = attack_growth.base_move_speed + attack_growth.move_speed_per_tick * ticks;
            stats.attack_speed =
                attack_growth.base_attack_speed + attack_growth.attack_speed_per_tick * ticks;
        } else {
            attack_phase.ticks_in_phase = 0;
            stats.move_speed = attack_growth.base_move_speed;
            stats.attack_speed = attack_growth.base_attack_speed;
        }
    }
}

fn giant_sapsucker_attack_targets(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    sim_hz: Res<SimHz>,
    mut sapsuckers: Query<
        (
            Entity,
            &SimPosition,
            &UnitStats,
            &GiantSapsuckerState,
            &GiantSapsuckerMemory,
            &mut GiantSapsuckerAttackPhase,
        ),
        With<GiantSapsucker>,
    >,
    targets: Query<(Entity, &SimPosition, &GiantSapsuckerTarget)>,
) {
    for (sapsucker_entity, sapsucker_position, stats, state, memory, mut attack_phase) in
        sapsuckers.iter_mut()
    {
        if !is_attack_phase(*state) {
            continue;
        }

        if attack_phase.ticks_until_attack > 0 {
            attack_phase.ticks_until_attack -= 1;
            continue;
        }

        let attack_range_squared = stats.attack_range * stats.attack_range;
        let mut selected: Option<(Entity, u64, GiantSapsuckerTargetKind)> = None;

        for (target_entity, target_position, target) in targets.iter() {
            if *state == GiantSapsuckerState::ChasingEggThief
                && (!memory.has_target || target.stable_id != memory.target_stable_id)
            {
                continue;
            }

            if distance_squared(*sapsucker_position, *target_position) > attack_range_squared {
                continue;
            }

            selected = match selected {
                Some((_, selected_stable_id, _)) if selected_stable_id <= target.stable_id => selected,
                _ => Some((target_entity, target.stable_id, target.kind)),
            };
        }

        if let Some((target_entity, _, target_kind)) = selected {
            let raw_amount = match target_kind {
                GiantSapsuckerTargetKind::Employee => GIANT_SAPSUCKER_PLAYER_ATTACK_DAMAGE,
                GiantSapsuckerTargetKind::Entity | GiantSapsuckerTargetKind::BaboonHawk => {
                    GIANT_SAPSUCKER_ENTITY_ATTACK_DAMAGE
                }
            };

            damage_events.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount,
                damage_type: DamageType::Standard,
                source: sapsucker_entity,
            });
            attack_phase.ticks_until_attack = attack_period_ticks(sim_hz.0, stats.attack_speed);
        }
    }
}

fn giant_sapsucker_return_after_attack_phase(
    mut sapsuckers: Query<
        (
            &mut GiantSapsuckerState,
            &SimPosition,
            &GiantSapsuckerNestAnchor,
            &mut GiantSapsuckerAttackPhase,
        ),
        With<GiantSapsucker>,
    >,
) {
    for (mut state, position, nest_anchor, mut attack_phase) in sapsuckers.iter_mut() {
        if is_attack_phase(*state) && attack_phase.ticks_in_phase >= attack_phase.return_after_ticks {
            *state = GiantSapsuckerState::ReturningToNest;
            attack_phase.ticks_in_phase = 0;
        }

        if *state == GiantSapsuckerState::ReturningEggToNest
            || *state == GiantSapsuckerState::ReturningToNest
        {
            let return_radius_squared = nest_anchor.return_radius * nest_anchor.return_radius;

            if distance_squared(*position, nest_anchor.position) <= return_radius_squared {
                *state = GiantSapsuckerState::Roaming;
                attack_phase.ticks_in_phase = 0;
            }
        }
    }
}

fn giant_sapsucker_apply_stun_multiplier(
    mut stun_events: EventReader<GiantSapsuckerStunAppliedEvent>,
    mut resolved_events: EventWriter<GiantSapsuckerStunResolvedEvent>,
    mut sapsuckers: Query<(Entity, &mut GiantSapsuckerAttackPhase), With<GiantSapsucker>>,
) {
    for event in stun_events.read() {
        for (sapsucker_entity, mut attack_phase) in sapsuckers.iter_mut() {
            if event.sapsucker != sapsucker_entity {
                continue;
            }

            attack_phase.ticks_until_attack = 0;
            resolved_events.send(GiantSapsuckerStunResolvedEvent {
                sapsucker: sapsucker_entity,
                effective_ticks: event.raw_ticks * GIANT_SAPSUCKER_STUN_MULTIPLIER,
            });
        }
    }
}

fn giant_sapsucker_report_zap_gun_difficulty(
    mut targeted_events: EventReader<GiantSapsuckerZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<GiantSapsuckerZapGunDifficultyEvent>,
    sapsuckers: Query<(), With<GiantSapsucker>>,
) {
    for event in targeted_events.read() {
        if sapsuckers.get(event.sapsucker).is_ok() {
            difficulty_events.send(GiantSapsuckerZapGunDifficultyEvent {
                sapsucker: event.sapsucker,
                difficulty: GIANT_SAPSUCKER_ZAP_GUN_DIFFICULTY,
            });
        }
    }
}

fn giant_sapsucker_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sapsuckers: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &GiantSapsuckerNestAnchor,
            &GiantSapsuckerState,
            &GiantSapsuckerAttackPhase,
            &GiantSapsuckerAttackGrowth,
            &GiantSapsuckerMemory,
        ),
        With<GiantSapsucker>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(GIANT_SAPSUCKER_POWER_LEVEL as u64);
    checksum.accumulate(GIANT_SAPSUCKER_MAX_SPAWNED as u64);
    checksum.accumulate(GIANT_SAPSUCKER_HP.to_bits() as u64);
    checksum.accumulate(GIANT_SAPSUCKER_PLAYER_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(GIANT_SAPSUCKER_ENTITY_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(GIANT_SAPSUCKER_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(GIANT_SAPSUCKER_SPAWN_DELAY_SECONDS as u64);
    checksum.accumulate(GIANT_SAPSUCKER_ZAP_GUN_DIFFICULTY.to_bits() as u64);
    checksum.accumulate(GIANT_SAPSUCKER_DANGER_MIN_PERCENT.to_bits() as u64);
    checksum.accumulate(GIANT_SAPSUCKER_DANGER_MAX_PERCENT.to_bits() as u64);
    checksum.accumulate(GIANT_SAPSUCKER_LEAVE_MINUTE_OF_DAY as u64);

    for (
        position,
        health,
        stats,
        nest_anchor,
        state,
        attack_phase,
        attack_growth,
        memory,
    ) in sapsuckers.iter()
    {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(nest_anchor.position.x.to_bits() as u64);
        checksum.accumulate(nest_anchor.position.y.to_bits() as u64);
        checksum.accumulate(nest_anchor.guard_radius.to_bits() as u64);
        checksum.accumulate(nest_anchor.too_close_radius.to_bits() as u64);
        checksum.accumulate(nest_anchor.return_radius.to_bits() as u64);
        checksum.accumulate(nest_anchor.egg_retrieve_radius.to_bits() as u64);
        checksum.accumulate(*state as u64);
        checksum.accumulate(attack_phase.ticks_in_phase as u64);
        checksum.accumulate(attack_phase.ticks_until_attack as u64);
        checksum.accumulate(attack_phase.return_after_ticks as u64);
        checksum.accumulate(attack_growth.base_move_speed.to_bits() as u64);
        checksum.accumulate(attack_growth.base_attack_speed.to_bits() as u64);
        checksum.accumulate(attack_growth.move_speed_per_tick.to_bits() as u64);
        checksum.accumulate(attack_growth.attack_speed_per_tick.to_bits() as u64);
        checksum.accumulate(memory.has_target as u64);
        checksum.accumulate(memory.target_stable_id);
        checksum.accumulate(memory.locked_location.x.to_bits() as u64);
        checksum.accumulate(memory.locked_location.y.to_bits() as u64);
    }
}

fn choose_lowest_stable_id(
    current: Option<(u64, SimPosition)>,
    stable_id: u64,
    position: SimPosition,
) -> Option<(u64, SimPosition)> {
    match current {
        Some((current_id, current_position)) if current_id <= stable_id => {
            Some((current_id, current_position))
        }
        _ => Some((stable_id, position)),
    }
}

fn is_passive_state(state: GiantSapsuckerState) -> bool {
    state == GiantSapsuckerState::Roaming
        || state == GiantSapsuckerState::Foraging
        || state == GiantSapsuckerState::Resting
}

fn is_attack_phase(state: GiantSapsuckerState) -> bool {
    state == GiantSapsuckerState::Aggravated
        || state == GiantSapsuckerState::ChasingEggThief
        || state == GiantSapsuckerState::Berserk
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn attack_period_ticks(sim_hz: I32F32, attack_speed: I32F32) -> u32 {
    if attack_speed <= I32F32::lit("0") {
        return 1;
    }

    let period = sim_hz / attack_speed;
    let ticks = period.ceil().to_num::<u32>();

    if ticks == 0 {
        1
    } else {
        ticks
    }
}