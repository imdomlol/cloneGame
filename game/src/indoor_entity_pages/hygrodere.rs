// Sources: vault/indoor_entity_pages/hygrodere.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimPosition, SimHz, UnitStats,
};

pub const HYGRODERE_ID: &str = "hygrodere";
pub const HYGRODERE_NAME: &str = "Hygrodere";
pub const HYGRODERE_TYPE: &str = "indoor_entity_pages";
pub const HYGRODERE_SUBTYPE: &str = "creature";
pub const HYGRODERE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Hygrodere";
pub const HYGRODERE_SOURCE_REVISION: u32 = 20577;
pub const HYGRODERE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const HYGRODERE_CONFIDENCE_BASIS_POINTS: u16 = 88;

pub const HYGRODERE_DWELLS: &str = "Inside";
pub const HYGRODERE_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const HYGRODERE_MAX_SPAWNED: usize = 2;
pub const HYGRODERE_CONTACT_DAMAGE: I32F32 = I32F32::lit("35");
pub const HYGRODERE_STUN_MULTIPLIER: I32F32 = I32F32::lit("4");
pub const HYGRODERE_RADAR_PIP_SIZE: &str = "Large";
pub const HYGRODERE_SHOVEL_HP: &str = "Immune";
pub const HYGRODERE_SHOCK_RESPONSE: &str = "Susceptible";
pub const HYGRODERE_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0.4");
pub const HYGRODERE_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.7");
pub const HYGRODERE_KILLED_BY_SPIKE_TRAP: bool = true;

pub const HYGRODERE_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");
pub const HYGRODERE_SLOW_SPEED: I32F32 = I32F32::lit("1");
pub const HYGRODERE_AGITATED_SPEED: I32F32 = I32F32::lit("2");
pub const HYGRODERE_HELD_BOOMBOX_SPEED: I32F32 = I32F32::lit("4");
pub const HYGRODERE_BASE_SIZE: I32F32 = I32F32::lit("1");
pub const HYGRODERE_AGITATED_SIZE: I32F32 = I32F32::lit("2");
pub const HYGRODERE_WATCH_RANGE: I32F32 = I32F32::lit("64");

pub const HYGRODERE_DEPENDS_ON: [&str; 2] = ["boombox", "spike_trap"];
pub const HYGRODERE_FRONTMATTER_BEHAVIOR: [&str; 1] = ["Chasing"];

pub const HYGRODERE_BEHAVIORAL_MECHANICS: [HygrodereBehaviorRule; 10] = [
    HygrodereBehaviorRule {
        condition: "players are present",
        outcome: "the Hygrodere follows the closest player slowly",
    },
    HygrodereBehaviorRule {
        condition: "no players are currently inside",
        outcome: "the Hygrodere moves to the last known player location",
    },
    HygrodereBehaviorRule {
        condition: "a player is trapped inside the blob",
        outcome: "it deals 35 contact damage",
    },
    HygrodereBehaviorRule {
        condition: "the Hygrodere takes damage",
        outcome: "it becomes agitated, grows in size, and moves faster",
    },
    HygrodereBehaviorRule {
        condition: "a boombox is playing on the ground",
        outcome: "the Hygrodere moves toward it and dances on top of it",
    },
    HygrodereBehaviorRule {
        condition: "a player is holding a boombox",
        outcome: "the Hygrodere chases that player and accelerates dramatically",
    },
    HygrodereBehaviorRule {
        condition: "its central hitbox touches a door interact collider",
        outcome: "it can open the door at an effective speed multiplier of 0.4",
    },
    HygrodereBehaviorRule {
        condition: "the Hygrodere is affected by stun",
        outcome: "the stun effect is multiplied by 4",
    },
    HygrodereBehaviorRule {
        condition: "a zap gun targets the Hygrodere",
        outcome: "the zap-gun difficulty modifier is 0.7",
    },
    HygrodereBehaviorRule {
        condition: "an elevated spike_trap is active or the trap base is beneath the Hygrodere",
        outcome: "the trap can kill it",
    },
];

