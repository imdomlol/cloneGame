// Sources: vault/indoor_entity_pages/thumper.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};

pub const THUMPER_ID: &str = "thumper";
pub const THUMPER_NAME: &str = "Thumper";
pub const THUMPER_TYPE: &str = "indoor_entity_pages";
pub const THUMPER_SUBTYPE: &str = "creature";
pub const THUMPER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Thumper";
pub const THUMPER_SOURCE_REVISION: u32 = 21485;
pub const THUMPER_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const THUMPER_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const THUMPER_DWELLS: &str = "Inside";
pub const THUMPER_DANGER: &str = "90%";
pub const THUMPER_SCIENTIFIC_NAME: &str = "Pistris-saevus";
pub const THUMPER_HP: I32F32 = I32F32::lit("4");
pub const THUMPER_POWER_LEVEL: I32F32 = I32F32::lit("3");
pub const THUMPER_MAX_SPAWNED: usize = 4;
pub const THUMPER_ATTACK_DAMAGE: I32F32 = I32F32::lit("40");
pub const THUMPER_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.4");
pub const THUMPER_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.5");
pub const THUMPER_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.3");
pub const THUMPER_INTERNAL_NAME: &str = "Crawler";
pub const THUMPER_PIP_SIZE: &str = "Medium";

pub const THUMPER_BASE_MOVE_SPEED: I32F32 = I32F32::lit("3");
pub const THUMPER_MAX_CHASE_SPEED: I32F32 = I32F32::lit("9");
pub const THUMPER_ACCELERATION_PER_TICK: I32F32 = I32F32::lit("0.25");
pub const THUMPER_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const THUMPER_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("1");
pub const THUMPER_WATCH_RANGE: I32F32 = I32F32::lit("30");
pub const THUMPER_WEAPON_HIT_DAMAGE: I32F32 = I32F32::lit("1");

pub const THUMPER_DEPENDS_ON: [&str; 0] = [];
pub const THUMPER_FRONTMATTER_BEHAVIOR: [&str; 1] = ["Aggressive"];

pub const THUMPER_BEHAVIORAL_MECHANICS: [ThumperBehaviorRule; 7] = [
    ThumperBehaviorRule {
        condition: "a target remains in line of sight",
        outcome: "the Thumper accelerates toward that target",
    },
    ThumperBehaviorRule {
        condition: "the Thumper loses line of sight",
        outcome: "it investigates the last seen position",
    },
    ThumperBehaviorRule {
        condition: "the Thumper reaches a target",
        outcome: "it attacks with 40 damage per hit",
    },
    ThumperBehaviorRule {
        condition: "turning around corners",
        outcome: "its speed and pathing become less reliable",
    },
    ThumperBehaviorRule {
        condition: "the Thumper is stunned",
        outcome: "its current speed resets and its momentum is interrupted",
    },
    ThumperBehaviorRule {
        condition: "a target stands above railing height",
        outcome: "the Thumper cannot attack from below",
    },
    ThumperBehaviorRule {
        condition: "the Thumper takes 4 successful hits",
        outcome: "it is defeated",
    },
];

pub struct ThumperPlugin;

