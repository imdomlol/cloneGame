// Sources: vault/map_hazard_pages/spike_trap.md, vault/map_hazard_pages/map_hazards.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, SimChecksumState, SimPosition,
    SimTick,
};

pub const SPIKE_TRAP_ID: &str = "spike_trap";
pub const SPIKE_TRAP_NAME: &str = "Spike Trap";
pub const SPIKE_TRAP_TYPE: &str = "map_hazard_pages";
pub const SPIKE_TRAP_SUBTYPE: &str = "map_hazard";
pub const SPIKE_TRAP_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Spike_Trap";
pub const SPIKE_TRAP_SOURCE_REVISION: u32 = 20557;
pub const SPIKE_TRAP_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const SPIKE_TRAP_CONFIDENCE_BASIS_POINTS: u16 = 95;

pub const SPIKE_TRAP_DAMAGE_LABEL: &str = "Instant Kill";
pub const SPIKE_TRAP_DAMAGE_TYPE_LABEL: &str = "Collision";
pub const SPIKE_TRAP_TIMER_MODE_CHANCE_PERCENT: u8 = 80;
pub const SPIKE_TRAP_DETECTION_MODE_CHANCE_PERCENT: u8 = 20;
pub const SPIKE_TRAP_INTERVAL_BRANCH_STANDARD_PERCENT: u8 = 81;
pub const SPIKE_TRAP_INTERVAL_BRANCH_LONG_PERCENT: u8 = 10;
pub const SPIKE_TRAP_INTERVAL_BRANCH_FAST_PERCENT: u8 = 9;
pub const SPIKE_TRAP_NEAR_PLAYER_RADIUS: I32F32 = I32F32::lit("8");
pub const SPIKE_TRAP_PLATE_KILL_RADIUS: I32F32 = I32F32::lit("8");
pub const SPIKE_TRAP_SENSOR_LINE_LENGTH: I32F32 = I32F32::lit("4.4");
pub const SPIKE_TRAP_RISING_COLLISION_DISABLED_SECONDS: I32F32 = I32F32::lit("0.75");
pub const SPIKE_TRAP_INTERVAL_MIN_SECONDS: I32F32 = I32F32::lit("0.8");
pub const SPIKE_TRAP_INTERVAL_STANDARD_MAX_SECONDS: I32F32 = I32F32::lit("10.8");
pub const SPIKE_TRAP_INTERVAL_LONG_MAX_SECONDS: I32F32 = I32F32::lit("26.2");
pub const SPIKE_TRAP_INTERVAL_FAST_MAX_SECONDS: I32F32 = I32F32::lit("2.1");
pub const SPIKE_TRAP_GLOBAL_COUNT_MIN: u16 = 0;
pub const SPIKE_TRAP_GLOBAL_COUNT_MAX: u16 = 18;
pub const SPIKE_TRAP_DEFAULT_SIM_HZ: u32 = 20;

pub const SPIKE_TRAP_DEPENDS_ON: [&str; 6] = [
    "map_hazard",
    "employee",
    "entity",
    "flashlight",
    "hoarding_bug",
    "thumper",
];

