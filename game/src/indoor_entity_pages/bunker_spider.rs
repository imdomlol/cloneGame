// Sources: vault/indoor_entity_pages/bunker_spider.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};

pub const BUNKER_SPIDER_ID: &str = "bunker_spider";
pub const BUNKER_SPIDER_NAME: &str = "Bunker Spider";
pub const BUNKER_SPIDER_TYPE: &str = "indoor_entity_pages";
pub const BUNKER_SPIDER_SUBTYPE: &str = "enemy";
pub const BUNKER_SPIDER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Bunker_Spider";
pub const BUNKER_SPIDER_SOURCE_REVISION: u32 = 21325;
pub const BUNKER_SPIDER_EXTRACTED_AT: &str = "2026-06-07";
pub const BUNKER_SPIDER_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const BUNKER_SPIDER_DWELLS: &str = "Indoors";
pub const BUNKER_SPIDER_DANGER: &str = "30%";
pub const BUNKER_SPIDER_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const BUNKER_SPIDER_MAX_SPAWNED: usize = 1;
pub const BUNKER_SPIDER_HP: I32F32 = I32F32::lit("5");
pub const BUNKER_SPIDER_ATTACK_DAMAGE: I32F32 = I32F32::lit("90");
pub const BUNKER_SPIDER_ATTACK_SPEED: I32F32 = I32F32::lit("1");
pub const BUNKER_SPIDER_DPS: I32F32 = I32F32::lit("90");
pub const BUNKER_SPIDER_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.25");
pub const BUNKER_SPIDER_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.15");
pub const BUNKER_SPIDER_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("1.1");
pub const BUNKER_SPIDER_INTERNAL_NAME: &str = "SandSpider";
pub const BUNKER_SPIDER_PIP_SIZE: &str = "Small";

pub const BUNKER_SPIDER_ROAM_SPEED: I32F32 = I32F32::lit("1");
pub const BUNKER_SPIDER_HUNT_SPEED: I32F32 = I32F32::lit("1");
pub const BUNKER_SPIDER_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const BUNKER_SPIDER_WATCH_RANGE: I32F32 = I32F32::lit("16");
pub const BUNKER_SPIDER_WEB_TRIGGER_RADIUS: I32F32 = I32F32::lit("1");
pub const BUNKER_SPIDER_MANY_WEBS_THRESHOLD: u16 = 8;

pub const BUNKER_SPIDER_DEPENDS_ON: [&str; 1] = ["employee"];
pub const BUNKER_SPIDER_FRONTMATTER_BEHAVIOR: [&str; 3] = ["Roaming", "Scouting", "Chasing"];

pub const BUNKER_SPIDER_BEHAVIORAL_MECHANICS: [BunkerSpiderBehaviorRule; 7] = [
    BunkerSpiderBehaviorRule {
        condition: "an employee touches one of its webs or breaks one with a melee weapon",
        outcome: "it stops web-making and starts hunting that employee",
    },
    BunkerSpiderBehaviorRule {
        condition: "web-making",
        outcome: "it roams indoors, lays webs, and usually ignores players until it is triggered",
    },
    BunkerSpiderBehaviorRule {
        condition: "it has built many webs",
        outcome: "it can enter a pseudo-hunting mode and sometimes chase a player on sight",
    },
    BunkerSpiderBehaviorRule {
        condition: "it attacks a player",
        outcome: "it deals 90 damage at 1 attack speed for 90 DPS",
    },
    BunkerSpiderBehaviorRule {
        condition: "it is stunned or otherwise affected by stun logic",
        outcome: "apply a stun multiplier of 0.25",
    },
    BunkerSpiderBehaviorRule {
        condition: "it is checked against Zap Gun difficulty",
        outcome: "use a difficulty multiplier of 1.1",
    },
    BunkerSpiderBehaviorRule {
        condition: "it moves through a door",
        outcome: "apply a door speed multiplier of 0.15",
    },
];

pub struct BunkerSpiderPlugin;

