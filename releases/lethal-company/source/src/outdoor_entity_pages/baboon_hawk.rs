// Sources: vault/wiki_homepage_pages/lethal_company_wiki.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, NoiseEmittedEvent,
    SimChecksumState, SimHz, SimPosition, SimTick, UnitStats,
};

pub const BABOON_HAWK_ID: &str = "baboon_hawk";
pub const BABOON_HAWK_NAME: &str = "Baboon Hawk";
pub const BABOON_HAWK_TYPE: &str = "outdoor_entity_pages";
pub const BABOON_HAWK_SUBTYPE: &str = "outdoor_entity";
pub const BABOON_HAWK_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Baboon_Hawk";
pub const BABOON_HAWK_SOURCE_REVISION: u32 = 17668;
pub const BABOON_HAWK_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const BABOON_HAWK_CONFIDENCE_BASIS_POINTS: u16 = 99;

pub const BABOON_HAWK_HP: I32F32 = I32F32::lit("6");
pub const BABOON_HAWK_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const BABOON_HAWK_MAX_SPAWNED: usize = 15;
pub const BABOON_HAWK_MOVE_SPEED: I32F32 = I32F32::lit("4");
pub const BABOON_HAWK_CHASE_SPEED: I32F32 = I32F32::lit("8");
pub const BABOON_HAWK_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const BABOON_HAWK_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
pub const BABOON_HAWK_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("1");
pub const BABOON_HAWK_WATCH_RANGE: I32F32 = I32F32::lit("35");
pub const BABOON_HAWK_THREAT_RANGE: I32F32 = I32F32::lit("8");
pub const BABOON_HAWK_ITEM_CLAIM_RANGE: I32F32 = I32F32::lit("2");
pub const BABOON_HAWK_NOISE_HEAR_RANGE: I32F32 = I32F32::lit("22");
pub const BABOON_HAWK_INTIMIDATION_SECONDS: I32F32 = I32F32::lit("3");
pub const BABOON_HAWK_FLEE_SECONDS: I32F32 = I32F32::lit("2");
pub const BABOON_HAWK_STUN_SECONDS: I32F32 = I32F32::lit("4");
pub const BABOON_HAWK_INTIMIDATION_SALT: u64 = 0xBA_B0_0A_00_0000_0001;
pub const BABOON_HAWK_RETARGET_SALT: u64 = 0xBA_B0_0A_00_0000_0002;

pub const BABOON_HAWK_DEPENDS_ON: [&str; 1] = ["entity"];

pub const BABOON_HAWK_BEHAVIORAL_MECHANICS: [BaboonHawkBehaviorRule; 6] = [
    BaboonHawkBehaviorRule {
        condition: "it sees an employee within watch range",
        outcome: "it approaches and begins intimidation",
    },
    BaboonHawkBehaviorRule {
        condition: "the employee stays within threat range during intimidation",
        outcome: "it may switch to chase",
    },
    BaboonHawkBehaviorRule {
        condition: "it reaches attack range while chasing",
        outcome: "it deals damage to the target employee",
    },
    BaboonHawkBehaviorRule {
        condition: "a loud noise occurs nearby",
        outcome: "it investigates the noise source",
    },
    BaboonHawkBehaviorRule {
        condition: "it is stunned",
        outcome: "it cannot move until the stun timer ends",
    },
    BaboonHawkBehaviorRule {
        condition: "it takes lethal damage",
        outcome: "it is defeated",
    },
];

pub struct BaboonHawkPlugin;

