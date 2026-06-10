// Sources: vault/harmless_entity_pages/manticoil.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    EntityKilledEvent, Health, SimChecksumState, SimHz, SimPosition, SimTick, UnitStats,
};

pub const MANTICOIL_ID: &str = "manticoil";
pub const MANTICOIL_NAME: &str = "Manticoil";
pub const MANTICOIL_TYPE: &str = "harmless_entity_pages";
pub const MANTICOIL_SUBTYPE: &str = "bestiary";
pub const MANTICOIL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Manticoil";
pub const MANTICOIL_SOURCE_REVISION: u32 = 19423;
pub const MANTICOIL_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const MANTICOIL_CONFIDENCE_BASIS_POINTS: u16 = 88;

pub const MANTICOIL_DWELLS: &str = "Outside (Daytime)";
pub const MANTICOIL_DANGER: &str = "Harmless";
pub const MANTICOIL_SCIENTIFIC_NAME: &str = "Bird";
pub const MANTICOIL_HP: I32F32 = I32F32::lit("1");
pub const MANTICOIL_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const MANTICOIL_MAX_SPAWNED: usize = 16;
pub const MANTICOIL_STUN_MULTIPLIER: I32F32 = I32F32::lit("1");
pub const MANTICOIL_STUN_GRENADE: bool = true;
pub const MANTICOIL_RADAR_PIP_SIZE: &str = "Tiny";
pub const MANTICOIL_SHOVEL_HP: &str = "1 Hit";
pub const MANTICOIL_CAN_SEE_THROUGH_FOG: bool = false;
pub const MANTICOIL_LEAVE_TIME: &str = "11:58 AM";
pub const MANTICOIL_LEAVE_MINUTES_AFTER_MIDNIGHT: u16 = 718;

pub const MANTICOIL_DEPENDS_ON: [&str; 7] = [
    "shovel",
    "stop_sign",
    "yield_sign",
    "kitchen_knife",
    "double_barrel",
    "stun_grenade",
    "old_bird",
];

pub const MANTICOIL_FRONTMATTER_BEHAVIOR: [&str; 1] = ["Roaming"];

pub const MANTICOIL_SPAWN_CHANCES: [ManticoilMoonSpawnChance; 7] = [
    ManticoilMoonSpawnChance {
        moon: "Offense",
        base_spawn_chance: I32F32::lit("100.0"),
    },
    ManticoilMoonSpawnChance {
        moon: "Adamance",
        base_spawn_chance: I32F32::lit("55.26"),
    },
    ManticoilMoonSpawnChance {
        moon: "Vow",
        base_spawn_chance: I32F32::lit("51.81"),
    },
    ManticoilMoonSpawnChance {
        moon: "Experimentation",
        base_spawn_chance: I32F32::lit("50.0"),
    },
    ManticoilMoonSpawnChance {
        moon: "Assurance",
        base_spawn_chance: I32F32::lit("50.0"),
    },
    ManticoilMoonSpawnChance {
        moon: "Artifice",
        base_spawn_chance: I32F32::lit("46.88"),
    },
    ManticoilMoonSpawnChance {
        moon: "March",
        base_spawn_chance: I32F32::lit("43.01"),
    },
];

pub const MANTICOIL_APPROACH_FLEE_RADIUS: I32F32 = I32F32::lit("6");
pub const MANTICOIL_SOUND_RESPONSE_THRESHOLD: I32F32 = I32F32::lit("1");
pub const MANTICOIL_OLD_BIRD_MISSILE_HEARING_RADIUS: I32F32 = I32F32::lit("35");
pub const MANTICOIL_STUN_SPIN_SECONDS: I32F32 = I32F32::lit("5");
pub const MANTICOIL_TAKEOFF_SPEED: I32F32 = I32F32::lit("18");
pub const MANTICOIL_ROAM_SPEED: I32F32 = I32F32::lit("2");
pub const MANTICOIL_ATTACK_DAMAGE: I32F32 = I32F32::lit("0");
pub const MANTICOIL_ATTACK_RANGE: I32F32 = I32F32::lit("0");
pub const MANTICOIL_ATTACK_SPEED: I32F32 = I32F32::lit("0");