pub const SPIKE_TRAP_BEHAVIORAL_MECHANICS: [SpikeTrapBehaviorRule; 10] = [
    SpikeTrapBehaviorRule::new(
        "the trap spawns",
        "it resolves its mode and slam interval from the map seed and spawn position",
    ),
    SpikeTrapBehaviorRule::new(
        "interval mode is selected",
        "there is an 80% chance that the trap slams periodically on its own",
    ),
    SpikeTrapBehaviorRule::new(
        "interval mode selects the 81% branch",
        "the slam interval is random between 0.8 s and 10.8 s",
    ),
    SpikeTrapBehaviorRule::new(
        "interval mode selects the 10% branch",
        "the slam interval is random between 0.8 s and 26.2 s",
    ),
    SpikeTrapBehaviorRule::new(
        "interval mode selects the 9% branch",
        "the slam interval is random between 0.8 s and 2.1 s",
    ),
    SpikeTrapBehaviorRule::new(
        "interval mode is active and no player is within 8 units",
        "entities within 8 units of the plate do not trigger a slam",
    ),
    SpikeTrapBehaviorRule::new(
        "detection mode is selected",
        "there is a 20% chance that the trap only slams when a player crosses a 4.4-unit sensor line in front of the support beam",
    ),
    SpikeTrapBehaviorRule::new(
        "a player crosses the detection line while killable targets are underneath",
        "those targets can be killed",
    ),
    SpikeTrapBehaviorRule::new(
        "the trap has just started rising",
        "collision detection remains disabled for 0.75 s",
    ),
    SpikeTrapBehaviorRule::new(
        "the trap slams into an employee or any other killable entity",
        "the target dies instantly",
    ),
];

const SPIKE_TRAP_SPAWN_MODE_SALT: u64 = 0x7370_696b_655f_0001;
const SPIKE_TRAP_INTERVAL_BRANCH_SALT: u64 = 0x7370_696b_655f_0002;
const SPIKE_TRAP_INTERVAL_VALUE_SALT: u64 = 0x7370_696b_655f_0003;

pub struct SpikeTrapPlugin;

impl Plugin for SpikeTrapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSpikeTrapEvent>()
            .add_event::<SpikeTrapSpawnCountResolvedEvent>()
            .add_event::<SpikeTrapSensorCrossedEvent>()
            .add_event::<SpikeTrapSlamRequestedEvent>()
            .add_event::<SpikeTrapSlamEvent>()
            .add_event::<SpikeTrapRiseStartedEvent>()
            .add_event::<SpikeTrapInstantKillEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_spike_trap,
                    resolve_spike_trap_spawn_count,
                    spike_trap_sensor_trigger,
                    spike_trap_interval_timer,
                    spike_trap_apply_slam_requests,
                    spike_trap_apply_slam_damage,
                    spike_trap_rising_collision_lockout,
                    spike_trap_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpikeTrapBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