impl Plugin for BaboonHawkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBaboonHawkEvent>()
            .add_event::<BaboonHawkStateChangedEvent>()
            .add_event::<BaboonHawkIntimidationEvent>()
            .add_event::<BaboonHawkNoiseInvestigatedEvent>()
            .add_event::<BaboonHawkItemClaimedEvent>()
            .add_event::<BaboonHawkStunAppliedEvent>()
            .add_event::<BaboonHawkDamageTakenEvent>()
            .add_event::<BaboonHawkDefeatedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    baboon_hawk_spawn_from_events,
                    baboon_hawk_acquire_target,
                    baboon_hawk_intimidate_target,
                    baboon_hawk_chase_target,
                    baboon_hawk_investigate_noise,
                    baboon_hawk_claim_items,
                    baboon_hawk_apply_stun,
                    baboon_hawk_take_damage,
                    baboon_hawk_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaboonHawk {
    pub stable_id: u64,
    pub pack_id: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaboonHawkEmployeeSensor {
    pub stable_id: u64,
    pub is_alive: bool,
    pub visible_to_hawk: bool,
    pub holding_item: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaboonHawkItemSensor {
    pub stable_id: u64,
    pub claimed: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkTarget {
    pub has_target: bool,
    pub target_entity: Entity,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
}

impl Default for BaboonHawkTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_entity: Entity::PLACEHOLDER,
            target_stable_id: 0,
            last_known_position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkMovement {
    pub current_speed: I32F32,
    pub destination: SimPosition,
    pub intimidation_ticks: u32,
    pub flee_ticks: u32,
    pub stun_ticks: u32,
}

impl Default for BaboonHawkMovement {
    fn default() -> Self {
        Self {
            current_speed: BABOON_HAWK_MOVE_SPEED,
            destination: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            intimidation_ticks: 0,
            flee_ticks: 0,
            stun_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BaboonHawkState {
    #[default]
    Roam,
    Intimidate,
    Chase,
    Investigate,
    Flee,
    Stunned,
    Dead,
}

#[derive(Bundle)]
pub struct BaboonHawkBundle {
    pub name: Name,
    pub hawk: BaboonHawk,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: BaboonHawkState,
    pub target: BaboonHawkTarget,
    pub movement: BaboonHawkMovement,
}

impl BaboonHawkBundle {
    pub fn new(event: SpawnBaboonHawkEvent) -> Self {
        Self {
            name: Name::new(BABOON_HAWK_NAME),
            hawk: BaboonHawk {
                stable_id: event.stable_id,
                pack_id: event.pack_id,
            },
            position: event.position,
            health: Health::full(BABOON_HAWK_HP),
            stats: UnitStats {
                move_speed: BABOON_HAWK_MOVE_SPEED,
                attack_range: BABOON_HAWK_ATTACK_RANGE,
                attack_damage: BABOON_HAWK_ATTACK_DAMAGE,
                attack_speed: BABOON_HAWK_ATTACK_SPEED_SECONDS,
                watch_range: BABOON_HAWK_WATCH_RANGE,
            },
            state: BaboonHawkState::Roam,
            target: BaboonHawkTarget {
                last_known_position: event.position,
                ..Default::default()
            },
            movement: BaboonHawkMovement {
                destination: event.position,
                ..Default::default()
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnBaboonHawkEvent {
    pub stable_id: u64,
    pub pack_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkStateChangedEvent {
    pub hawk: Entity,
    pub from: BaboonHawkState,
    pub to: BaboonHawkState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkIntimidationEvent {
    pub hawk: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub duration_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkNoiseInvestigatedEvent {
    pub hawk: Entity,
    pub noise_source: Entity,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkItemClaimedEvent {
    pub hawk: Entity,
    pub item: Entity,
    pub item_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkStunAppliedEvent {
    pub hawk: Entity,
    pub source: Entity,
    pub ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkDamageTakenEvent {
    pub hawk: Entity,
    pub source: Entity,
    pub damage: I32F32,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaboonHawkDefeatedEvent {
    pub hawk: Entity,
    pub source: Entity,
    pub stable_id: u64,
}

fn baboon_hawk_spawn_from_events(
    mut commands: Commands,
    mut events: EventReader<SpawnBaboonHawkEvent>,
    hawks: Query<(), With<BaboonHawk>>,
) {
    let mut spawned_count = hawks.iter().count();

    for event in events.read() {
        if spawned_count >= BABOON_HAWK_MAX_SPAWNED {
            break;
        }

        commands.spawn(BaboonHawkBundle::new(*event));
        spawned_count += 1;
    }
}

fn baboon_hawk_acquire_target(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<BaboonHawkStateChangedEvent>,
    mut intimidation_events: EventWriter<BaboonHawkIntimidationEvent>,
    mut hawks: Query<
        (
            Entity,
            &SimPosition,
            &mut BaboonHawkState,
            &mut BaboonHawkTarget,
            &mut BaboonHawkMovement,
            &mut UnitStats,
        ),
        With<BaboonHawk>,
    >,
    employees: Query<(Entity, &SimPosition, &BaboonHawkEmployeeSensor), Without<BaboonHawk>>,
) {
    for (hawk_entity, hawk_position, mut state, mut target, mut movement, mut stats) in
        hawks.iter_mut()
    {
        if *state != BaboonHawkState::Roam {
            continue;
        }

        let Some((employee_entity, employee_position, sensor)) =
            nearest_visible_employee(*hawk_position, BABOON_HAWK_WATCH_RANGE, &employees)
        else {
            stats.move_speed = BABOON_HAWK_MOVE_SPEED;
            continue;
        };

        target.has_target = true;
        target.target_entity = employee_entity;
        target.target_stable_id = sensor.stable_id;
        target.last_known_position = employee_position;
        movement.destination = employee_position;
        movement.intimidation_ticks =
            fixed_seconds_to_ticks(BABOON_HAWK_INTIMIDATION_SECONDS, sim_hz.0);
        stats.move_speed = BABOON_HAWK_MOVE_SPEED;

        set_baboon_hawk_state(
            hawk_entity,
            &mut state,
            BaboonHawkState::Intimidate,
            &mut state_events,
        );

        intimidation_events.send(BaboonHawkIntimidationEvent {
            hawk: hawk_entity,
            target: employee_entity,
            target_stable_id: sensor.stable_id,
            duration_ticks: movement.intimidation_ticks,
        });
    }
}

fn baboon_hawk_intimidate_target(
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<BaboonHawkStateChangedEvent>,
    mut hawks: Query<
        (
            Entity,
            &BaboonHawk,
            &mut SimPosition,
            &mut BaboonHawkState,
            &mut BaboonHawkTarget,
            &mut BaboonHawkMovement,
            &mut UnitStats,
        ),
        With<BaboonHawk>,
    >,
    employees: Query<(Entity, &SimPosition, &BaboonHawkEmployeeSensor), Without<BaboonHawk>>,
) {
    for (hawk_entity, hawk, mut position, mut state, mut target, mut movement, mut stats) in
        hawks.iter_mut()
    {
        if *state != BaboonHawkState::Intimidate || !target.has_target {
            continue;
        }

        let Some((employee_entity, employee_position, sensor)) =
            employee_by_stable_id(target.target_stable_id, &employees)
        else {
            clear_baboon_hawk_target(&mut target);
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Roam,
                &mut state_events,
            );
            continue;
        };

        target.target_entity = employee_entity;
        target.last_known_position = employee_position;
        movement.destination = employee_position;
        move_axis_toward(&mut position, employee_position, stats.move_speed / sim_hz.0);

        if fixed_distance_sq(*position, employee_position) > fixed_square(BABOON_HAWK_THREAT_RANGE) {
            continue;
        }

        if movement.intimidation_ticks > 0 {
            movement.intimidation_ticks -= 1;
            continue;
        }

        let mut rng = tick_rng(
            game_seed.0,
            tick.0,
            BABOON_HAWK_INTIMIDATION_SALT ^ hawk.stable_id ^ sensor.stable_id,
        );
        let aggression_roll = rng.next_u32() % 100;
        if aggression_roll < 50 || sensor.holding_item {
            stats.move_speed = BABOON_HAWK_CHASE_SPEED;
            movement.current_speed = BABOON_HAWK_CHASE_SPEED;
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Chase,
                &mut state_events,
            );
        } else {
            movement.flee_ticks = fixed_seconds_to_ticks(BABOON_HAWK_FLEE_SECONDS, sim_hz.0);
            stats.move_speed = BABOON_HAWK_CHASE_SPEED;
            movement.current_speed = BABOON_HAWK_CHASE_SPEED;
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Flee,
                &mut state_events,
            );
        }
    }
}

fn baboon_hawk_chase_target(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut state_events: EventWriter<BaboonHawkStateChangedEvent>,
    sim_hz: Res<SimHz>,
    mut hawks: Query<
        (
            Entity,
            &mut SimPosition,
            &mut BaboonHawkState,
            &mut BaboonHawkTarget,
            &mut BaboonHawkMovement,
            &mut UnitStats,
        ),
        With<BaboonHawk>,
    >,
    employees: Query<(Entity, &SimPosition, &BaboonHawkEmployeeSensor), Without<BaboonHawk>>,
) {
    for (hawk_entity, mut position, mut state, mut target, mut movement, mut stats) in
        hawks.iter_mut()
    {
        if *state == BaboonHawkState::Flee {
            if movement.flee_ticks > 0 {
                movement.flee_ticks -= 1;
                let away = SimPosition {
                    x: position.x + (position.x - target.last_known_position.x),
                    y: position.y + (position.y - target.last_known_position.y),
                };
                move_axis_toward(&mut position, away, stats.move_speed / sim_hz.0);
                continue;
            }

            clear_baboon_hawk_target(&mut target);
            stats.move_speed = BABOON_HAWK_MOVE_SPEED;
            movement.current_speed = BABOON_HAWK_MOVE_SPEED;
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Roam,
                &mut state_events,
            );
            continue;
        }

        if *state != BaboonHawkState::Chase || !target.has_target {
            continue;
        }

        let Some((employee_entity, employee_position, sensor)) =
            employee_by_stable_id(target.target_stable_id, &employees)
        else {
            clear_baboon_hawk_target(&mut target);
            stats.move_speed = BABOON_HAWK_MOVE_SPEED;
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Roam,
                &mut state_events,
            );
            continue;
        };

        if !sensor.is_alive {
            clear_baboon_hawk_target(&mut target);
            stats.move_speed = BABOON_HAWK_MOVE_SPEED;
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Roam,
                &mut state_events,
            );
            continue;
        }

        target.target_entity = employee_entity;
        target.last_known_position = employee_position;
        movement.destination = employee_position;
        stats.move_speed = BABOON_HAWK_CHASE_SPEED;
        movement.current_speed = BABOON_HAWK_CHASE_SPEED;
        move_axis_toward(&mut position, employee_position, stats.move_speed / sim_hz.0);

        if fixed_distance_sq(*position, employee_position) <= fixed_square(stats.attack_range) {
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: BABOON_HAWK_ATTACK_DAMAGE,
                damage_type: DamageType::Standard,
                source: hawk_entity,
            });
        }
    }
}

fn baboon_hawk_investigate_noise(
    mut noise_events: EventReader<NoiseEmittedEvent>,
    mut investigated_events: EventWriter<BaboonHawkNoiseInvestigatedEvent>,
    mut state_events: EventWriter<BaboonHawkStateChangedEvent>,
    sim_hz: Res<SimHz>,
    mut hawks: Query<
        (
            Entity,
            &SimPosition,
            &mut BaboonHawkState,
            &mut BaboonHawkMovement,
            &mut UnitStats,
        ),
        With<BaboonHawk>,
    >,
) {
    for event in noise_events.read() {
        for (hawk_entity, position, mut state, mut movement, mut stats) in hawks.iter_mut() {
            if *state == BaboonHawkState::Dead || *state == BaboonHawkState::Stunned {
                continue;
            }

            if fixed_distance_sq(*position, event.position) > fixed_square(BABOON_HAWK_NOISE_HEAR_RANGE) {
                continue;
            }

            movement.destination = event.position;
            stats.move_speed = BABOON_HAWK_MOVE_SPEED;
            movement.current_speed = BABOON_HAWK_MOVE_SPEED;
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Investigate,
                &mut state_events,
            );

            investigated_events.send(BaboonHawkNoiseInvestigatedEvent {
                hawk: hawk_entity,
                noise_source: event.source,
                position: event.position,
            });
        }
    }

    for (hawk_entity, position, mut state, movement, mut stats) in hawks.iter_mut() {
        if *state != BaboonHawkState::Investigate {
            continue;
        }

        let step = stats.move_speed / sim_hz.0;
        let mut next_position = *position;
        move_axis_toward(&mut next_position, movement.destination, step);

        if fixed_distance_sq(next_position, movement.destination) <= fixed_square(I32F32::lit("1")) {
            stats.move_speed = BABOON_HAWK_MOVE_SPEED;
            set_baboon_hawk_state(
                hawk_entity,
                &mut state,
                BaboonHawkState::Roam,
                &mut state_events,
            );
        }
    }
}

fn baboon_hawk_claim_items(
    mut claimed_events: EventWriter<BaboonHawkItemClaimedEvent>,
    hawks: Query<(Entity, &SimPosition), (With<BaboonHawk>, Without<BaboonHawkItemSensor>)>,
    mut items: Query<(Entity, &SimPosition, &mut BaboonHawkItemSensor), Without<BaboonHawk>>,
) {
    for (hawk_entity, hawk_position) in hawks.iter() {
        for (item_entity, item_position, mut item) in items.iter_mut() {
            if item.claimed {
                continue;
            }

            if fixed_distance_sq(*hawk_position, *item_position)
                > fixed_square(BABOON_HAWK_ITEM_CLAIM_RANGE)
            {
                continue;
            }

            item.claimed = true;
            claimed_events.send(BaboonHawkItemClaimedEvent {
                hawk: hawk_entity,
                item: item_entity,
                item_stable_id: item.stable_id,
            });
        }
    }
}

fn baboon_hawk_apply_stun(
    mut stun_events: EventReader<BaboonHawkStunAppliedEvent>,
    mut state_events: EventWriter<BaboonHawkStateChangedEvent>,
    mut hawks: Query<(Entity, &mut BaboonHawkState, &mut BaboonHawkMovement, &mut UnitStats), With<BaboonHawk>>,
) {
    for event in stun_events.read() {
        let Ok((hawk_entity, mut state, mut movement, mut stats)) = hawks.get_mut(event.hawk) else {
            continue;
        };

        movement.stun_ticks = event.ticks;
        stats.move_speed = I32F32::ZERO;
        set_baboon_hawk_state(
            hawk_entity,
            &mut state,
            BaboonHawkState::Stunned,
            &mut state_events,
        );
    }

    for (hawk_entity, mut state, mut movement, mut stats) in hawks.iter_mut() {
        if *state != BaboonHawkState::Stunned {
            continue;
        }

        if movement.stun_ticks > 0 {
            movement.stun_ticks -= 1;
            continue;
        }

        stats.move_speed = movement.current_speed;
        set_baboon_hawk_state(
            hawk_entity,
            &mut state,
            BaboonHawkState::Roam,
            &mut state_events,
        );
    }
}

fn baboon_hawk_take_damage(
    sim_hz: Res<SimHz>,
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut stun_events: EventWriter<BaboonHawkStunAppliedEvent>,
    mut taken_events: EventWriter<BaboonHawkDamageTakenEvent>,
    mut defeated_events: EventWriter<BaboonHawkDefeatedEvent>,
    mut hawks: Query<(Entity, &BaboonHawk, &mut Health, &mut BaboonHawkState), With<BaboonHawk>>,
) {
    for event in damage_events.read() {
        let Ok((hawk_entity, hawk, mut health, mut state)) = hawks.get_mut(event.target) else {
            continue;
        };

        if *state == BaboonHawkState::Dead {
            continue;
        }

        health.current -= event.raw_amount;
        if health.current < I32F32::ZERO {
            health.current = I32F32::ZERO;
        }

        taken_events.send(BaboonHawkDamageTakenEvent {
            hawk: hawk_entity,
            source: event.source,
            damage: event.raw_amount,
            remaining_health: health.current,
        });

        if health.current <= I32F32::ZERO {
            *state = BaboonHawkState::Dead;
            defeated_events.send(BaboonHawkDefeatedEvent {
                hawk: hawk_entity,
                source: event.source,
                stable_id: hawk.stable_id,
            });
        } else {
            stun_events.send(BaboonHawkStunAppliedEvent {
                hawk: hawk_entity,
                source: event.source,
                ticks: fixed_seconds_to_ticks(BABOON_HAWK_STUN_SECONDS, sim_hz.0),
            });
        }
    }
}

fn baboon_hawk_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    hawks: Query<
        (
            &BaboonHawk,
            &SimPosition,
            &Health,
            &UnitStats,
            &BaboonHawkState,
            &BaboonHawkTarget,
            &BaboonHawkMovement,
        ),
        With<BaboonHawk>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_SOURCE_REVISION as u64);
    checksum.accumulate(BABOON_HAWK_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(BABOON_HAWK_HP.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_MAX_SPAWNED as u64);
    checksum.accumulate(BABOON_HAWK_MOVE_SPEED.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_CHASE_SPEED.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_ATTACK_RANGE.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_ATTACK_SPEED_SECONDS.to_bits() as u64);
    checksum.accumulate(BABOON_HAWK_WATCH_RANGE.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, BABOON_HAWK_ID);
    accumulate_str(&mut checksum, 0x1001, BABOON_HAWK_NAME);
    accumulate_str(&mut checksum, 0x1002, BABOON_HAWK_TYPE);
    accumulate_str(&mut checksum, 0x1003, BABOON_HAWK_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, BABOON_HAWK_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, BABOON_HAWK_EXTRACTED_AT);

    for dependency in BABOON_HAWK_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in BABOON_HAWK_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (hawk, position, health, stats, state, target, movement) in hawks.iter() {
        checksum.accumulate(hawk.stable_id);
        checksum.accumulate(hawk.pack_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(baboon_hawk_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.last_known_position.x.to_bits() as u64);
        checksum.accumulate(target.last_known_position.y.to_bits() as u64);
        checksum.accumulate(movement.current_speed.to_bits() as u64);
        checksum.accumulate(movement.destination.x.to_bits() as u64);
        checksum.accumulate(movement.destination.y.to_bits() as u64);
        checksum.accumulate(movement.intimidation_ticks as u64);
        checksum.accumulate(movement.flee_ticks as u64);
        checksum.accumulate(movement.stun_ticks as u64);
    }
}

fn nearest_visible_employee(
    hawk_position: SimPosition,
    range: I32F32,
    employees: &Query<(Entity, &SimPosition, &BaboonHawkEmployeeSensor), Without<BaboonHawk>>,
) -> Option<(Entity, SimPosition, BaboonHawkEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, BaboonHawkEmployeeSensor, I32F32)> = None;
    let range_sq = fixed_square(range);

    for (entity, position, sensor) in employees.iter() {
        if !sensor.is_alive || !sensor.visible_to_hawk {
            continue;
        }

        let distance_sq = fixed_distance_sq(hawk_position, *position);
        if distance_sq > range_sq {
            continue;
        }

        match best {
            Some((_entity, _position, _sensor, best_distance_sq)) if distance_sq >= best_distance_sq => {}
            _ => best = Some((entity, *position, *sensor, distance_sq)),
        }
    }

    best.map(|(entity, position, sensor, _distance_sq)| (entity, position, sensor))
}

fn employee_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &SimPosition, &BaboonHawkEmployeeSensor), Without<BaboonHawk>>,
) -> Option<(Entity, SimPosition, BaboonHawkEmployeeSensor)> {
    for (entity, position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((entity, *position, *sensor));
        }
    }

    None
}

fn clear_baboon_hawk_target(target: &mut BaboonHawkTarget) {
    target.has_target = false;
    target.target_entity = Entity::PLACEHOLDER;
    target.target_stable_id = 0;
}

fn set_baboon_hawk_state(
    hawk: Entity,
    state: &mut BaboonHawkState,
    next: BaboonHawkState,
    events: &mut EventWriter<BaboonHawkStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(BaboonHawkStateChangedEvent {
        hawk,
        from: previous,
        to: next,
    });
}

fn move_axis_toward(position: &mut SimPosition, target: SimPosition, max_step: I32F32) {
    position.x = move_scalar_toward(position.x, target.x, max_step);
    position.y = move_scalar_toward(position.y, target.y, max_step);
}

fn move_scalar_toward(current: I32F32, target: I32F32, max_step: I32F32) -> I32F32 {
    if current < target {
        let next = current + max_step;
        if next > target {
            target
        } else {
            next
        }
    } else if current > target {
        let next = current - max_step;
        if next < target {
            target
        } else {
            next
        }
    } else {
        current
    }
}

fn fixed_distance_sq(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn fixed_square(value: I32F32) -> I32F32 {
    value * value
}

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    if ticks <= I32F32::ZERO {
        0
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn baboon_hawk_state_bits(state: BaboonHawkState) -> u64 {
    match state {
        BaboonHawkState::Roam => 0,
        BaboonHawkState::Intimidate => 1,
        BaboonHawkState::Chase => 2,
        BaboonHawkState::Investigate => 3,
        BaboonHawkState::Flee => 4,
        BaboonHawkState::Stunned => 5,
        BaboonHawkState::Dead => 6,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}