// Sources: vault/indoor_entity_pages/barber.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
    UnitStats,
};

pub const BARBER_ID: &str = "barber";
pub const BARBER_NAME: &str = "Barber";
pub const BARBER_TYPE: &str = "indoor_entity_pages";
pub const BARBER_SUBTYPE: &str = "entity";
pub const BARBER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Barber";
pub const BARBER_SOURCE_REVISION: u32 = 21383;
pub const BARBER_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const BARBER_CONFIDENCE_BASIS_POINTS: u16 = 86;

pub const BARBER_DWELLS: &str = "Indoors";
pub const BARBER_DANGER: &str = "Not mentioned by Sigurd";
pub const BARBER_HP: &str = "Immune";
pub const BARBER_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const BARBER_MAX_SPAWNED: usize = 1;
pub const BARBER_ATTACK_SPEED: &str = "2s - 0.2s";
pub const BARBER_STUN_MULTIPLIER: I32F32 = I32F32::lit("1.35");
pub const BARBER_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.7");
pub const BARBER_INTERNAL_NAME: &str = "ClaySurgeon";
pub const BARBER_PIP_SIZE: &str = "Medium";
pub const BARBER_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.3");
pub const BARBER_CONTACT_DAMAGE: &str = "Instant Kill";

pub const BARBER_DAY_START_MINUTE: u16 = 480;
pub const BARBER_DAY_END_MINUTE: u16 = 1380;
pub const BARBER_START_JUMP_INTERVAL_SECONDS: I32F32 = I32F32::lit("2.75");
pub const BARBER_END_JUMP_INTERVAL_SECONDS: I32F32 = I32F32::lit("1.25");
pub const BARBER_UNSEEN_LUNGE_COUNT: u8 = 6;
pub const BARBER_LUNGE_DISTANCE: I32F32 = I32F32::lit("4");
pub const BARBER_ROAM_SPEED: I32F32 = I32F32::lit("1");
pub const BARBER_FOLLOW_SPEED: I32F32 = I32F32::lit("2");
pub const BARBER_WATCH_RANGE: I32F32 = I32F32::lit("24");
pub const BARBER_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");
pub const BARBER_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("1000000");

pub const BARBER_DEPENDS_ON: [&str; 0] = [];
pub const BARBER_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Roaming", "Following"];

pub const BARBER_BEHAVIORAL_MECHANICS: [BarberBehaviorRule; 12] = [
    BarberBehaviorRule {
        condition: "the day begins at 8:00 a.m.",
        outcome: "the barber's jump interval starts at 2.75 s",
    },
    BarberBehaviorRule {
        condition: "time advances through the day",
        outcome: "the barber's jump interval decreases until it reaches 1.25 s at 11:00 p.m.",
    },
    BarberBehaviorRule {
        condition: "the barber is in Roaming state",
        outcome: "it wanders back and forth between 2 areas",
    },
    BarberBehaviorRule {
        condition: "an employee can fully see the barber and is in front of it",
        outcome: "it switches to Following state",
    },
    BarberBehaviorRule {
        condition: "the barber is in Following state",
        outcome: "it keeps jumping toward the target employee at the current interval",
    },
    BarberBehaviorRule {
        condition: "a snare drum roll occurs",
        outcome: "the barber locks its track to the target player's location at that moment",
    },
    BarberBehaviorRule {
        condition: "a snare hit occurs",
        outcome: "the barber lunges forward toward the tracked employee",
    },
    BarberBehaviorRule {
        condition: "the target employee is out of range to see the pursuing barber",
        outcome: "it continues about 6 lunges before returning to Roaming",
    },
    BarberBehaviorRule {
        condition: "the barber attempts a door",
        outcome: "its door speed is multiplied by 0.3 and it typically fails because it jumps before opening it",
    },
    BarberBehaviorRule {
        condition: "the barber is stunned",
        outcome: "the stun effect is multiplied by 1.35",
    },
    BarberBehaviorRule {
        condition: "a zap gun targets the barber",
        outcome: "the zap gun difficulty modifier is 0.7",
    },
    BarberBehaviorRule {
        condition: "a player touches the barber",
        outcome: "the contact damage is Instant Kill",
    },
];

pub struct BarberPlugin;