pub const MANTICOIL_BEHAVIORAL_MECHANICS: [ManticoilBehaviorRule; 7] = [
    ManticoilBehaviorRule {
        condition: "a player approaches or frightens it",
        outcome: "it disperses into the sky and flees",
    },
    ManticoilBehaviorRule {
        condition: "it hears a sound above its response threshold",
        outcome: "it takes off",
    },
    ManticoilBehaviorRule {
        condition: "it is struck by shovel, stop_sign, yield_sign, or kitchen_knife",
        outcome: "it dies in 1 hit",
    },
    ManticoilBehaviorRule {
        condition: "it is hit by double_barrel",
        outcome: "it takes no damage",
    },
    ManticoilBehaviorRule {
        condition: "it is affected by a stun_grenade",
        outcome: "it can be stunned and spins in circles",
    },
    ManticoilBehaviorRule {
        condition: "11:58 AM occurs",
        outcome: "it leaves the map",
    },
    ManticoilBehaviorRule {
        condition: "old_bird missile fire is heard nearby",
        outcome: "nearby Manticoils usually take off immediately",
    },
];

pub struct ManticoilPlugin;

impl Plugin for ManticoilPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnManticoilEvent>()
            .add_event::<ManticoilApproachedEvent>()
            .add_event::<ManticoilFrightenedEvent>()
            .add_event::<ManticoilSoundHeardEvent>()
            .add_event::<ManticoilWeaponHitEvent>()
            .add_event::<ManticoilStunGrenadeEvent>()
            .add_event::<ManticoilOldBirdMissileHeardEvent>()
            .add_event::<ManticoilLeaveTimeEvent>()
            .add_event::<ManticoilStateChangedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_manticoil,
                    manticoil_approach_or_frighten_to_flee,
                    manticoil_sound_to_takeoff,
                    manticoil_weapon_damage_rules,
                    manticoil_stun_grenade_spin,
                    manticoil_leave_at_time,
                    manticoil_old_bird_missile_takeoff,
                    manticoil_stun_timer,
                    manticoil_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManticoilBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManticoilMoonSpawnChance {
    pub moon: &'static str,
    pub base_spawn_chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Manticoil;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManticoilEmployeeTarget {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManticoilTimers {
    pub stun_ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManticoilMovement {
    pub flee_origin: SimPosition,
    pub takeoff_speed: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ManticoilState {
    #[default]
    Roaming = 0,
    Fleeing = 1,
    StunnedSpinning = 2,
    LeavingMap = 3,
    Dead = 4,
}

#[derive(Bundle)]
pub struct ManticoilBundle {
    pub name: Name,
    pub manticoil: Manticoil,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: ManticoilState,
    pub timers: ManticoilTimers,
    pub movement: ManticoilMovement,
}

impl ManticoilBundle {
    pub fn new(event: SpawnManticoilEvent) -> Self {
        Self {
            name: Name::new(MANTICOIL_NAME),
            manticoil: Manticoil,
            position: event.position,
            health: Health::full(MANTICOIL_HP),
            stats: UnitStats {
                move_speed: MANTICOIL_ROAM_SPEED,
                attack_range: MANTICOIL_ATTACK_RANGE,
                attack_damage: MANTICOIL_ATTACK_DAMAGE,
                attack_speed: MANTICOIL_ATTACK_SPEED,
                watch_range: MANTICOIL_APPROACH_FLEE_RADIUS,
            },
            state: ManticoilState::Roaming,
            timers: ManticoilTimers {
                stun_ticks_remaining: 0,
            },
            movement: ManticoilMovement {
                flee_origin: event.position,
                takeoff_speed: MANTICOIL_TAKEOFF_SPEED,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnManticoilEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct ManticoilApproachedEvent {
    pub manticoil: Entity,
    pub employee: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct ManticoilFrightenedEvent {
    pub manticoil: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct ManticoilSoundHeardEvent {
    pub position: SimPosition,
    pub loudness: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManticoilWeaponHitEvent {
    pub manticoil: Entity,
    pub source: Option<Entity>,
    pub weapon_id: &'static str,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct ManticoilStunGrenadeEvent {
    pub manticoil: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct ManticoilOldBirdMissileHeardEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct ManticoilLeaveTimeEvent {
    pub minutes_after_midnight: u16,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManticoilStateChangedEvent {
    pub manticoil: Entity,
    pub from: ManticoilState,
    pub to: ManticoilState,
}

fn spawn_manticoil(
    mut commands: Commands,
    mut events: EventReader<SpawnManticoilEvent>,
    manticoils: Query<(), With<Manticoil>>,
) {
    let mut spawned_count = manticoils.iter().count();

    for event in events.read() {
        if spawned_count >= MANTICOIL_MAX_SPAWNED {
            break;
        }

        commands.spawn(ManticoilBundle::new(*event));
        spawned_count += 1;
    }
}

fn manticoil_approach_or_frighten_to_flee(
    mut approached_events: EventReader<ManticoilApproachedEvent>,
    mut frightened_events: EventReader<ManticoilFrightenedEvent>,
    mut state_events: EventWriter<ManticoilStateChangedEvent>,
    mut manticoils: Query<(Entity, &SimPosition, &mut ManticoilState, &mut UnitStats, &mut ManticoilMovement), With<Manticoil>>,
) {
    for event in approached_events.read() {
        if let Ok((entity, position, mut state, mut stats, mut movement)) =
            manticoils.get_mut(event.manticoil)
        {
            let _ = event.employee;
            flee_from_position(entity, *position, &mut state, &mut stats, &mut movement, &mut state_events);
        }
    }

    for event in frightened_events.read() {
        if let Ok((entity, position, mut state, mut stats, mut movement)) =
            manticoils.get_mut(event.manticoil)
        {
            let _ = event.source;
            flee_from_position(entity, *position, &mut state, &mut stats, &mut movement, &mut state_events);
        }
    }
}

fn manticoil_sound_to_takeoff(
    mut sound_events: EventReader<ManticoilSoundHeardEvent>,
    mut state_events: EventWriter<ManticoilStateChangedEvent>,
    mut manticoils: Query<(Entity, &SimPosition, &mut ManticoilState, &mut UnitStats, &mut ManticoilMovement), With<Manticoil>>,
) {
    for event in sound_events.read() {
        if event.loudness < MANTICOIL_SOUND_RESPONSE_THRESHOLD {
            continue;
        }

        for (entity, position, mut state, mut stats, mut movement) in manticoils.iter_mut() {
            if *state == ManticoilState::Dead || *state == ManticoilState::LeavingMap {
                continue;
            }

            let _ = event.position;
            flee_from_position(entity, *position, &mut state, &mut stats, &mut movement, &mut state_events);
        }
    }
}

fn manticoil_weapon_damage_rules(
    mut weapon_events: EventReader<ManticoilWeaponHitEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    mut state_events: EventWriter<ManticoilStateChangedEvent>,
    mut manticoils: Query<(Entity, &mut Health, &mut ManticoilState), With<Manticoil>>,
) {
    for event in weapon_events.read() {
        let Ok((entity, mut health, mut state)) = manticoils.get_mut(event.manticoil) else {
            continue;
        };

        if event.weapon_id == "double_barrel" {
            continue;
        }

        if matches!(
            event.weapon_id,
            "shovel" | "stop_sign" | "yield_sign" | "kitchen_knife"
        ) {
            health.current = I32F32::lit("0");
            set_state(entity, &mut state, ManticoilState::Dead, &mut state_events);
            killed_events.send(EntityKilledEvent {
                entity,
                killer: event.source.unwrap_or(entity),
                exp_reward: I32F32::lit("0"),
                difficulty_tier: 0,
            });
        }
    }
}

fn manticoil_stun_grenade_spin(
    sim_hz: Res<SimHz>,
    mut stun_events: EventReader<ManticoilStunGrenadeEvent>,
    mut state_events: EventWriter<ManticoilStateChangedEvent>,
    mut manticoils: Query<(Entity, &mut ManticoilState, &mut UnitStats, &mut ManticoilTimers), With<Manticoil>>,
) {
    let stun_ticks = seconds_to_ticks(MANTICOIL_STUN_SPIN_SECONDS * MANTICOIL_STUN_MULTIPLIER, sim_hz.0);

    for event in stun_events.read() {
        let Ok((entity, mut state, mut stats, mut timers)) = manticoils.get_mut(event.manticoil)
        else {
            continue;
        };

        if *state == ManticoilState::Dead || *state == ManticoilState::LeavingMap {
            continue;
        }

        timers.stun_ticks_remaining = stun_ticks;
        stats.move_speed = I32F32::lit("0");
        set_state(entity, &mut state, ManticoilState::StunnedSpinning, &mut state_events);
    }
}

fn manticoil_leave_at_time(
    mut leave_events: EventReader<ManticoilLeaveTimeEvent>,
    mut state_events: EventWriter<ManticoilStateChangedEvent>,
    mut manticoils: Query<(Entity, &SimPosition, &mut ManticoilState, &mut UnitStats, &mut ManticoilMovement), With<Manticoil>>,
) {
    for event in leave_events.read() {
        if event.minutes_after_midnight != MANTICOIL_LEAVE_MINUTES_AFTER_MIDNIGHT {
            continue;
        }

        for (entity, position, mut state, mut stats, mut movement) in manticoils.iter_mut() {
            if *state == ManticoilState::Dead {
                continue;
            }

            movement.flee_origin = *position;
            stats.move_speed = MANTICOIL_TAKEOFF_SPEED;
            set_state(entity, &mut state, ManticoilState::LeavingMap, &mut state_events);
        }
    }
}

fn manticoil_old_bird_missile_takeoff(
    mut missile_events: EventReader<ManticoilOldBirdMissileHeardEvent>,
    mut state_events: EventWriter<ManticoilStateChangedEvent>,
    mut manticoils: Query<(Entity, &SimPosition, &mut ManticoilState, &mut UnitStats, &mut ManticoilMovement), With<Manticoil>>,
) {
    let hearing_radius_squared =
        MANTICOIL_OLD_BIRD_MISSILE_HEARING_RADIUS * MANTICOIL_OLD_BIRD_MISSILE_HEARING_RADIUS;

    for event in missile_events.read() {
        for (entity, position, mut state, mut stats, mut movement) in manticoils.iter_mut() {
            if *state == ManticoilState::Dead || *state == ManticoilState::LeavingMap {
                continue;
            }

            if distance_squared(*position, event.position) > hearing_radius_squared {
                continue;
            }

            flee_from_position(entity, *position, &mut state, &mut stats, &mut movement, &mut state_events);
        }
    }
}

fn manticoil_stun_timer(
    mut state_events: EventWriter<ManticoilStateChangedEvent>,
    mut manticoils: Query<(Entity, &mut ManticoilState, &mut UnitStats, &mut ManticoilTimers), With<Manticoil>>,
) {
    for (entity, mut state, mut stats, mut timers) in manticoils.iter_mut() {
        if *state != ManticoilState::StunnedSpinning {
            continue;
        }

        timers.stun_ticks_remaining = timers.stun_ticks_remaining.saturating_sub(1);

        if timers.stun_ticks_remaining == 0 {
            stats.move_speed = MANTICOIL_ROAM_SPEED;
            set_state(entity, &mut state, ManticoilState::Roaming, &mut state_events);
        }
    }
}

fn manticoil_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    manticoils: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &ManticoilState,
            &ManticoilTimers,
            &ManticoilMovement,
        ),
        With<Manticoil>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(MANTICOIL_SOURCE_REVISION as u64);
    checksum.accumulate(MANTICOIL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(MANTICOIL_HP.to_bits() as u64);
    checksum.accumulate(MANTICOIL_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(MANTICOIL_MAX_SPAWNED as u64);
    checksum.accumulate(MANTICOIL_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(MANTICOIL_STUN_GRENADE as u64);
    checksum.accumulate(MANTICOIL_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(MANTICOIL_LEAVE_MINUTES_AFTER_MIDNIGHT as u64);
    checksum.accumulate(MANTICOIL_APPROACH_FLEE_RADIUS.to_bits() as u64);
    checksum.accumulate(MANTICOIL_SOUND_RESPONSE_THRESHOLD.to_bits() as u64);
    checksum.accumulate(MANTICOIL_OLD_BIRD_MISSILE_HEARING_RADIUS.to_bits() as u64);
    checksum.accumulate(MANTICOIL_STUN_SPIN_SECONDS.to_bits() as u64);
    checksum.accumulate(MANTICOIL_TAKEOFF_SPEED.to_bits() as u64);
    checksum.accumulate(MANTICOIL_ROAM_SPEED.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, MANTICOIL_ID);
    accumulate_str(&mut checksum, 0x1001, MANTICOIL_NAME);
    accumulate_str(&mut checksum, 0x1002, MANTICOIL_TYPE);
    accumulate_str(&mut checksum, 0x1003, MANTICOIL_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, MANTICOIL_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, MANTICOIL_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, MANTICOIL_DWELLS);
    accumulate_str(&mut checksum, 0x1007, MANTICOIL_DANGER);
    accumulate_str(&mut checksum, 0x1008, MANTICOIL_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1009, MANTICOIL_RADAR_PIP_SIZE);
    accumulate_str(&mut checksum, 0x100a, MANTICOIL_SHOVEL_HP);
    accumulate_str(&mut checksum, 0x100b, MANTICOIL_LEAVE_TIME);

    for dependency in MANTICOIL_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in MANTICOIL_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for spawn_chance in MANTICOIL_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3500, spawn_chance.moon);
        checksum.accumulate(spawn_chance.base_spawn_chance.to_bits() as u64);
    }

    for rule in MANTICOIL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (position, health, stats, state, timers, movement) in manticoils.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(*state as u64);
        checksum.accumulate(timers.stun_ticks_remaining as u64);
        checksum.accumulate(movement.flee_origin.x.to_bits() as u64);
        checksum.accumulate(movement.flee_origin.y.to_bits() as u64);
        checksum.accumulate(movement.takeoff_speed.to_bits() as u64);
    }
}

fn flee_from_position(
    entity: Entity,
    position: SimPosition,
    state: &mut ManticoilState,
    stats: &mut UnitStats,
    movement: &mut ManticoilMovement,
    state_events: &mut EventWriter<ManticoilStateChangedEvent>,
) {
    if *state == ManticoilState::Dead || *state == ManticoilState::LeavingMap {
        return;
    }

    movement.flee_origin = position;
    stats.move_speed = movement.takeoff_speed;
    set_state(entity, state, ManticoilState::Fleeing, state_events);
}

fn set_state(
    entity: Entity,
    state: &mut ManticoilState,
    to: ManticoilState,
    events: &mut EventWriter<ManticoilStateChangedEvent>,
) {
    if *state == to {
        return;
    }

    let from = *state;
    *state = to;
    events.send(ManticoilStateChangedEvent {
        manticoil: entity,
        from,
        to,
    });
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = (seconds * sim_hz).ceil().to_num::<u32>();

    if ticks == 0 {
        1
    } else {
        ticks
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}