// Sources: vault/indoor_entity_pages/nutcrackers.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, UnitStats,
};

pub const NUTCRACKERS_ID: &str = "nutcrackers";
pub const NUTCRACKERS_NAME: &str = "Nutcrackers";
pub const NUTCRACKERS_TYPE: &str = "indoor_entity_pages";
pub const NUTCRACKERS_SUBTYPE: &str = "creature";
pub const NUTCRACKERS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Nutcracker";
pub const NUTCRACKERS_SOURCE_REVISION: u32 = 21470;
pub const NUTCRACKERS_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const NUTCRACKERS_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const NUTCRACKERS_DWELLS: &str = "Inside";
pub const NUTCRACKERS_SHOCK_RESPONSE: &str = "Susceptible";
pub const NUTCRACKERS_RADAR_PIP_SIZE: &str = "Medium + Scrap";
pub const NUTCRACKERS_SHOVEL_HP: I32F32 = I32F32::lit("5");
pub const NUTCRACKERS_ATTACK_DAMAGE: &str = "5 - 100";
pub const NUTCRACKERS_ATTACK_DAMAGE_MIN: I32F32 = I32F32::lit("5");
pub const NUTCRACKERS_ATTACK_DAMAGE_MAX: I32F32 = I32F32::lit("100");
pub const NUTCRACKERS_ATTACK_SPEED: &str = "Shooting ~1 s; Reloading ~1.5 s; Kicking Instant";
pub const NUTCRACKERS_DPS: &str = "Variable";
pub const NUTCRACKERS_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const NUTCRACKERS_MAX_SPAWNED: usize = 10;
pub const NUTCRACKERS_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.5");
pub const NUTCRACKERS_CAN_SEE_THROUGH_FOG: bool = true;
pub const NUTCRACKERS_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("2");
pub const NUTCRACKERS_CONTACT_DAMAGE: &str = "Instant Kill";
pub const NUTCRACKERS_INTERNAL_NAME: &str = "Nutcracker";

pub const NUTCRACKERS_SCAN_FOV_DEGREES: i16 = 70;
pub const NUTCRACKERS_SCAN_ROTATION_INCREMENT_DEGREES: i16 = 30;
pub const NUTCRACKERS_SHOOT_TICKS_AT_20HZ: u32 = 20;
pub const NUTCRACKERS_RELOAD_TICKS_AT_20HZ: u32 = 30;
pub const NUTCRACKERS_ENRAGED_SHOOT_TICKS_AT_20HZ: u32 = 10;
pub const NUTCRACKERS_SHOTGUN_SHELLS_DROPPED: u8 = 2;
pub const NUTCRACKERS_SHOTGUN_CAPACITY: u8 = 2;
pub const NUTCRACKERS_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("1000000");

pub const NUTCRACKERS_DEPENDS_ON: [&str; 0] = [];
pub const NUTCRACKERS_FRONTMATTER_BEHAVIOR: [&str; 3] = ["Scanning", "Chasing", "Roaming"];

pub const NUTCRACKERS_BEHAVIORAL_MECHANICS: [NutcrackersBehaviorRule; 8] = [
    NutcrackersBehaviorRule {
        condition: "the Nutcracker is in patrol state",
        outcome: "it is invincible",
    },
    NutcrackersBehaviorRule {
        condition: "the Nutcracker stops to scan",
        outcome: "it lifts its head and exposes a single eye",
    },
    NutcrackersBehaviorRule {
        condition: "the eye is exposed",
        outcome: "it scans for movement within a 70-degree field of view and rotates in 30-degree increments",
    },
    NutcrackersBehaviorRule {
        condition: "the Nutcracker detects movement",
        outcome: "it targets the moving entity and fires its shotgun",
    },
    NutcrackersBehaviorRule {
        condition: "an entity comes too close",
        outcome: "the Nutcracker kicks it for instant kill contact damage",
    },
    NutcrackersBehaviorRule {
        condition: "death",
        outcome: "the Nutcracker drops its shotgun and 2 guaranteed shotgun shells",
    },
    NutcrackersBehaviorRule {
        condition: "the Nutcracker is killed after reloading",
        outcome: "its dropped shotgun can contain the shells that remained in it at the moment of death",
    },
    NutcrackersBehaviorRule {
        condition: "the Nutcracker deals the last hit of damage",
        outcome: "it enters an enraged state with a shorter delay between shots",
    },
];