impl Plugin for BarberPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBarberEvent>()
            .add_event::<BarberStateChangedEvent>()
            .add_event::<BarberSnareDrumRollEvent>()
            .add_event::<BarberSnareHitEvent>()
            .add_event::<BarberDoorAttemptEvent>()
            .add_event::<BarberDoorAttemptResolvedEvent>()
            .add_event::<BarberStunAppliedEvent>()
            .add_event::<BarberStunAdjustedEvent>()
            .add_event::<BarberZapGunTargetedEvent>()
            .add_event::<BarberZapGunDifficultyEvent>()
            .add_event::<BarberContactKillEvent>()
            .add_event::<BarberIgnoredDamageEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_barber,
                    barber_update_jump_interval,
                    barber_roam_between_two_areas,
                    barber_follow_when_seen_from_front,
                    barber_lock_track_on_snare_roll,
                    barber_lunge_on_snare_hit,
                    barber_continue_then_roam_when_unseen,
                    barber_door_attempt_speed,
                    barber_apply_stun_multiplier,
                    barber_report_zap_gun_difficulty,
                    barber_contact_instant_kill,
                    barber_ignore_damage,
                    barber_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Barber;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BarberEmployeeSensor {
    pub stable_id: u64,
    pub can_fully_see_barber: bool,
    pub is_in_front_of_barber: bool,
    pub can_see_pursuing_barber: bool,
    pub touching_barber: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberRoamRoute {
    pub area_a: SimPosition,
    pub area_b: SimPosition,
    pub destination_index: u8,
}

impl Default for BarberRoamRoute {
    fn default() -> Self {
        Self {
            area_a: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            area_b: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            destination_index: 1,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberJumpTiming {
    pub interval_ticks: u32,
    pub timer_ticks: u32,
}

impl Default for BarberJumpTiming {
    fn default() -> Self {
        Self {
            interval_ticks: 69,
            timer_ticks: 69,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberTrack {
    pub has_target: bool,
    pub target_stable_id: u64,
    pub tracked_position: SimPosition,
    pub unseen_lunges_remaining: u8,
}

impl Default for BarberTrack {
    fn default() -> Self {
        Self {
            has_target: false,
            target_stable_id: 0,
            tracked_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            unseen_lunges_remaining: BARBER_UNSEEN_LUNGE_COUNT,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarberState {
    #[default]
    Roaming,
    Following,
}

#[derive(Bundle)]
pub struct BarberBundle {
    pub name: Name,
    pub barber: Barber,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: BarberState,
    pub route: BarberRoamRoute,
    pub jump_timing: BarberJumpTiming,
    pub track: BarberTrack,
}

impl BarberBundle {
    pub fn new(event: SpawnBarberEvent) -> Self {
        Self {
            name: Name::new(BARBER_NAME),
            barber: Barber,
            position: event.position,
            health: Health::full(BARBER_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: BARBER_ROAM_SPEED,
                attack_range: I32F32::lit("0"),
                attack_damage: BARBER_INSTANT_KILL_DAMAGE,
                attack_speed: I32F32::lit("2"),
                watch_range: BARBER_WATCH_RANGE,
            },
            state: BarberState::Roaming,
            route: BarberRoamRoute {
                area_a: event.roam_area_a,
                area_b: event.roam_area_b,
                destination_index: 1,
            },
            jump_timing: BarberJumpTiming::default(),
            track: BarberTrack {
                has_target: false,
                target_stable_id: 0,
                tracked_position: event.position,
                unseen_lunges_remaining: BARBER_UNSEEN_LUNGE_COUNT,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnBarberEvent {
    pub position: SimPosition,
    pub roam_area_a: SimPosition,
    pub roam_area_b: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberStateChangedEvent {
    pub barber: Entity,
    pub from: BarberState,
    pub to: BarberState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberSnareDrumRollEvent {
    pub barber: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberSnareHitEvent {
    pub barber: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberDoorAttemptEvent {
    pub barber: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberDoorAttemptResolvedEvent {
    pub barber: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
    pub likely_failed_before_next_jump: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberStunAppliedEvent {
    pub barber: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberStunAdjustedEvent {
    pub barber: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberZapGunTargetedEvent {
    pub barber: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberZapGunDifficultyEvent {
    pub barber: Entity,
    pub difficulty_modifier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberContactKillEvent {
    pub barber: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarberIgnoredDamageEvent {
    pub barber: Entity,
    pub source: Entity,
}

fn spawn_barber(
    mut commands: Commands,
    mut events: EventReader<SpawnBarberEvent>,
    barbers: Query<(), With<Barber>>,
) {
    let mut spawned_count = barbers.iter().count();

    for event in events.read() {
        if spawned_count >= BARBER_MAX_SPAWNED {
            break;
        }

        commands.spawn(BarberBundle::new(*event));
        spawned_count += 1;
    }
}

fn barber_update_jump_interval(
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut barbers: Query<&mut BarberJumpTiming, With<Barber>>,
) {
    let elapsed_seconds = I32F32::from_num(sim_tick.0) / sim_hz.0;
    let day_window_seconds = I32F32::from_num(
        (BARBER_DAY_END_MINUTE - BARBER_DAY_START_MINUTE) as u32 * 60,
    );
    let clamped_elapsed = fixed_clamp(
        elapsed_seconds,
        I32F32::lit("0"),
        day_window_seconds,
    );
    let interval_delta = BARBER_START_JUMP_INTERVAL_SECONDS - BARBER_END_JUMP_INTERVAL_SECONDS;
    let interval_seconds =
        BARBER_START_JUMP_INTERVAL_SECONDS - (interval_delta * clamped_elapsed / day_window_seconds);
    let interval_ticks = fixed_seconds_to_ticks(interval_seconds, sim_hz.0);

    for mut timing in barbers.iter_mut() {
        timing.interval_ticks = interval_ticks;
        if timing.timer_ticks > timing.interval_ticks {
            timing.timer_ticks = timing.interval_ticks;
        }
    }
}

fn barber_roam_between_two_areas(
    sim_hz: Res<SimHz>,
    mut barbers: Query<
        (
            Entity,
            &mut SimPosition,
            &BarberState,
            &UnitStats,
            &mut BarberRoamRoute,
            &mut BarberJumpTiming,
        ),
        With<Barber>,
    >,
) {
    for (_entity, mut position, state, stats, mut route, mut timing) in barbers.iter_mut() {
        if *state != BarberState::Roaming {
            continue;
        }

        if timing.timer_ticks > 0 {
            timing.timer_ticks -= 1;
            continue;
        }

        let destination = if route.destination_index == 0 {
            route.area_a
        } else {
            route.area_b
        };

        move_axis_toward(&mut position, destination, stats.move_speed / sim_hz.0);
        timing.timer_ticks = timing.interval_ticks;

        if *position == destination {
            route.destination_index = 1 - route.destination_index;
        }
    }
}

fn barber_follow_when_seen_from_front(
    mut state_events: EventWriter<BarberStateChangedEvent>,
    mut barbers: Query<(Entity, &mut BarberState, &mut UnitStats, &mut BarberTrack), With<Barber>>,
    employees: Query<(&SimPosition, &BarberEmployeeSensor)>,
) {
    let Some((employee_position, employee_sensor)) = visible_front_employee(&employees) else {
        return;
    };

    for (barber_entity, mut state, mut stats, mut track) in barbers.iter_mut() {
        if *state != BarberState::Roaming {
            continue;
        }

        track.has_target = true;
        track.target_stable_id = employee_sensor.stable_id;
        track.tracked_position = employee_position;
        track.unseen_lunges_remaining = BARBER_UNSEEN_LUNGE_COUNT;
        stats.move_speed = BARBER_FOLLOW_SPEED;

        set_barber_state(
            barber_entity,
            &mut state,
            BarberState::Following,
            &mut state_events,
        );
    }
}

fn barber_lock_track_on_snare_roll(
    mut events: EventReader<BarberSnareDrumRollEvent>,
    mut barbers: Query<&mut BarberTrack, With<Barber>>,
    employees: Query<(&SimPosition, &BarberEmployeeSensor)>,
) {
    for event in events.read() {
        let Ok(mut track) = barbers.get_mut(event.barber) else {
            continue;
        };

        if !track.has_target {
            continue;
        }

        if let Some(position) = employee_position_by_stable_id(track.target_stable_id, &employees) {
            track.tracked_position = position;
        }
    }
}

fn barber_lunge_on_snare_hit(
    mut events: EventReader<BarberSnareHitEvent>,
    mut barbers: Query<
        (
            &mut SimPosition,
            &BarberState,
            &mut BarberJumpTiming,
            &mut BarberTrack,
        ),
        With<Barber>,
    >,
) {
    for event in events.read() {
        let Ok((mut position, state, mut timing, track)) = barbers.get_mut(event.barber) else {
            continue;
        };

        if *state != BarberState::Following || !track.has_target {
            continue;
        }

        move_axis_toward(&mut position, track.tracked_position, BARBER_LUNGE_DISTANCE);
        timing.timer_ticks = timing.interval_ticks;
    }
}

fn barber_continue_then_roam_when_unseen(
    mut state_events: EventWriter<BarberStateChangedEvent>,
    mut barbers: Query<(Entity, &mut BarberState, &mut UnitStats, &mut BarberTrack), With<Barber>>,
    employees: Query<&BarberEmployeeSensor>,
) {
    for (barber_entity, mut state, mut stats, mut track) in barbers.iter_mut() {
        if *state != BarberState::Following || !track.has_target {
            continue;
        }

        let target_visible = employees
            .iter()
            .any(|employee| employee.stable_id == track.target_stable_id && employee.can_see_pursuing_barber);

        if target_visible {
            track.unseen_lunges_remaining = BARBER_UNSEEN_LUNGE_COUNT;
            continue;
        }

        if track.unseen_lunges_remaining > 0 {
            track.unseen_lunges_remaining -= 1;
            continue;
        }

        track.has_target = false;
        track.target_stable_id = 0;
        track.unseen_lunges_remaining = BARBER_UNSEEN_LUNGE_COUNT;
        stats.move_speed = BARBER_ROAM_SPEED;

        set_barber_state(
            barber_entity,
            &mut state,
            BarberState::Roaming,
            &mut state_events,
        );
    }
}

fn barber_door_attempt_speed(
    mut events: EventReader<BarberDoorAttemptEvent>,
    mut resolved_events: EventWriter<BarberDoorAttemptResolvedEvent>,
    barbers: Query<&BarberJumpTiming, With<Barber>>,
) {
    for event in events.read() {
        let Ok(timing) = barbers.get(event.barber) else {
            continue;
        };

        let adjusted_open_ticks = fixed_ticks_scaled(
            event.base_open_ticks,
            BARBER_DOOR_SPEED_MULTIPLIER,
        );
        resolved_events.send(BarberDoorAttemptResolvedEvent {
            barber: event.barber,
            door: event.door,
            adjusted_open_ticks,
            likely_failed_before_next_jump: adjusted_open_ticks > timing.timer_ticks,
        });
    }
}

fn barber_apply_stun_multiplier(
    mut events: EventReader<BarberStunAppliedEvent>,
    mut adjusted_events: EventWriter<BarberStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(BarberStunAdjustedEvent {
            barber: event.barber,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, BARBER_STUN_MULTIPLIER),
        });
    }
}

fn barber_report_zap_gun_difficulty(
    mut events: EventReader<BarberZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<BarberZapGunDifficultyEvent>,
    barbers: Query<(), With<Barber>>,
) {
    for event in events.read() {
        if barbers.get(event.barber).is_err() {
            continue;
        }

        difficulty_events.send(BarberZapGunDifficultyEvent {
            barber: event.barber,
            difficulty_modifier: BARBER_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn barber_contact_instant_kill(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<BarberContactKillEvent>,
    barbers: Query<(Entity, &SimPosition), With<Barber>>,
    employees: Query<(Entity, &BarberEmployeeSensor)>,
) {
    for (barber_entity, _barber_position) in barbers.iter() {
        for (employee_entity, employee_sensor) in employees.iter() {
            if !employee_sensor.touching_barber {
                continue;
            }

            contact_events.send(BarberContactKillEvent {
                barber: barber_entity,
                employee: employee_entity,
                employee_stable_id: employee_sensor.stable_id,
            });
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: BARBER_INSTANT_KILL_DAMAGE,
                damage_type: DamageType::Standard,
                source: barber_entity,
            });
        }
    }
}

fn barber_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<BarberIgnoredDamageEvent>,
    barbers: Query<(), With<Barber>>,
) {
    for event in damage_events.read() {
        if barbers.get(event.target).is_err() {
            continue;
        }

        ignored_events.send(BarberIgnoredDamageEvent {
            barber: event.target,
            source: event.source,
        });
    }
}

fn barber_checksum(
    mut checksum: ResMut<SimChecksumState>,
    barbers: Query<(
        &SimPosition,
        &Health,
        &UnitStats,
        &BarberState,
        &BarberRoamRoute,
        &BarberJumpTiming,
        &BarberTrack,
    ), With<Barber>>,
) {
    for (position, health, stats, state, route, timing, track) in barbers.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(barber_state_bits(*state));
        checksum.accumulate(route.area_a.x.to_bits() as u64);
        checksum.accumulate(route.area_a.y.to_bits() as u64);
        checksum.accumulate(route.area_b.x.to_bits() as u64);
        checksum.accumulate(route.area_b.y.to_bits() as u64);
        checksum.accumulate(route.destination_index as u64);
        checksum.accumulate(timing.interval_ticks as u64);
        checksum.accumulate(timing.timer_ticks as u64);
        checksum.accumulate(track.has_target as u64);
        checksum.accumulate(track.target_stable_id);
        checksum.accumulate(track.tracked_position.x.to_bits() as u64);
        checksum.accumulate(track.tracked_position.y.to_bits() as u64);
        checksum.accumulate(track.unseen_lunges_remaining as u64);
    }
}

fn visible_front_employee(
    employees: &Query<(&SimPosition, &BarberEmployeeSensor)>,
) -> Option<(SimPosition, BarberEmployeeSensor)> {
    let mut best: Option<(SimPosition, BarberEmployeeSensor)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.can_fully_see_barber || !sensor.is_in_front_of_barber {
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

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &Query<(&SimPosition, &BarberEmployeeSensor)>,
) -> Option<SimPosition> {
    for (position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some(*position);
        }
    }

    None
}

fn set_barber_state(
    barber: Entity,
    state: &mut BarberState,
    next: BarberState,
    events: &mut EventWriter<BarberStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(BarberStateChangedEvent {
        barber,
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

fn barber_state_bits(state: BarberState) -> u64 {
    match state {
        BarberState::Roaming => 0,
        BarberState::Following => 1,
    }
}