pub struct HygroderePlugin;

impl Plugin for HygroderePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHygrodereEvent>()
            .add_event::<HygrodereStateChangedEvent>()
            .add_event::<HygrodereDoorAttemptEvent>()
            .add_event::<HygrodereDoorAttemptResolvedEvent>()
            .add_event::<HygrodereStunAppliedEvent>()
            .add_event::<HygrodereStunAdjustedEvent>()
            .add_event::<HygrodereZapGunTargetedEvent>()
            .add_event::<HygrodereZapGunDifficultyEvent>()
            .add_event::<HygrodereContactDamageEvent>()
            .add_event::<HygrodereAgitatedEvent>()
            .add_event::<HygrodereSpikeTrapKillEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_hygrodere,
                    hygrodere_chase_held_boombox,
                    hygrodere_move_to_ground_boombox,
                    hygrodere_follow_closest_player,
                    hygrodere_move_to_last_known_location,
                    hygrodere_contact_damage,
                    hygrodere_agitate_on_damage,
                    hygrodere_door_attempt_speed,
                    hygrodere_apply_stun_multiplier,
                    hygrodere_report_zap_gun_difficulty,
                    hygrodere_spike_trap_kill,
                    hygrodere_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hygrodere;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HygroderePlayerSensor {
    pub stable_id: u64,
    pub inside_blob: bool,
    pub holding_boombox: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HygrodereBoombox {
    pub stable_id: u64,
    pub playing_on_ground: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HygrodereSpikeTrapSensor {
    pub elevated_active: bool,
    pub base_beneath_hygrodere: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereBody {
    pub size: I32F32,
    pub agitated: bool,
}

impl Default for HygrodereBody {
    fn default() -> Self {
        Self {
            size: HYGRODERE_BASE_SIZE,
            agitated: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereMemory {
    pub has_last_known_player_location: bool,
    pub last_known_player_location: SimPosition,
    pub target_stable_id: u64,
}

impl Default for HygrodereMemory {
    fn default() -> Self {
        Self {
            has_last_known_player_location: false,
            last_known_player_location: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            target_stable_id: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HygrodereState {
    #[default]
    Chasing,
    MovingToLastKnownPlayerLocation,
    DancingOnBoombox,
    KilledBySpikeTrap,
}

#[derive(Bundle)]
pub struct HygrodereBundle {
    pub name: Name,
    pub hygrodere: Hygrodere,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: HygrodereState,
    pub body: HygrodereBody,
    pub memory: HygrodereMemory,
}

impl HygrodereBundle {
    pub fn new(event: SpawnHygrodereEvent) -> Self {
        Self {
            name: Name::new(HYGRODERE_NAME),
            hygrodere: Hygrodere,
            position: event.position,
            health: Health::full(HYGRODERE_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: HYGRODERE_SLOW_SPEED,
                attack_range: I32F32::lit("0"),
                attack_damage: HYGRODERE_CONTACT_DAMAGE,
                attack_speed: I32F32::lit("1"),
                watch_range: HYGRODERE_WATCH_RANGE,
            },
            state: HygrodereState::Chasing,
            body: HygrodereBody::default(),
            memory: HygrodereMemory {
                has_last_known_player_location: false,
                last_known_player_location: event.position,
                target_stable_id: 0,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnHygrodereEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereStateChangedEvent {
    pub hygrodere: Entity,
    pub from: HygrodereState,
    pub to: HygrodereState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereDoorAttemptEvent {
    pub hygrodere: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereDoorAttemptResolvedEvent {
    pub hygrodere: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereStunAppliedEvent {
    pub hygrodere: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereStunAdjustedEvent {
    pub hygrodere: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereZapGunTargetedEvent {
    pub hygrodere: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereZapGunDifficultyEvent {
    pub hygrodere: Entity,
    pub difficulty_modifier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereContactDamageEvent {
    pub hygrodere: Entity,
    pub player: Entity,
    pub player_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereAgitatedEvent {
    pub hygrodere: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HygrodereSpikeTrapKillEvent {
    pub hygrodere: Entity,
}

fn spawn_hygrodere(
    mut commands: Commands,
    mut events: EventReader<SpawnHygrodereEvent>,
    hygroderes: Query<(), With<Hygrodere>>,
) {
    let mut spawned_count = hygroderes.iter().count();

    for event in events.read() {
        if spawned_count >= HYGRODERE_MAX_SPAWNED {
            break;
        }

        commands.spawn(HygrodereBundle::new(*event));
        spawned_count += 1;
    }
}

fn hygrodere_chase_held_boombox(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HygrodereStateChangedEvent>,
    mut hygroderes: Query<
        (
            Entity,
            &mut SimPosition,
            &mut UnitStats,
            &mut HygrodereState,
            &mut HygrodereMemory,
        ),
        With<Hygrodere>,
    >,
    players: Query<(&SimPosition, &HygroderePlayerSensor), Without<Hygrodere>>,
) {
    let Some((target_position, target_sensor)) = closest_held_boombox_player(&players) else {
        return;
    };

    for (entity, mut position, mut stats, mut state, mut memory) in hygroderes.iter_mut() {
        stats.move_speed = HYGRODERE_HELD_BOOMBOX_SPEED;
        memory.has_last_known_player_location = true;
        memory.last_known_player_location = target_position;
        memory.target_stable_id = target_sensor.stable_id;
        set_hygrodere_state(entity, &mut state, HygrodereState::Chasing, &mut state_events);
        move_axis_toward(&mut position, target_position, stats.move_speed / sim_hz.0);
    }
}

fn hygrodere_move_to_ground_boombox(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HygrodereStateChangedEvent>,
    mut hygroderes: Query<
        (Entity, &mut SimPosition, &mut UnitStats, &mut HygrodereState),
        With<Hygrodere>,
    >,
    boomboxes: Query<(&SimPosition, &HygrodereBoombox), Without<Hygrodere>>,
    players: Query<&HygroderePlayerSensor, Without<Hygrodere>>,
) {
    if players.iter().any(|player| player.holding_boombox) {
        return;
    }

    let Some(target_position) = closest_ground_boombox(&boomboxes) else {
        return;
    };

    for (entity, mut position, mut stats, mut state) in hygroderes.iter_mut() {
        stats.move_speed = HYGRODERE_SLOW_SPEED;
        move_axis_toward(&mut position, target_position, stats.move_speed / sim_hz.0);

        if *position == target_position {
            set_hygrodere_state(
                entity,
                &mut state,
                HygrodereState::DancingOnBoombox,
                &mut state_events,
            );
        }
    }
}

fn hygrodere_follow_closest_player(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HygrodereStateChangedEvent>,
    mut hygroderes: Query<
        (
            Entity,
            &mut SimPosition,
            &mut UnitStats,
            &mut HygrodereState,
            &mut HygrodereMemory,
        ),
        With<Hygrodere>,
    >,
    players: Query<(&SimPosition, &HygroderePlayerSensor), Without<Hygrodere>>,
    boomboxes: Query<&HygrodereBoombox, Without<Hygrodere>>,
) {
    if players.iter().any(|(_position, player)| player.holding_boombox)
        || boomboxes.iter().any(|boombox| boombox.playing_on_ground)
    {
        return;
    }

    for (entity, mut position, mut stats, mut state, mut memory) in hygroderes.iter_mut() {
        let Some((target_position, target_sensor)) =
            closest_player_to_position(*position, &players)
        else {
            continue;
        };

        if *state == HygrodereState::KilledBySpikeTrap {
            continue;
        }

        stats.move_speed = if stats.move_speed > HYGRODERE_SLOW_SPEED {
            stats.move_speed
        } else {
            HYGRODERE_SLOW_SPEED
        };
        memory.has_last_known_player_location = true;
        memory.last_known_player_location = target_position;
        memory.target_stable_id = target_sensor.stable_id;
        set_hygrodere_state(entity, &mut state, HygrodereState::Chasing, &mut state_events);
        move_axis_toward(&mut position, target_position, stats.move_speed / sim_hz.0);
    }
}

fn hygrodere_move_to_last_known_location(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HygrodereStateChangedEvent>,
    mut hygroderes: Query<
        (
            Entity,
            &mut SimPosition,
            &UnitStats,
            &mut HygrodereState,
            &HygrodereMemory,
        ),
        With<Hygrodere>,
    >,
    players: Query<(), (With<HygroderePlayerSensor>, Without<Hygrodere>)>,
) {
    if !players.is_empty() {
        return;
    }

    for (entity, mut position, stats, mut state, memory) in hygroderes.iter_mut() {
        if !memory.has_last_known_player_location || *state == HygrodereState::KilledBySpikeTrap {
            continue;
        }

        set_hygrodere_state(
            entity,
            &mut state,
            HygrodereState::MovingToLastKnownPlayerLocation,
            &mut state_events,
        );
        move_axis_toward(
            &mut position,
            memory.last_known_player_location,
            stats.move_speed / sim_hz.0,
        );
    }
}

fn hygrodere_contact_damage(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<HygrodereContactDamageEvent>,
    hygroderes: Query<Entity, (With<Hygrodere>, Without<HygroderePlayerSensor>)>,
    players: Query<(Entity, &HygroderePlayerSensor), Without<Hygrodere>>,
) {
    for hygrodere in hygroderes.iter() {
        for (player_entity, player_sensor) in players.iter() {
            if !player_sensor.inside_blob {
                continue;
            }

            contact_events.send(HygrodereContactDamageEvent {
                hygrodere,
                player: player_entity,
                player_stable_id: player_sensor.stable_id,
                damage: HYGRODERE_CONTACT_DAMAGE,
            });
            damage_events.send(IncomingDamageEvent {
                target: player_entity,
                raw_amount: HYGRODERE_CONTACT_DAMAGE,
                damage_type: DamageType::Standard,
                source: hygrodere,
            });
        }
    }
}

fn hygrodere_agitate_on_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut agitated_events: EventWriter<HygrodereAgitatedEvent>,
    mut hygroderes: Query<(&mut UnitStats, &mut HygrodereBody), With<Hygrodere>>,
) {
    for event in damage_events.read() {
        let Ok((mut stats, mut body)) = hygroderes.get_mut(event.target) else {
            continue;
        };

        body.agitated = true;
        body.size = HYGRODERE_AGITATED_SIZE;
        stats.move_speed = HYGRODERE_AGITATED_SPEED;
        agitated_events.send(HygrodereAgitatedEvent {
            hygrodere: event.target,
        });
    }
}

fn hygrodere_door_attempt_speed(
    mut events: EventReader<HygrodereDoorAttemptEvent>,
    mut resolved_events: EventWriter<HygrodereDoorAttemptResolvedEvent>,
    hygroderes: Query<(), With<Hygrodere>>,
) {
    for event in events.read() {
        if hygroderes.get(event.hygrodere).is_err() {
            continue;
        }

        resolved_events.send(HygrodereDoorAttemptResolvedEvent {
            hygrodere: event.hygrodere,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, HYGRODERE_DOOR_OPEN_SPEED),
        });
    }
}

fn hygrodere_apply_stun_multiplier(
    mut events: EventReader<HygrodereStunAppliedEvent>,
    mut adjusted_events: EventWriter<HygrodereStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(HygrodereStunAdjustedEvent {
            hygrodere: event.hygrodere,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, HYGRODERE_STUN_MULTIPLIER),
        });
    }
}

fn hygrodere_report_zap_gun_difficulty(
    mut events: EventReader<HygrodereZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<HygrodereZapGunDifficultyEvent>,
    hygroderes: Query<(), With<Hygrodere>>,
) {
    for event in events.read() {
        if hygroderes.get(event.hygrodere).is_err() {
            continue;
        }

        difficulty_events.send(HygrodereZapGunDifficultyEvent {
            hygrodere: event.hygrodere,
            difficulty_modifier: HYGRODERE_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn hygrodere_spike_trap_kill(
    mut kill_events: EventWriter<HygrodereSpikeTrapKillEvent>,
    mut hygroderes: Query<(Entity, &mut HygrodereState, &HygrodereSpikeTrapSensor), With<Hygrodere>>,
) {
    if !HYGRODERE_KILLED_BY_SPIKE_TRAP {
        return;
    }

    for (entity, mut state, trap_sensor) in hygroderes.iter_mut() {
        if *state == HygrodereState::KilledBySpikeTrap {
            continue;
        }

        if trap_sensor.elevated_active || trap_sensor.base_beneath_hygrodere {
            *state = HygrodereState::KilledBySpikeTrap;
            kill_events.send(HygrodereSpikeTrapKillEvent { hygrodere: entity });
        }
    }
}

fn hygrodere_checksum(
    mut checksum: ResMut<SimChecksumState>,
    hygroderes: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &HygrodereState,
            &HygrodereBody,
            &HygrodereMemory,
        ),
        With<Hygrodere>,
    >,
) {
    for (position, health, stats, state, body, memory) in hygroderes.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(hygrodere_state_bits(*state));
        checksum.accumulate(body.size.to_bits() as u64);
        checksum.accumulate(body.agitated as u64);
        checksum.accumulate(memory.has_last_known_player_location as u64);
        checksum.accumulate(memory.last_known_player_location.x.to_bits() as u64);
        checksum.accumulate(memory.last_known_player_location.y.to_bits() as u64);
        checksum.accumulate(memory.target_stable_id);
    }
}

fn closest_held_boombox_player(
    players: &Query<(&SimPosition, &HygroderePlayerSensor), Without<Hygrodere>>,
) -> Option<(SimPosition, HygroderePlayerSensor)> {
    let mut best: Option<(SimPosition, HygroderePlayerSensor)> = None;

    for (position, sensor) in players.iter() {
        if !sensor.holding_boombox {
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

fn closest_ground_boombox(
    boomboxes: &Query<(&SimPosition, &HygrodereBoombox), Without<Hygrodere>>,
) -> Option<SimPosition> {
    let mut best: Option<(SimPosition, HygrodereBoombox)> = None;

    for (position, boombox) in boomboxes.iter() {
        if !boombox.playing_on_ground {
            continue;
        }

        if let Some((_best_position, best_boombox)) = best {
            if boombox.stable_id >= best_boombox.stable_id {
                continue;
            }
        }

        best = Some((*position, *boombox));
    }

    best.map(|(position, _boombox)| position)
}

fn closest_player_to_position(
    origin: SimPosition,
    players: &Query<(&SimPosition, &HygroderePlayerSensor), Without<Hygrodere>>,
) -> Option<(SimPosition, HygroderePlayerSensor)> {
    let mut best: Option<(SimPosition, HygroderePlayerSensor, I32F32)> = None;

    for (position, sensor) in players.iter() {
        let distance = fixed_abs(position.x - origin.x) + fixed_abs(position.y - origin.y);

        if let Some((_best_position, best_sensor, best_distance)) = best {
            if distance > best_distance {
                continue;
            }

            if distance == best_distance && sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((*position, *sensor, distance));
    }

    best.map(|(position, sensor, _distance)| (position, sensor))
}

fn set_hygrodere_state(
    hygrodere: Entity,
    state: &mut HygrodereState,
    next: HygrodereState,
    events: &mut EventWriter<HygrodereStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(HygrodereStateChangedEvent {
        hygrodere,
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

fn hygrodere_state_bits(state: HygrodereState) -> u64 {
    match state {
        HygrodereState::Chasing => 0,
        HygrodereState::MovingToLastKnownPlayerLocation => 1,
        HygrodereState::DancingOnBoombox => 2,
        HygrodereState::KilledBySpikeTrap => 3,
    }
}