pub struct NutcrackersPlugin;

impl Plugin for NutcrackersPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnNutcrackersEvent>()
            .add_event::<NutcrackersStateChangedEvent>()
            .add_event::<NutcrackersScanStartedEvent>()
            .add_event::<NutcrackersMovementDetectedEvent>()
            .add_event::<NutcrackersShotgunFiredEvent>()
            .add_event::<NutcrackersReloadStartedEvent>()
            .add_event::<NutcrackersContactKickEvent>()
            .add_event::<NutcrackersDeathDropEvent>()
            .add_event::<NutcrackersEnragedEvent>()
            .add_event::<NutcrackersIgnoredDamageEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_nutcrackers,
                    nutcrackers_begin_scan,
                    nutcrackers_rotate_scan,
                    nutcrackers_detect_movement,
                    nutcrackers_fire_shotgun,
                    nutcrackers_reload,
                    nutcrackers_contact_kick,
                    nutcrackers_ignore_protected_damage,
                    nutcrackers_death_drops,
                    nutcrackers_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nutcrackers;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NutcrackersTargetSensor {
    pub stable_id: u64,
    pub moved_this_tick: bool,
    pub in_scan_fov: bool,
    pub too_close: bool,
    pub visible_through_fog: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersScan {
    pub eye_exposed: bool,
    pub facing_degrees: i16,
    pub scan_timer_ticks: u32,
}

impl Default for NutcrackersScan {
    fn default() -> Self {
        Self {
            eye_exposed: false,
            facing_degrees: 0,
            scan_timer_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersShotgun {
    pub shells_loaded: u8,
    pub shoot_cooldown_ticks: u32,
    pub reload_timer_ticks: u32,
    pub killed_after_reloading: bool,
}

impl Default for NutcrackersShotgun {
    fn default() -> Self {
        Self {
            shells_loaded: NUTCRACKERS_SHOTGUN_CAPACITY,
            shoot_cooldown_ticks: 0,
            reload_timer_ticks: 0,
            killed_after_reloading: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersTarget {
    pub has_target: bool,
    pub target: Entity,
    pub target_stable_id: u64,
}

impl Default for NutcrackersTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target: Entity::PLACEHOLDER,
            target_stable_id: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NutcrackersState {
    #[default]
    Patrol,
    Scanning,
    Chasing,
    Reloading,
    Enraged,
}

#[derive(Bundle)]
pub struct NutcrackersBundle {
    pub name: Name,
    pub nutcrackers: Nutcrackers,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: NutcrackersState,
    pub scan: NutcrackersScan,
    pub shotgun: NutcrackersShotgun,
    pub target: NutcrackersTarget,
}

impl NutcrackersBundle {
    pub fn new(event: SpawnNutcrackersEvent) -> Self {
        Self {
            name: Name::new(NUTCRACKERS_NAME),
            nutcrackers: Nutcrackers,
            position: event.position,
            health: Health::full(NUTCRACKERS_SHOVEL_HP),
            stats: UnitStats {
                move_speed: I32F32::lit("0"),
                attack_range: I32F32::lit("0"),
                attack_damage: NUTCRACKERS_ATTACK_DAMAGE_MAX,
                attack_speed: I32F32::lit("1"),
                watch_range: I32F32::lit("0"),
            },
            state: NutcrackersState::Patrol,
            scan: NutcrackersScan::default(),
            shotgun: NutcrackersShotgun::default(),
            target: NutcrackersTarget::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnNutcrackersEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersStateChangedEvent {
    pub nutcrackers: Entity,
    pub from: NutcrackersState,
    pub to: NutcrackersState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersScanStartedEvent {
    pub nutcrackers: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersMovementDetectedEvent {
    pub nutcrackers: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersShotgunFiredEvent {
    pub nutcrackers: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersReloadStartedEvent {
    pub nutcrackers: Entity,
    pub reload_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersContactKickEvent {
    pub nutcrackers: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersDeathDropEvent {
    pub nutcrackers: Entity,
    pub guaranteed_shells: u8,
    pub remaining_loaded_shells: u8,
    pub dropped_shotgun_contains_remaining_shells: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersEnragedEvent {
    pub nutcrackers: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NutcrackersIgnoredDamageEvent {
    pub nutcrackers: Entity,
    pub source: Entity,
    pub state: NutcrackersState,
    pub eye_exposed: bool,
}

fn spawn_nutcrackers(
    mut commands: Commands,
    mut events: EventReader<SpawnNutcrackersEvent>,
    nutcrackers: Query<(), With<Nutcrackers>>,
) {
    let mut spawned_count = nutcrackers.iter().count();

    for event in events.read() {
        if spawned_count >= NUTCRACKERS_MAX_SPAWNED {
            break;
        }

        commands.spawn(NutcrackersBundle::new(*event));
        spawned_count += 1;
    }
}

fn nutcrackers_begin_scan(
    sim_hz: Res<SimHz>,
    mut scan_events: EventWriter<NutcrackersScanStartedEvent>,
    mut state_events: EventWriter<NutcrackersStateChangedEvent>,
    mut nutcrackers: Query<(Entity, &mut NutcrackersState, &mut NutcrackersScan), With<Nutcrackers>>,
) {
    for (entity, mut state, mut scan) in nutcrackers.iter_mut() {
        if *state != NutcrackersState::Patrol {
            continue;
        }

        scan.eye_exposed = true;
        scan.scan_timer_ticks = fixed_seconds_to_ticks(I32F32::lit("1"), sim_hz.0);
        set_nutcrackers_state(
            entity,
            &mut state,
            NutcrackersState::Scanning,
            &mut state_events,
        );
        scan_events.send(NutcrackersScanStartedEvent {
            nutcrackers: entity,
        });
    }
}

fn nutcrackers_rotate_scan(
    mut nutcrackers: Query<(&mut NutcrackersScan, &NutcrackersState), With<Nutcrackers>>,
) {
    for (mut scan, state) in nutcrackers.iter_mut() {
        if *state != NutcrackersState::Scanning || !scan.eye_exposed {
            continue;
        }

        scan.facing_degrees =
            wrap_degrees(scan.facing_degrees + NUTCRACKERS_SCAN_ROTATION_INCREMENT_DEGREES);

        if scan.scan_timer_ticks > 0 {
            scan.scan_timer_ticks -= 1;
        }
    }
}

fn nutcrackers_detect_movement(
    mut movement_events: EventWriter<NutcrackersMovementDetectedEvent>,
    mut state_events: EventWriter<NutcrackersStateChangedEvent>,
    mut nutcrackers: Query<
        (Entity, &mut NutcrackersState, &NutcrackersScan, &mut NutcrackersTarget),
        With<Nutcrackers>,
    >,
    targets: Query<(Entity, &NutcrackersTargetSensor)>,
) {
    for (nutcracker_entity, mut state, scan, mut target) in nutcrackers.iter_mut() {
        if *state != NutcrackersState::Scanning || !scan.eye_exposed {
            continue;
        }

        let Some((target_entity, sensor)) = first_detected_target(&targets) else {
            continue;
        };

        target.has_target = true;
        target.target = target_entity;
        target.target_stable_id = sensor.stable_id;
        movement_events.send(NutcrackersMovementDetectedEvent {
            nutcrackers: nutcracker_entity,
            target: target_entity,
            target_stable_id: sensor.stable_id,
        });
        set_nutcrackers_state(
            nutcracker_entity,
            &mut state,
            NutcrackersState::Chasing,
            &mut state_events,
        );
    }
}

fn nutcrackers_fire_shotgun(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut fired_events: EventWriter<NutcrackersShotgunFiredEvent>,
    mut reload_events: EventWriter<NutcrackersReloadStartedEvent>,
    mut enraged_events: EventWriter<NutcrackersEnragedEvent>,
    mut state_events: EventWriter<NutcrackersStateChangedEvent>,
    mut nutcrackers: Query<
        (
            Entity,
            &mut NutcrackersState,
            &mut NutcrackersScan,
            &mut NutcrackersShotgun,
            &NutcrackersTarget,
        ),
        With<Nutcrackers>,
    >,
) {
    for (entity, mut state, mut scan, mut shotgun, target) in nutcrackers.iter_mut() {
        if *state != NutcrackersState::Chasing && *state != NutcrackersState::Enraged {
            continue;
        }

        if shotgun.shoot_cooldown_ticks > 0 {
            shotgun.shoot_cooldown_ticks -= 1;
            continue;
        }

        if !target.has_target {
            continue;
        }

        if shotgun.shells_loaded == 0 {
            shotgun.reload_timer_ticks = fixed_seconds_to_ticks(I32F32::lit("1.5"), sim_hz.0);
            reload_events.send(NutcrackersReloadStartedEvent {
                nutcrackers: entity,
                reload_ticks: shotgun.reload_timer_ticks,
            });
            set_nutcrackers_state(
                entity,
                &mut state,
                NutcrackersState::Reloading,
                &mut state_events,
            );
            continue;
        }

        shotgun.shells_loaded -= 1;
        scan.eye_exposed = false;
        damage_events.send(IncomingDamageEvent {
            target: target.target,
            raw_amount: NUTCRACKERS_ATTACK_DAMAGE_MAX,
            damage_type: DamageType::Standard,
            source: entity,
        });
        fired_events.send(NutcrackersShotgunFiredEvent {
            nutcrackers: entity,
            target: target.target,
            target_stable_id: target.target_stable_id,
            damage: NUTCRACKERS_ATTACK_DAMAGE_MAX,
        });

        let next_cooldown = if *state == NutcrackersState::Enraged {
            fixed_ticks_scaled_to_hz(NUTCRACKERS_ENRAGED_SHOOT_TICKS_AT_20HZ, sim_hz.0)
        } else {
            fixed_ticks_scaled_to_hz(NUTCRACKERS_SHOOT_TICKS_AT_20HZ, sim_hz.0)
        };
        shotgun.shoot_cooldown_ticks = next_cooldown;

        if *state != NutcrackersState::Enraged {
            set_nutcrackers_state(
                entity,
                &mut state,
                NutcrackersState::Enraged,
                &mut state_events,
            );
            enraged_events.send(NutcrackersEnragedEvent {
                nutcrackers: entity,
            });
        }
    }
}

fn nutcrackers_reload(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<NutcrackersStateChangedEvent>,
    mut nutcrackers: Query<
        (Entity, &mut NutcrackersState, &mut NutcrackersShotgun),
        With<Nutcrackers>,
    >,
) {
    for (entity, mut state, mut shotgun) in nutcrackers.iter_mut() {
        if *state != NutcrackersState::Reloading {
            continue;
        }

        if shotgun.reload_timer_ticks > 0 {
            shotgun.reload_timer_ticks -= 1;
            continue;
        }

        shotgun.shells_loaded = NUTCRACKERS_SHOTGUN_CAPACITY;
        shotgun.killed_after_reloading = true;
        shotgun.shoot_cooldown_ticks =
            fixed_ticks_scaled_to_hz(NUTCRACKERS_RELOAD_TICKS_AT_20HZ, sim_hz.0);
        set_nutcrackers_state(
            entity,
            &mut state,
            NutcrackersState::Chasing,
            &mut state_events,
        );
    }
}

fn nutcrackers_contact_kick(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut kick_events: EventWriter<NutcrackersContactKickEvent>,
    nutcrackers: Query<Entity, With<Nutcrackers>>,
    targets: Query<(Entity, &NutcrackersTargetSensor)>,
) {
    for nutcracker_entity in nutcrackers.iter() {
        for (target_entity, sensor) in targets.iter() {
            if !sensor.too_close {
                continue;
            }

            kick_events.send(NutcrackersContactKickEvent {
                nutcrackers: nutcracker_entity,
                target: target_entity,
                target_stable_id: sensor.stable_id,
            });
            damage_events.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: NUTCRACKERS_INSTANT_KILL_DAMAGE,
                damage_type: DamageType::Standard,
                source: nutcracker_entity,
            });
        }
    }
}

fn nutcrackers_ignore_protected_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<NutcrackersIgnoredDamageEvent>,
    nutcrackers: Query<(&NutcrackersState, &NutcrackersScan), With<Nutcrackers>>,
) {
    for event in damage_events.read() {
        let Ok((state, scan)) = nutcrackers.get(event.target) else {
            continue;
        };

        if *state != NutcrackersState::Patrol && scan.eye_exposed {
            continue;
        }

        ignored_events.send(NutcrackersIgnoredDamageEvent {
            nutcrackers: event.target,
            source: event.source,
            state: *state,
            eye_exposed: scan.eye_exposed,
        });
    }
}

fn nutcrackers_death_drops(
    mut killed_events: EventReader<EntityKilledEvent>,
    mut drop_events: EventWriter<NutcrackersDeathDropEvent>,
    nutcrackers: Query<&NutcrackersShotgun, With<Nutcrackers>>,
) {
    for event in killed_events.read() {
        let Ok(shotgun) = nutcrackers.get(event.entity) else {
            continue;
        };

        drop_events.send(NutcrackersDeathDropEvent {
            nutcrackers: event.entity,
            guaranteed_shells: NUTCRACKERS_SHOTGUN_SHELLS_DROPPED,
            remaining_loaded_shells: shotgun.shells_loaded,
            dropped_shotgun_contains_remaining_shells: shotgun.killed_after_reloading,
        });
    }
}

fn nutcrackers_checksum(
    mut checksum: ResMut<SimChecksumState>,
    nutcrackers: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &NutcrackersState,
            &NutcrackersScan,
            &NutcrackersShotgun,
            &NutcrackersTarget,
        ),
        With<Nutcrackers>,
    >,
) {
    for (position, health, stats, state, scan, shotgun, target) in nutcrackers.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(nutcrackers_state_bits(*state));
        checksum.accumulate(scan.eye_exposed as u64);
        checksum.accumulate(scan.facing_degrees as u64);
        checksum.accumulate(scan.scan_timer_ticks as u64);
        checksum.accumulate(shotgun.shells_loaded as u64);
        checksum.accumulate(shotgun.shoot_cooldown_ticks as u64);
        checksum.accumulate(shotgun.reload_timer_ticks as u64);
        checksum.accumulate(shotgun.killed_after_reloading as u64);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
    }
}

fn first_detected_target(
    targets: &Query<(Entity, &NutcrackersTargetSensor)>,
) -> Option<(Entity, NutcrackersTargetSensor)> {
    let mut best: Option<(Entity, NutcrackersTargetSensor)> = None;

    for (entity, sensor) in targets.iter() {
        if !sensor.moved_this_tick || !sensor.in_scan_fov {
            continue;
        }

        if !NUTCRACKERS_CAN_SEE_THROUGH_FOG && sensor.visible_through_fog {
            continue;
        }

        if let Some((_best_entity, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *sensor));
    }

    best
}

fn set_nutcrackers_state(
    nutcrackers: Entity,
    state: &mut NutcrackersState,
    next: NutcrackersState,
    events: &mut EventWriter<NutcrackersStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(NutcrackersStateChangedEvent {
        nutcrackers,
        from: previous,
        to: next,
    });
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

fn fixed_ticks_scaled_to_hz(ticks_at_20hz: u32, sim_hz: I32F32) -> u32 {
    let scaled = I32F32::from_num(ticks_at_20hz) * sim_hz / I32F32::lit("20");
    let whole_ticks: u32 = scaled.to_num();

    if whole_ticks == 0 {
        1
    } else {
        whole_ticks
    }
}

fn wrap_degrees(degrees: i16) -> i16 {
    let mut wrapped = degrees % 360;

    if wrapped < 0 {
        wrapped += 360;
    }

    wrapped
}

fn nutcrackers_state_bits(state: NutcrackersState) -> u64 {
    match state {
        NutcrackersState::Patrol => 0,
        NutcrackersState::Scanning => 1,
        NutcrackersState::Chasing => 2,
        NutcrackersState::Reloading => 3,
        NutcrackersState::Enraged => 4,
    }
}