impl SpikeTrapBehaviorRule {
    pub const fn new(condition: &'static str, outcome: &'static str) -> Self {
        Self { condition, outcome }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpikeTrap {
    pub stable_id: u64,
    pub mode: SpikeTrapMode,
    pub interval_branch: SpikeTrapIntervalBranch,
    pub slam_interval_ticks: u32,
    pub ticks_until_slam: u32,
    pub rising_collision_disabled_ticks_remaining: u32,
    pub slamming: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpikeTrapTarget {
    pub stable_id: u64,
    pub is_employee: bool,
    pub is_killable_entity: bool,
    pub is_player: bool,
}

#[derive(Bundle, Clone, Copy, Debug)]
pub struct SpikeTrapBundle {
    pub spike_trap: SpikeTrap,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnSpikeTrapEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeTrapSpawnCountResolvedEvent {
    pub min_spike_traps: u16,
    pub max_spike_traps: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeTrapSensorCrossedEvent {
    pub trap: Entity,
    pub player_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeTrapSlamRequestedEvent {
    pub trap: Entity,
    pub trigger: SpikeTrapTrigger,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeTrapSlamEvent {
    pub trap: Entity,
    pub trap_stable_id: u64,
    pub position: SimPosition,
    pub trigger: SpikeTrapTrigger,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeTrapRiseStartedEvent {
    pub trap: Entity,
    pub trap_stable_id: u64,
    pub collision_disabled_ticks: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeTrapInstantKillEvent {
    pub trap: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpikeTrapMode {
    #[default]
    Interval,
    Detection,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpikeTrapIntervalBranch {
    #[default]
    Standard81,
    Long10,
    Fast9,
    DetectionOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpikeTrapTrigger {
    IntervalTimer,
    SensorLine,
}

fn spawn_spike_trap(
    mut commands: Commands,
    mut events: EventReader<SpawnSpikeTrapEvent>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for event in events.read() {
        let position_salt = position_salt(event.position) ^ event.stable_id;
        let mode_roll =
            tick_rng(seed.0, tick.0, SPIKE_TRAP_SPAWN_MODE_SALT ^ position_salt).next_u32() % 100;

        let (mode, branch, interval_ticks) = if mode_roll < SPIKE_TRAP_TIMER_MODE_CHANCE_PERCENT as u32
        {
            let branch_roll = tick_rng(
                seed.0,
                tick.0,
                SPIKE_TRAP_INTERVAL_BRANCH_SALT ^ position_salt,
            )
            .next_u32()
                % 100;
            let branch = if branch_roll < SPIKE_TRAP_INTERVAL_BRANCH_STANDARD_PERCENT as u32 {
                SpikeTrapIntervalBranch::Standard81
            } else if branch_roll
                < (SPIKE_TRAP_INTERVAL_BRANCH_STANDARD_PERCENT
                    + SPIKE_TRAP_INTERVAL_BRANCH_LONG_PERCENT) as u32
            {
                SpikeTrapIntervalBranch::Long10
            } else {
                SpikeTrapIntervalBranch::Fast9
            };

            (
                SpikeTrapMode::Interval,
                branch,
                resolve_interval_ticks(seed.0, tick.0, position_salt, branch),
            )
        } else {
            (
                SpikeTrapMode::Detection,
                SpikeTrapIntervalBranch::DetectionOnly,
                0,
            )
        };

        commands.spawn(SpikeTrapBundle {
            spike_trap: SpikeTrap {
                stable_id: event.stable_id,
                mode,
                interval_branch: branch,
                slam_interval_ticks: interval_ticks,
                ticks_until_slam: interval_ticks,
                rising_collision_disabled_ticks_remaining: 0,
                slamming: false,
            },
            position: event.position,
        });
    }
}

fn resolve_spike_trap_spawn_count(mut events: EventWriter<SpikeTrapSpawnCountResolvedEvent>) {
    events.send(SpikeTrapSpawnCountResolvedEvent {
        min_spike_traps: SPIKE_TRAP_GLOBAL_COUNT_MIN,
        max_spike_traps: SPIKE_TRAP_GLOBAL_COUNT_MAX,
    });
}

fn spike_trap_sensor_trigger(
    mut events: EventReader<SpikeTrapSensorCrossedEvent>,
    traps: Query<&SpikeTrap>,
    mut requests: EventWriter<SpikeTrapSlamRequestedEvent>,
) {
    for event in events.read() {
        if let Ok(trap) = traps.get(event.trap) {
            if trap.mode == SpikeTrapMode::Detection {
                requests.send(SpikeTrapSlamRequestedEvent {
                    trap: event.trap,
                    trigger: SpikeTrapTrigger::SensorLine,
                });
            }
        }
    }
}

fn spike_trap_interval_timer(
    mut traps: Query<(Entity, &mut SpikeTrap, &SimPosition)>,
    targets: Query<(&SpikeTrapTarget, &SimPosition)>,
    mut requests: EventWriter<SpikeTrapSlamRequestedEvent>,
) {
    let mut sorted_targets: Vec<(u64, SpikeTrapTarget, SimPosition)> = targets
        .iter()
        .map(|(target, position)| (target.stable_id, *target, *position))
        .collect();
    sorted_targets.sort_by_key(|(stable_id, _, _)| *stable_id);

    for (entity, mut trap, trap_position) in &mut traps {
        if trap.mode != SpikeTrapMode::Interval || trap.slam_interval_ticks == 0 {
            continue;
        }

        if trap.ticks_until_slam > 0 {
            trap.ticks_until_slam -= 1;
        }

        if trap.ticks_until_slam == 0 {
            let player_nearby = sorted_targets.iter().any(|(_, target, target_position)| {
                target.is_player
                    && distance_squared(*trap_position, *target_position)
                        <= SPIKE_TRAP_NEAR_PLAYER_RADIUS * SPIKE_TRAP_NEAR_PLAYER_RADIUS
            });

            trap.ticks_until_slam = trap.slam_interval_ticks;

            if player_nearby {
                requests.send(SpikeTrapSlamRequestedEvent {
                    trap: entity,
                    trigger: SpikeTrapTrigger::IntervalTimer,
                });
            }
        }
    }
}

fn spike_trap_apply_slam_requests(
    mut requests: EventReader<SpikeTrapSlamRequestedEvent>,
    mut traps: Query<(&mut SpikeTrap, &SimPosition)>,
    mut slams: EventWriter<SpikeTrapSlamEvent>,
    mut rising: EventWriter<SpikeTrapRiseStartedEvent>,
) {
    for request in requests.read() {
        if let Ok((mut trap, position)) = traps.get_mut(request.trap) {
            if trap.rising_collision_disabled_ticks_remaining > 0 {
                continue;
            }

            trap.slamming = true;
            trap.rising_collision_disabled_ticks_remaining =
                seconds_to_ticks(SPIKE_TRAP_RISING_COLLISION_DISABLED_SECONDS);

            slams.send(SpikeTrapSlamEvent {
                trap: request.trap,
                trap_stable_id: trap.stable_id,
                position: *position,
                trigger: request.trigger,
            });

            rising.send(SpikeTrapRiseStartedEvent {
                trap: request.trap,
                trap_stable_id: trap.stable_id,
                collision_disabled_ticks: trap.rising_collision_disabled_ticks_remaining,
            });
        }
    }
}

fn spike_trap_apply_slam_damage(
    mut slams: EventReader<SpikeTrapSlamEvent>,
    targets: Query<(Entity, &SpikeTrapTarget, &SimPosition, &Health)>,
    mut damage: EventWriter<IncomingDamageEvent>,
    mut kills: EventWriter<SpikeTrapInstantKillEvent>,
) {
    let mut sorted_targets: Vec<(u64, Entity, SpikeTrapTarget, SimPosition, Health)> = targets
        .iter()
        .map(|(entity, target, position, health)| {
            (target.stable_id, entity, *target, *position, *health)
        })
        .collect();
    sorted_targets.sort_by_key(|(stable_id, _, _, _, _)| *stable_id);

    for slam in slams.read() {
        for (_, entity, target, position, health) in &sorted_targets {
            if !target.is_employee && !target.is_killable_entity {
                continue;
            }

            if distance_squared(slam.position, *position)
                <= SPIKE_TRAP_PLATE_KILL_RADIUS * SPIKE_TRAP_PLATE_KILL_RADIUS
            {
                damage.send(IncomingDamageEvent {
                    target: *entity,
                    raw_amount: health.max,
                    damage_type: DamageType::Standard,
                    source: slam.trap,
                });

                kills.send(SpikeTrapInstantKillEvent {
                    trap: slam.trap,
                    target: *entity,
                    target_stable_id: target.stable_id,
                });
            }
        }
    }
}

fn spike_trap_rising_collision_lockout(mut traps: Query<&mut SpikeTrap>) {
    for mut trap in &mut traps {
        if trap.rising_collision_disabled_ticks_remaining > 0 {
            trap.rising_collision_disabled_ticks_remaining -= 1;
            if trap.rising_collision_disabled_ticks_remaining == 0 {
                trap.slamming = false;
            }
        }
    }
}

fn spike_trap_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    traps: Query<(&SpikeTrap, &SimPosition)>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(SPIKE_TRAP_SOURCE_REVISION as u64);
    checksum.accumulate(SPIKE_TRAP_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(SPIKE_TRAP_TIMER_MODE_CHANCE_PERCENT as u64);
    checksum.accumulate(SPIKE_TRAP_DETECTION_MODE_CHANCE_PERCENT as u64);
    checksum.accumulate(SPIKE_TRAP_INTERVAL_BRANCH_STANDARD_PERCENT as u64);
    checksum.accumulate(SPIKE_TRAP_INTERVAL_BRANCH_LONG_PERCENT as u64);
    checksum.accumulate(SPIKE_TRAP_INTERVAL_BRANCH_FAST_PERCENT as u64);
    checksum.accumulate(SPIKE_TRAP_NEAR_PLAYER_RADIUS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_PLATE_KILL_RADIUS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_SENSOR_LINE_LENGTH.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_RISING_COLLISION_DISABLED_SECONDS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_INTERVAL_MIN_SECONDS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_INTERVAL_STANDARD_MAX_SECONDS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_INTERVAL_LONG_MAX_SECONDS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_INTERVAL_FAST_MAX_SECONDS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_GLOBAL_COUNT_MIN as u64);
    checksum.accumulate(SPIKE_TRAP_GLOBAL_COUNT_MAX as u64);

    for dependency in SPIKE_TRAP_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in SPIKE_TRAP_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for (trap, position) in &traps {
        checksum.accumulate(trap.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(mode_bits(trap.mode));
        checksum.accumulate(interval_branch_bits(trap.interval_branch));
        checksum.accumulate(trap.slam_interval_ticks as u64);
        checksum.accumulate(trap.ticks_until_slam as u64);
        checksum.accumulate(trap.rising_collision_disabled_ticks_remaining as u64);
        checksum.accumulate(trap.slamming as u64);
    }
}

fn resolve_interval_ticks(
    game_seed: u64,
    tick: u64,
    position_salt_value: u64,
    branch: SpikeTrapIntervalBranch,
) -> u32 {
    let max_seconds = match branch {
        SpikeTrapIntervalBranch::Standard81 => SPIKE_TRAP_INTERVAL_STANDARD_MAX_SECONDS,
        SpikeTrapIntervalBranch::Long10 => SPIKE_TRAP_INTERVAL_LONG_MAX_SECONDS,
        SpikeTrapIntervalBranch::Fast9 => SPIKE_TRAP_INTERVAL_FAST_MAX_SECONDS,
        SpikeTrapIntervalBranch::DetectionOnly => SPIKE_TRAP_INTERVAL_MIN_SECONDS,
    };

    let min_tenths = fixed_seconds_to_tenths(SPIKE_TRAP_INTERVAL_MIN_SECONDS);
    let max_tenths = fixed_seconds_to_tenths(max_seconds);
    let range = max_tenths - min_tenths + 1;
    let roll =
        tick_rng(game_seed, tick, SPIKE_TRAP_INTERVAL_VALUE_SALT ^ position_salt_value).next_u32()
            % range;

    tenths_to_ticks(min_tenths + roll)
}

fn fixed_seconds_to_tenths(value: I32F32) -> u32 {
    (value * I32F32::lit("10")).to_num::<u32>()
}

fn tenths_to_ticks(tenths: u32) -> u32 {
    let ticks = tenths * SPIKE_TRAP_DEFAULT_SIM_HZ;
    (ticks + 9) / 10
}

fn seconds_to_ticks(seconds: I32F32) -> u32 {
    let tenths = fixed_seconds_to_tenths(seconds);
    tenths_to_ticks(tenths)
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn position_salt(position: SimPosition) -> u64 {
    (position.x.to_bits() as u64).rotate_left(17) ^ (position.y.to_bits() as u64).rotate_left(31)
}

fn mode_bits(mode: SpikeTrapMode) -> u64 {
    match mode {
        SpikeTrapMode::Interval => 1,
        SpikeTrapMode::Detection => 2,
    }
}

fn interval_branch_bits(branch: SpikeTrapIntervalBranch) -> u64 {
    match branch {
        SpikeTrapIntervalBranch::Standard81 => 1,
        SpikeTrapIntervalBranch::Long10 => 2,
        SpikeTrapIntervalBranch::Fast9 => 3,
        SpikeTrapIntervalBranch::DetectionOnly => 4,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}