impl Plugin for ThumperPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnThumperEvent>()
            .add_event::<ThumperStateChangedEvent>()
            .add_event::<ThumperAttackEvent>()
            .add_event::<ThumperDoorAttemptEvent>()
            .add_event::<ThumperDoorAttemptResolvedEvent>()
            .add_event::<ThumperStunAppliedEvent>()
            .add_event::<ThumperStunAdjustedEvent>()
            .add_event::<ThumperZapGunTargetedEvent>()
            .add_event::<ThumperZapGunDifficultyEvent>()
            .add_event::<ThumperCornerTurnEvent>()
            .add_event::<ThumperDamageTakenEvent>()
            .add_event::<ThumperDefeatedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_thumper,
                    thumper_acquire_or_update_visible_target,
                    thumper_investigate_last_seen_position,
                    thumper_corner_turn_unreliable_pathing,
                    thumper_attack_reachable_targets,
                    thumper_apply_stun_multiplier,
                    thumper_report_zap_gun_difficulty,
                    thumper_door_attempt_speed,
                    thumper_take_weapon_hits,
                    thumper_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Thumper;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThumperTargetSensor {
    pub stable_id: u64,
    pub in_line_of_sight: bool,
    pub reachable: bool,
    pub above_railing_height: bool,
    pub around_corner: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperTargetMemory {
    pub has_target: bool,
    pub target_stable_id: u64,
    pub last_seen_position: SimPosition,
}

impl Default for ThumperTargetMemory {
    fn default() -> Self {
        Self {
            has_target: false,
            target_stable_id: 0,
            last_seen_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperMomentum {
    pub current_speed: I32F32,
    pub interrupted: bool,
    pub unreliable_pathing_ticks: u32,
}

impl Default for ThumperMomentum {
    fn default() -> Self {
        Self {
            current_speed: THUMPER_BASE_MOVE_SPEED,
            interrupted: false,
            unreliable_pathing_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperAttackTiming {
    pub cooldown_ticks: u32,
    pub timer_ticks: u32,
}

impl Default for ThumperAttackTiming {
    fn default() -> Self {
        Self {
            cooldown_ticks: 20,
            timer_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThumperState {
    #[default]
    Searching,
    Chasing,
    Investigating,
    Stunned,
    Defeated,
}

#[derive(Bundle)]
pub struct ThumperBundle {
    pub name: Name,
    pub thumper: Thumper,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: ThumperState,
    pub memory: ThumperTargetMemory,
    pub momentum: ThumperMomentum,
    pub attack_timing: ThumperAttackTiming,
}

impl ThumperBundle {
    pub fn new(event: SpawnThumperEvent) -> Self {
        Self {
            name: Name::new(THUMPER_NAME),
            thumper: Thumper,
            position: event.position,
            health: Health::full(THUMPER_HP),
            stats: UnitStats {
                move_speed: THUMPER_BASE_MOVE_SPEED,
                attack_range: THUMPER_ATTACK_RANGE,
                attack_damage: THUMPER_ATTACK_DAMAGE,
                attack_speed: THUMPER_ATTACK_SPEED_SECONDS,
                watch_range: THUMPER_WATCH_RANGE,
            },
            state: ThumperState::Searching,
            memory: ThumperTargetMemory {
                has_target: false,
                target_stable_id: 0,
                last_seen_position: event.position,
            },
            momentum: ThumperMomentum::default(),
            attack_timing: ThumperAttackTiming::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnThumperEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperStateChangedEvent {
    pub thumper: Entity,
    pub from: ThumperState,
    pub to: ThumperState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperAttackEvent {
    pub thumper: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperDoorAttemptEvent {
    pub thumper: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperDoorAttemptResolvedEvent {
    pub thumper: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperStunAppliedEvent {
    pub thumper: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperStunAdjustedEvent {
    pub thumper: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperZapGunTargetedEvent {
    pub thumper: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperZapGunDifficultyEvent {
    pub thumper: Entity,
    pub difficulty_modifier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperCornerTurnEvent {
    pub thumper: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperDamageTakenEvent {
    pub thumper: Entity,
    pub source: Entity,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumperDefeatedEvent {
    pub thumper: Entity,
    pub source: Entity,
}

fn spawn_thumper(
    mut commands: Commands,
    mut events: EventReader<SpawnThumperEvent>,
    thumpers: Query<(), With<Thumper>>,
) {
    let mut spawned_count = thumpers.iter().count();

    for event in events.read() {
        if spawned_count >= THUMPER_MAX_SPAWNED {
            break;
        }

        commands.spawn(ThumperBundle::new(*event));
        spawned_count += 1;
    }
}

fn thumper_acquire_or_update_visible_target(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ThumperStateChangedEvent>,
    mut thumpers: Query<
        (
            Entity,
            &mut SimPosition,
            &mut ThumperState,
            &mut ThumperTargetMemory,
            &mut ThumperMomentum,
            &mut UnitStats,
        ),
        With<Thumper>,
    >,
    targets: Query<(&SimPosition, &ThumperTargetSensor), Without<Thumper>>,
) {
    let Some((target_position, target_sensor)) = visible_target(&targets) else {
        return;
    };

    for (thumper_entity, mut position, mut state, mut memory, mut momentum, mut stats) in
        thumpers.iter_mut()
    {
        if *state == ThumperState::Defeated || *state == ThumperState::Stunned {
            continue;
        }

        memory.has_target = true;
        memory.target_stable_id = target_sensor.stable_id;
        memory.last_seen_position = target_position;
        momentum.current_speed = fixed_min(
            momentum.current_speed + THUMPER_ACCELERATION_PER_TICK,
            THUMPER_MAX_CHASE_SPEED,
        );
        momentum.interrupted = false;
        stats.move_speed = momentum.current_speed;

        move_axis_toward(&mut position, target_position, momentum.current_speed / sim_hz.0);
        set_thumper_state(
            thumper_entity,
            &mut state,
            ThumperState::Chasing,
            &mut state_events,
        );
    }
}

fn thumper_investigate_last_seen_position(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ThumperStateChangedEvent>,
    mut thumpers: Query<
        (
            Entity,
            &mut SimPosition,
            &mut ThumperState,
            &ThumperTargetMemory,
            &mut ThumperMomentum,
            &mut UnitStats,
        ),
        With<Thumper>,
    >,
    targets: Query<&ThumperTargetSensor>,
) {
    for (thumper_entity, mut position, mut state, memory, mut momentum, mut stats) in
        thumpers.iter_mut()
    {
        if *state == ThumperState::Defeated || *state == ThumperState::Stunned {
            continue;
        }

        if !memory.has_target {
            continue;
        }

        let target_visible = targets
            .iter()
            .any(|target| target.stable_id == memory.target_stable_id && target.in_line_of_sight);

        if target_visible {
            continue;
        }

        momentum.current_speed = THUMPER_BASE_MOVE_SPEED;
        momentum.interrupted = false;
        stats.move_speed = momentum.current_speed;
        move_axis_toward(
            &mut position,
            memory.last_seen_position,
            momentum.current_speed / sim_hz.0,
        );

        set_thumper_state(
            thumper_entity,
            &mut state,
            ThumperState::Investigating,
            &mut state_events,
        );
    }
}

fn thumper_corner_turn_unreliable_pathing(
    mut events: EventWriter<ThumperCornerTurnEvent>,
    mut thumpers: Query<(Entity, &ThumperTargetMemory, &mut ThumperMomentum), With<Thumper>>,
    targets: Query<&ThumperTargetSensor>,
) {
    for (thumper_entity, memory, mut momentum) in thumpers.iter_mut() {
        if !memory.has_target {
            continue;
        }

        let turning_corner = targets
            .iter()
            .any(|target| target.stable_id == memory.target_stable_id && target.around_corner);

        if !turning_corner {
            if momentum.unreliable_pathing_ticks > 0 {
                momentum.unreliable_pathing_ticks -= 1;
            }
            continue;
        }

        momentum.current_speed = fixed_max(THUMPER_BASE_MOVE_SPEED, momentum.current_speed / I32F32::lit("2"));
        momentum.unreliable_pathing_ticks = 1;
        events.send(ThumperCornerTurnEvent {
            thumper: thumper_entity,
            target_stable_id: memory.target_stable_id,
        });
    }
}

fn thumper_attack_reachable_targets(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut attack_events: EventWriter<ThumperAttackEvent>,
    mut thumpers: Query<
        (
            Entity,
            &ThumperState,
            &ThumperTargetMemory,
            &UnitStats,
            &mut ThumperAttackTiming,
        ),
        With<Thumper>,
    >,
    targets: Query<(Entity, &ThumperTargetSensor)>,
) {
    for (thumper_entity, state, memory, stats, mut timing) in thumpers.iter_mut() {
        if *state == ThumperState::Defeated || *state == ThumperState::Stunned {
            continue;
        }

        if timing.cooldown_ticks == 0 {
            timing.cooldown_ticks = fixed_seconds_to_ticks(stats.attack_speed, sim_hz.0);
        }

        if timing.timer_ticks > 0 {
            timing.timer_ticks -= 1;
            continue;
        }

        if !memory.has_target {
            continue;
        }

        let Some((target_entity, target_sensor)) = reachable_target(memory.target_stable_id, &targets) else {
            continue;
        };

        if target_sensor.above_railing_height {
            continue;
        }

        attack_events.send(ThumperAttackEvent {
            thumper: thumper_entity,
            target: target_entity,
            target_stable_id: target_sensor.stable_id,
            damage: THUMPER_ATTACK_DAMAGE,
        });
        damage_events.send(IncomingDamageEvent {
            target: target_entity,
            raw_amount: THUMPER_ATTACK_DAMAGE,
            damage_type: DamageType::Standard,
            source: thumper_entity,
        });
        timing.timer_ticks = timing.cooldown_ticks;
    }
}

fn thumper_apply_stun_multiplier(
    mut events: EventReader<ThumperStunAppliedEvent>,
    mut adjusted_events: EventWriter<ThumperStunAdjustedEvent>,
    mut state_events: EventWriter<ThumperStateChangedEvent>,
    mut thumpers: Query<(Entity, &mut ThumperState, &mut ThumperMomentum, &mut UnitStats), With<Thumper>>,
) {
    for event in events.read() {
        let Ok((thumper_entity, mut state, mut momentum, mut stats)) = thumpers.get_mut(event.thumper) else {
            continue;
        };

        momentum.current_speed = I32F32::lit("0");
        momentum.interrupted = true;
        stats.move_speed = I32F32::lit("0");

        set_thumper_state(
            thumper_entity,
            &mut state,
            ThumperState::Stunned,
            &mut state_events,
        );

        adjusted_events.send(ThumperStunAdjustedEvent {
            thumper: event.thumper,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, THUMPER_STUN_MULTIPLIER),
        });
    }
}

fn thumper_report_zap_gun_difficulty(
    mut events: EventReader<ThumperZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<ThumperZapGunDifficultyEvent>,
    thumpers: Query<(), With<Thumper>>,
) {
    for event in events.read() {
        if thumpers.get(event.thumper).is_err() {
            continue;
        }

        difficulty_events.send(ThumperZapGunDifficultyEvent {
            thumper: event.thumper,
            difficulty_modifier: THUMPER_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn thumper_door_attempt_speed(
    mut events: EventReader<ThumperDoorAttemptEvent>,
    mut resolved_events: EventWriter<ThumperDoorAttemptResolvedEvent>,
    thumpers: Query<(), With<Thumper>>,
) {
    for event in events.read() {
        if thumpers.get(event.thumper).is_err() {
            continue;
        }

        resolved_events.send(ThumperDoorAttemptResolvedEvent {
            thumper: event.thumper,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(
                event.base_open_ticks,
                THUMPER_DOOR_SPEED_MULTIPLIER,
            ),
        });
    }
}

fn thumper_take_weapon_hits(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut taken_events: EventWriter<ThumperDamageTakenEvent>,
    mut defeated_events: EventWriter<ThumperDefeatedEvent>,
    mut state_events: EventWriter<ThumperStateChangedEvent>,
    mut thumpers: Query<(Entity, &mut Health, &mut ThumperState), With<Thumper>>,
) {
    for event in damage_events.read() {
        let Ok((thumper_entity, mut health, mut state)) = thumpers.get_mut(event.target) else {
            continue;
        };

        if *state == ThumperState::Defeated {
            continue;
        }

        health.current = fixed_max(I32F32::lit("0"), health.current - THUMPER_WEAPON_HIT_DAMAGE);

        taken_events.send(ThumperDamageTakenEvent {
            thumper: thumper_entity,
            source: event.source,
            remaining_health: health.current,
        });

        if health.current > I32F32::lit("0") {
            continue;
        }

        set_thumper_state(
            thumper_entity,
            &mut state,
            ThumperState::Defeated,
            &mut state_events,
        );
        defeated_events.send(ThumperDefeatedEvent {
            thumper: thumper_entity,
            source: event.source,
        });
    }
}

fn thumper_checksum(
    mut checksum: ResMut<SimChecksumState>,
    thumpers: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &ThumperState,
            &ThumperTargetMemory,
            &ThumperMomentum,
            &ThumperAttackTiming,
        ),
        With<Thumper>,
    >,
) {
    for (position, health, stats, state, memory, momentum, timing) in thumpers.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(thumper_state_bits(*state));
        checksum.accumulate(memory.has_target as u64);
        checksum.accumulate(memory.target_stable_id);
        checksum.accumulate(memory.last_seen_position.x.to_bits() as u64);
        checksum.accumulate(memory.last_seen_position.y.to_bits() as u64);
        checksum.accumulate(momentum.current_speed.to_bits() as u64);
        checksum.accumulate(momentum.interrupted as u64);
        checksum.accumulate(momentum.unreliable_pathing_ticks as u64);
        checksum.accumulate(timing.cooldown_ticks as u64);
        checksum.accumulate(timing.timer_ticks as u64);
    }
}

fn visible_target(
    targets: &Query<(&SimPosition, &ThumperTargetSensor), Without<Thumper>>,
) -> Option<(SimPosition, ThumperTargetSensor)> {
    let mut best: Option<(SimPosition, ThumperTargetSensor)> = None;

    for (position, sensor) in targets.iter() {
        if !sensor.in_line_of_sight {
            continue;
        }

        if let Some((_best_position, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((*position, *sensor));
    }

    best
}

fn reachable_target(
    stable_id: u64,
    targets: &Query<(Entity, &ThumperTargetSensor)>,
) -> Option<(Entity, ThumperTargetSensor)> {
    for (entity, sensor) in targets.iter() {
        if sensor.stable_id == stable_id && sensor.reachable {
            return Some((entity, *sensor));
        }
    }

    None
}

fn set_thumper_state(
    thumper: Entity,
    state: &mut ThumperState,
    next: ThumperState,
    events: &mut EventWriter<ThumperStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(ThumperStateChangedEvent {
        thumper,
        from: previous,
        to: next,
    });
}

fn move_axis_toward(position: &mut SimPosition, destination: SimPosition, max_step: I32F32) {
    let dx = destination.x - position.x;
    let dy = destination.y - position.y;

    if fixed_abs(dx) >= fixed_abs(dy) {
        position.x += fixed_clamp(dx, -max_step, max_step);
    } else {
        position.y += fixed_clamp(dy, -max_step, max_step);
    }
}

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    let whole_ticks: u32 = ticks.to_num();

    if whole_ticks == 0 {
        1
    } else {
        whole_ticks
    }
}

fn fixed_ticks_scaled(ticks: u32, scale: I32F32) -> u32 {
    let scaled = I32F32::from_num(ticks) * scale;
    let whole_ticks: u32 = scaled.to_num();

    if whole_ticks == 0 {
        1
    } else {
        whole_ticks
    }
}

fn fixed_abs(value: I32F32) -> I32F32 {
    if value < I32F32::lit("0") {
        -value
    } else {
        value
    }
}

fn fixed_clamp(value: I32F32, min: I32F32, max: I32F32) -> I32F32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn fixed_min(left: I32F32, right: I32F32) -> I32F32 {
    if left < right {
        left
    } else {
        right
    }
}

fn fixed_max(left: I32F32, right: I32F32) -> I32F32 {
    if left > right {
        left
    } else {
        right
    }
}

fn thumper_state_bits(state: ThumperState) -> u64 {
    match state {
        ThumperState::Searching => 0,
        ThumperState::Chasing => 1,
        ThumperState::Investigating => 2,
        ThumperState::Stunned => 3,
        ThumperState::Defeated => 4,
    }
}