impl Plugin for BunkerSpiderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBunkerSpiderEvent>()
            .add_event::<BunkerSpiderStateChangedEvent>()
            .add_event::<BunkerSpiderWebLaidEvent>()
            .add_event::<BunkerSpiderWebTriggeredEvent>()
            .add_event::<BunkerSpiderAttackEvent>()
            .add_event::<BunkerSpiderDoorAttemptEvent>()
            .add_event::<BunkerSpiderDoorAttemptResolvedEvent>()
            .add_event::<BunkerSpiderStunAppliedEvent>()
            .add_event::<BunkerSpiderStunAdjustedEvent>()
            .add_event::<BunkerSpiderZapGunTargetedEvent>()
            .add_event::<BunkerSpiderZapGunDifficultyEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_bunker_spider,
                    bunker_spider_web_make_roam,
                    bunker_spider_trigger_from_webs,
                    bunker_spider_enter_pseudo_hunt_after_many_webs,
                    bunker_spider_chase_seen_employee_in_pseudo_hunt,
                    bunker_spider_hunt_target,
                    bunker_spider_attack_employee,
                    bunker_spider_door_attempt_speed,
                    bunker_spider_apply_stun_multiplier,
                    bunker_spider_report_zap_gun_difficulty,
                    bunker_spider_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BunkerSpider;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BunkerSpiderEmployeeSensor {
    pub stable_id: u64,
    pub touched_web: bool,
    pub broke_web_with_melee: bool,
    pub too_close: bool,
    pub visible_on_sight: bool,
    pub in_attack_range: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderRoamRoute {
    pub area_a: SimPosition,
    pub area_b: SimPosition,
    pub destination_index: u8,
}

impl Default for BunkerSpiderRoamRoute {
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

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BunkerSpiderWebs {
    pub built_count: u16,
    pub web_making: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderTarget {
    pub has_target: bool,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
}

impl Default for BunkerSpiderTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_stable_id: 0,
            last_known_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderAttackTiming {
    pub cooldown_ticks: u32,
    pub timer_ticks: u32,
}

impl Default for BunkerSpiderAttackTiming {
    fn default() -> Self {
        Self {
            cooldown_ticks: 1,
            timer_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BunkerSpiderState {
    #[default]
    WebMaking,
    PseudoHunting,
    Hunting,
}

#[derive(Bundle)]
pub struct BunkerSpiderBundle {
    pub name: Name,
    pub bunker_spider: BunkerSpider,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: BunkerSpiderState,
    pub route: BunkerSpiderRoamRoute,
    pub webs: BunkerSpiderWebs,
    pub target: BunkerSpiderTarget,
    pub attack_timing: BunkerSpiderAttackTiming,
}

impl BunkerSpiderBundle {
    pub fn new(event: SpawnBunkerSpiderEvent) -> Self {
        Self {
            name: Name::new(BUNKER_SPIDER_NAME),
            bunker_spider: BunkerSpider,
            position: event.position,
            health: Health::full(BUNKER_SPIDER_HP),
            stats: UnitStats {
                move_speed: BUNKER_SPIDER_ROAM_SPEED,
                attack_range: BUNKER_SPIDER_ATTACK_RANGE,
                attack_damage: BUNKER_SPIDER_ATTACK_DAMAGE,
                attack_speed: BUNKER_SPIDER_ATTACK_SPEED,
                watch_range: BUNKER_SPIDER_WATCH_RANGE,
            },
            state: BunkerSpiderState::WebMaking,
            route: BunkerSpiderRoamRoute {
                area_a: event.roam_area_a,
                area_b: event.roam_area_b,
                destination_index: 1,
            },
            webs: BunkerSpiderWebs {
                built_count: 0,
                web_making: true,
            },
            target: BunkerSpiderTarget {
                has_target: false,
                target_stable_id: 0,
                last_known_position: event.position,
            },
            attack_timing: BunkerSpiderAttackTiming::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnBunkerSpiderEvent {
    pub position: SimPosition,
    pub roam_area_a: SimPosition,
    pub roam_area_b: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderStateChangedEvent {
    pub spider: Entity,
    pub from: BunkerSpiderState,
    pub to: BunkerSpiderState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderWebLaidEvent {
    pub spider: Entity,
    pub built_count: u16,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderWebTriggeredEvent {
    pub spider: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub reason: BunkerSpiderTriggerReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BunkerSpiderTriggerReason {
    TouchedWeb,
    BrokeWebWithMelee,
    TooClose,
    SightAfterManyWebs,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderAttackEvent {
    pub spider: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub damage: I32F32,
    pub attack_speed: I32F32,
    pub dps: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderDoorAttemptEvent {
    pub spider: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderDoorAttemptResolvedEvent {
    pub spider: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderStunAppliedEvent {
    pub spider: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderStunAdjustedEvent {
    pub spider: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderZapGunTargetedEvent {
    pub spider: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BunkerSpiderZapGunDifficultyEvent {
    pub spider: Entity,
    pub difficulty_modifier: I32F32,
}

fn spawn_bunker_spider(
    mut commands: Commands,
    mut events: EventReader<SpawnBunkerSpiderEvent>,
    spiders: Query<(), With<BunkerSpider>>,
) {
    let mut spawned_count = spiders.iter().count();

    for event in events.read() {
        if spawned_count >= BUNKER_SPIDER_MAX_SPAWNED {
            break;
        }

        commands.spawn(BunkerSpiderBundle::new(*event));
        spawned_count += 1;
    }
}

fn bunker_spider_web_make_roam(
    sim_hz: Res<SimHz>,
    mut web_events: EventWriter<BunkerSpiderWebLaidEvent>,
    mut spiders: Query<
        (
            Entity,
            &mut SimPosition,
            &BunkerSpiderState,
            &UnitStats,
            &mut BunkerSpiderRoamRoute,
            &mut BunkerSpiderWebs,
        ),
        With<BunkerSpider>,
    >,
) {
    for (spider_entity, mut position, state, stats, mut route, mut webs) in spiders.iter_mut() {
        if *state != BunkerSpiderState::WebMaking || !webs.web_making {
            continue;
        }

        let destination = if route.destination_index == 0 {
            route.area_a
        } else {
            route.area_b
        };

        let previous_position = *position;
        move_axis_toward(&mut position, destination, stats.move_speed / sim_hz.0);

        if *position != previous_position {
            webs.built_count = webs.built_count.saturating_add(1);
            web_events.send(BunkerSpiderWebLaidEvent {
                spider: spider_entity,
                built_count: webs.built_count,
            });
        }

        if *position == destination {
            route.destination_index = 1 - route.destination_index;
        }
    }
}

fn bunker_spider_trigger_from_webs(
    mut state_events: EventWriter<BunkerSpiderStateChangedEvent>,
    mut trigger_events: EventWriter<BunkerSpiderWebTriggeredEvent>,
    mut spiders: Query<
        (
            Entity,
            &mut BunkerSpiderState,
            &mut UnitStats,
            &mut BunkerSpiderWebs,
            &mut BunkerSpiderTarget,
        ),
        With<BunkerSpider>,
    >,
    employees: Query<(Entity, &SimPosition, &BunkerSpiderEmployeeSensor)>,
) {
    let Some((employee_entity, employee_position, employee_sensor, reason)) =
        first_web_trigger_employee(&employees)
    else {
        return;
    };

    for (spider_entity, mut state, mut stats, mut webs, mut target) in spiders.iter_mut() {
        if *state == BunkerSpiderState::Hunting {
            continue;
        }

        webs.web_making = false;
        target.has_target = true;
        target.target_stable_id = employee_sensor.stable_id;
        target.last_known_position = employee_position;
        stats.move_speed = BUNKER_SPIDER_HUNT_SPEED;

        trigger_events.send(BunkerSpiderWebTriggeredEvent {
            spider: spider_entity,
            employee: employee_entity,
            employee_stable_id: employee_sensor.stable_id,
            reason,
        });
        set_bunker_spider_state(
            spider_entity,
            &mut state,
            BunkerSpiderState::Hunting,
            &mut state_events,
        );
    }
}

fn bunker_spider_enter_pseudo_hunt_after_many_webs(
    mut state_events: EventWriter<BunkerSpiderStateChangedEvent>,
    mut spiders: Query<(Entity, &mut BunkerSpiderState, &BunkerSpiderWebs), With<BunkerSpider>>,
) {
    for (spider_entity, mut state, webs) in spiders.iter_mut() {
        if *state != BunkerSpiderState::WebMaking {
            continue;
        }

        if webs.built_count < BUNKER_SPIDER_MANY_WEBS_THRESHOLD {
            continue;
        }

        set_bunker_spider_state(
            spider_entity,
            &mut state,
            BunkerSpiderState::PseudoHunting,
            &mut state_events,
        );
    }
}

fn bunker_spider_chase_seen_employee_in_pseudo_hunt(
    mut state_events: EventWriter<BunkerSpiderStateChangedEvent>,
    mut trigger_events: EventWriter<BunkerSpiderWebTriggeredEvent>,
    mut spiders: Query<
        (
            Entity,
            &mut BunkerSpiderState,
            &mut UnitStats,
            &mut BunkerSpiderWebs,
            &mut BunkerSpiderTarget,
        ),
        With<BunkerSpider>,
    >,
    employees: Query<(Entity, &SimPosition, &BunkerSpiderEmployeeSensor)>,
) {
    let Some((employee_entity, employee_position, employee_sensor)) =
        first_visible_employee(&employees)
    else {
        return;
    };

    for (spider_entity, mut state, mut stats, mut webs, mut target) in spiders.iter_mut() {
        if *state != BunkerSpiderState::PseudoHunting {
            continue;
        }

        webs.web_making = false;
        target.has_target = true;
        target.target_stable_id = employee_sensor.stable_id;
        target.last_known_position = employee_position;
        stats.move_speed = BUNKER_SPIDER_HUNT_SPEED;

        trigger_events.send(BunkerSpiderWebTriggeredEvent {
            spider: spider_entity,
            employee: employee_entity,
            employee_stable_id: employee_sensor.stable_id,
            reason: BunkerSpiderTriggerReason::SightAfterManyWebs,
        });
        set_bunker_spider_state(
            spider_entity,
            &mut state,
            BunkerSpiderState::Hunting,
            &mut state_events,
        );
    }
}

fn bunker_spider_hunt_target(
    sim_hz: Res<SimHz>,
    mut queries: ParamSet<(
        Query<(
            &mut SimPosition,
            &BunkerSpiderState,
            &UnitStats,
            &mut BunkerSpiderTarget,
        ), With<BunkerSpider>>,
        Query<(&SimPosition, &BunkerSpiderEmployeeSensor)>,
    )>,
) {
    let mut employee_positions: Vec<(u64, SimPosition)> = queries
        .p1()
        .iter()
        .map(|(position, sensor)| (sensor.stable_id, *position))
        .collect();
    employee_positions.sort_by_key(|(stable_id, _position)| *stable_id);

    for (mut position, state, stats, mut target) in queries.p0().iter_mut() {
        if *state != BunkerSpiderState::Hunting || !target.has_target {
            continue;
        }

        if let Some(employee_position) =
            employee_position_by_stable_id(target.target_stable_id, &employee_positions)
        {
            target.last_known_position = employee_position;
        }

        move_axis_toward(&mut position, target.last_known_position, stats.move_speed / sim_hz.0);
    }
}

fn bunker_spider_attack_employee(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut attack_events: EventWriter<BunkerSpiderAttackEvent>,
    mut spiders: Query<
        (
            Entity,
            &BunkerSpiderState,
            &BunkerSpiderTarget,
            &UnitStats,
            &mut BunkerSpiderAttackTiming,
        ),
        With<BunkerSpider>,
    >,
    employees: Query<(Entity, &BunkerSpiderEmployeeSensor)>,
) {
    for (spider_entity, state, target, stats, mut timing) in spiders.iter_mut() {
        if *state != BunkerSpiderState::Hunting || !target.has_target {
            continue;
        }

        timing.cooldown_ticks = fixed_seconds_to_ticks(stats.attack_speed, sim_hz.0);

        if timing.timer_ticks > 0 {
            timing.timer_ticks -= 1;
            continue;
        }

        let Some((employee_entity, employee_sensor)) =
            attackable_employee_by_stable_id(target.target_stable_id, &employees)
        else {
            continue;
        };

        attack_events.send(BunkerSpiderAttackEvent {
            spider: spider_entity,
            employee: employee_entity,
            employee_stable_id: employee_sensor.stable_id,
            damage: BUNKER_SPIDER_ATTACK_DAMAGE,
            attack_speed: BUNKER_SPIDER_ATTACK_SPEED,
            dps: BUNKER_SPIDER_DPS,
        });
        damage_events.send(IncomingDamageEvent {
            target: employee_entity,
            raw_amount: BUNKER_SPIDER_ATTACK_DAMAGE,
            damage_type: DamageType::Standard,
            source: spider_entity,
        });
        timing.timer_ticks = timing.cooldown_ticks;
    }
}

fn bunker_spider_door_attempt_speed(
    mut events: EventReader<BunkerSpiderDoorAttemptEvent>,
    mut resolved_events: EventWriter<BunkerSpiderDoorAttemptResolvedEvent>,
    spiders: Query<(), With<BunkerSpider>>,
) {
    for event in events.read() {
        if spiders.get(event.spider).is_err() {
            continue;
        }

        resolved_events.send(BunkerSpiderDoorAttemptResolvedEvent {
            spider: event.spider,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(
                event.base_open_ticks,
                BUNKER_SPIDER_DOOR_SPEED_MULTIPLIER,
            ),
        });
    }
}

fn bunker_spider_apply_stun_multiplier(
    mut events: EventReader<BunkerSpiderStunAppliedEvent>,
    mut adjusted_events: EventWriter<BunkerSpiderStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(BunkerSpiderStunAdjustedEvent {
            spider: event.spider,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, BUNKER_SPIDER_STUN_MULTIPLIER),
        });
    }
}

fn bunker_spider_report_zap_gun_difficulty(
    mut events: EventReader<BunkerSpiderZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<BunkerSpiderZapGunDifficultyEvent>,
    spiders: Query<(), With<BunkerSpider>>,
) {
    for event in events.read() {
        if spiders.get(event.spider).is_err() {
            continue;
        }

        difficulty_events.send(BunkerSpiderZapGunDifficultyEvent {
            spider: event.spider,
            difficulty_modifier: BUNKER_SPIDER_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn bunker_spider_checksum(
    mut checksum: ResMut<SimChecksumState>,
    spiders: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &BunkerSpiderState,
            &BunkerSpiderRoamRoute,
            &BunkerSpiderWebs,
            &BunkerSpiderTarget,
            &BunkerSpiderAttackTiming,
        ),
        With<BunkerSpider>,
    >,
) {
    for (position, health, stats, state, route, webs, target, timing) in spiders.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(bunker_spider_state_bits(*state));
        checksum.accumulate(route.area_a.x.to_bits() as u64);
        checksum.accumulate(route.area_a.y.to_bits() as u64);
        checksum.accumulate(route.area_b.x.to_bits() as u64);
        checksum.accumulate(route.area_b.y.to_bits() as u64);
        checksum.accumulate(route.destination_index as u64);
        checksum.accumulate(webs.built_count as u64);
        checksum.accumulate(webs.web_making as u64);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.last_known_position.x.to_bits() as u64);
        checksum.accumulate(target.last_known_position.y.to_bits() as u64);
        checksum.accumulate(timing.cooldown_ticks as u64);
        checksum.accumulate(timing.timer_ticks as u64);
    }
}

fn first_web_trigger_employee(
    employees: &Query<(Entity, &SimPosition, &BunkerSpiderEmployeeSensor)>,
) -> Option<(Entity, SimPosition, BunkerSpiderEmployeeSensor, BunkerSpiderTriggerReason)> {
    let mut best: Option<(Entity, SimPosition, BunkerSpiderEmployeeSensor, BunkerSpiderTriggerReason)> = None;

    for (entity, position, sensor) in employees.iter() {
        let reason = if sensor.touched_web {
            BunkerSpiderTriggerReason::TouchedWeb
        } else if sensor.broke_web_with_melee {
            BunkerSpiderTriggerReason::BrokeWebWithMelee
        } else if sensor.too_close {
            BunkerSpiderTriggerReason::TooClose
        } else {
            continue;
        };

        if let Some((_best_entity, _best_position, best_sensor, _best_reason)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *position, *sensor, reason));
    }

    best
}

fn first_visible_employee(
    employees: &Query<(Entity, &SimPosition, &BunkerSpiderEmployeeSensor)>,
) -> Option<(Entity, SimPosition, BunkerSpiderEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, BunkerSpiderEmployeeSensor)> = None;

    for (entity, position, sensor) in employees.iter() {
        if !sensor.visible_on_sight {
            continue;
        }

        if let Some((_best_entity, _best_position, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *position, *sensor));
    }

    best
}

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &[(u64, SimPosition)],
) -> Option<SimPosition> {
    for (employee_stable_id, position) in employees.iter() {
        if *employee_stable_id == stable_id {
            return Some(*position);
        }
    }

    None
}

fn attackable_employee_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &BunkerSpiderEmployeeSensor)>,
) -> Option<(Entity, BunkerSpiderEmployeeSensor)> {
    for (entity, sensor) in employees.iter() {
        if sensor.stable_id == stable_id && sensor.in_attack_range {
            return Some((entity, *sensor));
        }
    }

    None
}

fn set_bunker_spider_state(
    spider: Entity,
    state: &mut BunkerSpiderState,
    next: BunkerSpiderState,
    events: &mut EventWriter<BunkerSpiderStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(BunkerSpiderStateChangedEvent {
        spider,
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

fn bunker_spider_state_bits(state: BunkerSpiderState) -> u64 {
    match state {
        BunkerSpiderState::WebMaking => 0,
        BunkerSpiderState::PseudoHunting => 1,
        BunkerSpiderState::Hunting => 2,
    }
}