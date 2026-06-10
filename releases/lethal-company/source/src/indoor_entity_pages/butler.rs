// Sources: vault/indoor_entity_pages/butler.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, EntityKilledEvent, GameSeed, Health, IncomingDamageEvent,
    SimChecksumState, SimHz, SimPosition, SimTick, UnitStats,
};

pub const BUTLER_ID: &str = "butler";
pub const BUTLER_NAME: &str = "Butler";
pub const BUTLER_TYPE: &str = "indoor_entity_pages";
pub const BUTLER_SUBTYPE: &str = "creature";
pub const BUTLER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Butler";
pub const BUTLER_SOURCE_REVISION: u32 = 20958;
pub const BUTLER_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const BUTLER_CONFIDENCE_BASIS_POINTS: u16 = 91;

pub const BUTLER_DWELLS: &str = "Inside";
pub const BUTLER_HP: I32F32 = I32F32::lit("8");
pub const BUTLER_HP_SINGLEPLAYER: I32F32 = I32F32::lit("2");
pub const BUTLER_SHOVEL_HP: I32F32 = I32F32::lit("8");
pub const BUTLER_SHOVEL_HP_SINGLEPLAYER: I32F32 = I32F32::lit("2");
pub const BUTLER_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const BUTLER_MAX_SPAWNED: usize = 7;
pub const BUTLER_ATTACK_DAMAGE: I32F32 = I32F32::lit("10");
pub const BUTLER_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.6");
pub const BUTLER_STUN_GRENADE: &str = "Immune";
pub const BUTLER_SHOCK_RESPONSE: &str = "Immune";
pub const BUTLER_RADAR_PIP_SIZE: &str = "Medium";
pub const BUTLER_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0.5");
pub const BUTLER_INTERNAL_NAME: &str = "Butler";
pub const BUTLER_LEAVE_TIME_SECONDS: I32F32 = I32F32::lit("15");

pub const BUTLER_ISOLATION_SECONDS: I32F32 = I32F32::lit("6");
pub const BUTLER_LINGER_SECONDS: I32F32 = I32F32::lit("12");
pub const BUTLER_ROAMING_NEAR_NODE_METERS: I32F32 = I32F32::lit("5");
pub const BUTLER_RUNNING_NEAR_NODE_METERS: I32F32 = I32F32::lit("10");
pub const BUTLER_CLOSE_CONTACT_SECONDS: I32F32 = I32F32::lit("0.25");
pub const BUTLER_CLOSE_CONTACT_BERSERK_PERCENT: u32 = 14;
pub const BUTLER_CLOSE_CONTACT_BERSERK_SECONDS: I32F32 = I32F32::lit("3");
pub const BUTLER_BERSERK_SECONDS: I32F32 = I32F32::lit("8");
pub const BUTLER_DEATH_EXPLOSION_DAMAGE: I32F32 = I32F32::lit("30");
pub const BUTLER_KNIFE_ONE_HIT_DAMAGE: I32F32 = I32F32::lit("1000000");
pub const BUTLER_ROAM_SPEED: I32F32 = I32F32::lit("1");
pub const BUTLER_RUN_SPEED: I32F32 = I32F32::lit("2");
pub const BUTLER_MURDER_SPEED: I32F32 = I32F32::lit("3");
pub const BUTLER_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const BUTLER_ATTACK_SPEED: I32F32 = I32F32::lit("0.4");
pub const BUTLER_WATCH_RANGE: I32F32 = I32F32::lit("30");

pub const KITCHEN_KNIFE_ID: &str = "kitchen_knife";
pub const MASK_HORNETS_ID: &str = "mask_hornets";

pub const BUTLER_DEPENDS_ON: [&str; 4] = [
    "lethal_company",
    "employee",
    "kitchen_knife",
    "mask_hornets",
];

pub const BUTLER_FRONTMATTER_BEHAVIOR: [&str; 4] = [
    "Roaming",
    "Lingering",
    "Murder",
    "Berserk",
];

pub const BUTLER_BEHAVIORAL_MECHANICS: [ButlerBehaviorRule; 18] = [
    ButlerBehaviorRule {
        condition: "a Butler is in roaming phase and can see multiple employees",
        outcome: "it stays in roaming instead of committing to a target",
    },
    ButlerBehaviorRule {
        condition: "a Butler spots an isolated employee or is left alone with one for 6 seconds",
        outcome: "it enters lingering phase",
    },
    ButlerBehaviorRule {
        condition: "roaming and 6 seconds pass without spotting an employee",
        outcome: "it starts running and switches to choosing its next node after coming within 10 meters instead of 5 meters",
    },
    ButlerBehaviorRule {
        condition: "lingering and the target has not seen another employee for 12 seconds",
        outcome: "the Butler checks for other targets and enters murder phase",
    },
    ButlerBehaviorRule {
        condition: "in murder phase",
        outcome: "the Butler puts away its broom, draws a kitchen knife, sprints at the target, and attacks rapidly",
    },
    ButlerBehaviorRule {
        condition: "the target leaves line of sight after the 12-second wait and later reappears",
        outcome: "the Butler can attack immediately without waiting again",
    },
    ButlerBehaviorRule {
        condition: "the Butler kills an employee",
        outcome: "it flees for 15 seconds before returning to roaming",
    },
    ButlerBehaviorRule {
        condition: "the Butler sees another employee during those 15 seconds",
        outcome: "it switches to lingering instead",
    },
    ButlerBehaviorRule {
        condition: "a non-kitchen_knife melee weapon hits a Butler",
        outcome: "it enters berserk phase",
    },
    ButlerBehaviorRule {
        condition: "a Butler sees the body of an employee it just killed",
        outcome: "it enters berserk phase",
    },
    ButlerBehaviorRule {
        condition: "a Butler remains in physical contact too long",
        outcome: "it enters berserk phase",
    },
    ButlerBehaviorRule {
        condition: "a Butler is in roaming or lingering phase and an employee stays in close contact for 0.25 seconds",
        outcome: "there is a 14% chance to trigger berserk phase on that tick",
    },
    ButlerBehaviorRule {
        condition: "berserk",
        outcome: "the Butler attacks the nearest employee and immediately retargets if another employee gets closer",
    },
    ButlerBehaviorRule {
        condition: "berserk was triggered by close contact",
        outcome: "it lasts 3 seconds",
    },
    ButlerBehaviorRule {
        condition: "berserk was not triggered by close contact",
        outcome: "it lasts 8 seconds",
    },
    ButlerBehaviorRule {
        condition: "one Butler sees another Butler in berserk phase",
        outcome: "it immediately enters berserk phase too",
    },
    ButlerBehaviorRule {
        condition: "a Butler is killed",
        outcome: "it explodes for 30 damage, can throw nearby employees into fall damage, drops its kitchen_knife, and spawns mask_hornets",
    },
    ButlerBehaviorRule {
        condition: "a Butler is active",
        outcome: "it can see through doors, so a closed door does not break line of sight",
    },
];

pub struct ButlerPlugin;

impl Plugin for ButlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnButlerEvent>()
            .add_event::<ButlerStateChangedEvent>()
            .add_event::<ButlerWeaponHitEvent>()
            .add_event::<ButlerDoorAttemptEvent>()
            .add_event::<ButlerDoorAttemptResolvedEvent>()
            .add_event::<ButlerStunAppliedEvent>()
            .add_event::<ButlerStunAdjustedEvent>()
            .add_event::<ButlerDeathBurstEvent>()
            .add_event::<ButlerDropItemEvent>()
            .add_event::<ButlerSpawnEntityEvent>()
            .add_event::<ButlerContactBerserkRollEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_butler,
                    butler_roaming_visibility,
                    butler_roaming_timeout,
                    butler_lingering_timeout,
                    butler_murder_attack,
                    butler_flee_after_kill,
                    butler_weapon_hit,
                    butler_close_contact_berserk,
                    butler_berserk_retarget,
                    butler_berserk_contagion,
                    butler_berserk_timeout,
                    butler_door_attempt_speed,
                    butler_apply_stun_multiplier,
                    butler_death_effects,
                    butler_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Butler;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerAi {
    pub state: ButlerState,
    pub target_stable_id: u64,
    pub timer_ticks: u32,
    pub running_to_next_node: bool,
    pub attack_cooldown_ticks: u32,
    pub killed_employee_stable_id: u64,
}

impl Default for ButlerAi {
    fn default() -> Self {
        Self {
            state: ButlerState::Roaming,
            target_stable_id: 0,
            timer_ticks: 0,
            running_to_next_node: false,
            attack_cooldown_ticks: 0,
            killed_employee_stable_id: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerRoamRoute {
    pub node_a: SimPosition,
    pub node_b: SimPosition,
    pub destination_index: u8,
    pub near_node_distance: I32F32,
}

impl Default for ButlerRoamRoute {
    fn default() -> Self {
        Self {
            node_a: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            node_b: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            destination_index: 1,
            near_node_distance: BUTLER_ROAMING_NEAR_NODE_METERS,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButlerEmployeeSensor {
    pub stable_id: u64,
    pub can_be_seen_by_butler: bool,
    pub is_isolated: bool,
    pub has_seen_another_employee: bool,
    pub in_close_contact: bool,
    pub body_of_recent_butler_kill: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButlerActiveLineOfSight {
    pub sees_through_doors: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButlerState {
    #[default]
    Roaming,
    Lingering,
    Murder,
    Fleeing,
    Berserk,
}

#[derive(Bundle)]
pub struct ButlerBundle {
    pub name: Name,
    pub butler: Butler,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub ai: ButlerAi,
    pub route: ButlerRoamRoute,
    pub line_of_sight: ButlerActiveLineOfSight,
}

impl ButlerBundle {
    pub fn new(event: SpawnButlerEvent) -> Self {
        Self {
            name: Name::new(BUTLER_NAME),
            butler: Butler,
            position: event.position,
            health: Health::full(if event.singleplayer {
                BUTLER_HP_SINGLEPLAYER
            } else {
                BUTLER_HP
            }),
            stats: UnitStats {
                move_speed: BUTLER_ROAM_SPEED,
                attack_range: BUTLER_ATTACK_RANGE,
                attack_damage: BUTLER_ATTACK_DAMAGE,
                attack_speed: BUTLER_ATTACK_SPEED,
                watch_range: BUTLER_WATCH_RANGE,
            },
            ai: ButlerAi::default(),
            route: ButlerRoamRoute {
                node_a: event.roam_node_a,
                node_b: event.roam_node_b,
                destination_index: 1,
                near_node_distance: BUTLER_ROAMING_NEAR_NODE_METERS,
            },
            line_of_sight: ButlerActiveLineOfSight {
                sees_through_doors: true,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnButlerEvent {
    pub position: SimPosition,
    pub roam_node_a: SimPosition,
    pub roam_node_b: SimPosition,
    pub singleplayer: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerStateChangedEvent {
    pub butler: Entity,
    pub from: ButlerState,
    pub to: ButlerState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerWeaponHitEvent {
    pub butler: Entity,
    pub source: Entity,
    pub weapon_id: &'static str,
    pub melee: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerDoorAttemptEvent {
    pub butler: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerDoorAttemptResolvedEvent {
    pub butler: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
    pub bug_can_reset_timer: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerStunAppliedEvent {
    pub butler: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerStunAdjustedEvent {
    pub butler: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
    pub stun_grenade_immune: bool,
    pub shock_immune: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerDeathBurstEvent {
    pub butler: Entity,
    pub explosion_damage: I32F32,
    pub can_throw_into_fall_damage: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerDropItemEvent {
    pub butler: Entity,
    pub item_id: &'static str,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerSpawnEntityEvent {
    pub butler: Entity,
    pub entity_id: &'static str,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButlerContactBerserkRollEvent {
    pub butler: Entity,
    pub employee_stable_id: u64,
    pub roll_percent: u32,
    pub triggered: bool,
}

fn spawn_butler(
    mut commands: Commands,
    mut events: EventReader<SpawnButlerEvent>,
    butlers: Query<(), With<Butler>>,
) {
    let mut spawned_count = butlers.iter().count();

    for event in events.read() {
        if spawned_count >= BUTLER_MAX_SPAWNED {
            break;
        }

        commands.spawn(ButlerBundle::new(*event));
        spawned_count += 1;
    }
}

fn butler_roaming_visibility(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ButlerStateChangedEvent>,
    mut butlers: Query<(Entity, &mut ButlerAi), With<Butler>>,
    employees: Query<&ButlerEmployeeSensor>,
) {
    for (butler_entity, mut ai) in butlers.iter_mut() {
        if ai.state != ButlerState::Roaming {
            continue;
        }

        let visible_count = visible_employee_count(&employees);
        if visible_count > 1 {
            ai.target_stable_id = 0;
            continue;
        }

        let Some(target) = isolated_visible_employee(&employees) else {
            continue;
        };

        ai.target_stable_id = target.stable_id;
        ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_ISOLATION_SECONDS, sim_hz.0);
        set_butler_state(
            butler_entity,
            &mut ai,
            ButlerState::Lingering,
            &mut state_events,
        );
    }
}

fn butler_roaming_timeout(
    sim_hz: Res<SimHz>,
    mut butlers: Query<(&mut ButlerAi, &mut UnitStats, &mut ButlerRoamRoute), With<Butler>>,
    employees: Query<&ButlerEmployeeSensor>,
) {
    for (mut ai, mut stats, mut route) in butlers.iter_mut() {
        if ai.state != ButlerState::Roaming {
            continue;
        }

        if visible_employee_count(&employees) > 0 {
            ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_ISOLATION_SECONDS, sim_hz.0);
            continue;
        }

        if ai.timer_ticks == 0 {
            ai.running_to_next_node = true;
            stats.move_speed = BUTLER_RUN_SPEED;
            route.near_node_distance = BUTLER_RUNNING_NEAR_NODE_METERS;
            ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_ISOLATION_SECONDS, sim_hz.0);
            continue;
        }

        ai.timer_ticks -= 1;
    }
}

fn butler_lingering_timeout(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ButlerStateChangedEvent>,
    mut butlers: Query<(Entity, &mut ButlerAi, &mut UnitStats), With<Butler>>,
    employees: Query<&ButlerEmployeeSensor>,
) {
    for (butler_entity, mut ai, mut stats) in butlers.iter_mut() {
        if ai.state != ButlerState::Lingering {
            continue;
        }

        if target_has_seen_other_employee(ai.target_stable_id, &employees) {
            ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_LINGER_SECONDS, sim_hz.0);
            continue;
        }

        if ai.timer_ticks > 0 {
            ai.timer_ticks -= 1;
            continue;
        }

        if let Some(target) = isolated_visible_employee(&employees) {
            ai.target_stable_id = target.stable_id;
        }

        stats.move_speed = BUTLER_MURDER_SPEED;
        ai.attack_cooldown_ticks = 0;
        set_butler_state(
            butler_entity,
            &mut ai,
            ButlerState::Murder,
            &mut state_events,
        );
    }
}

fn butler_murder_attack(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut butlers: Query<(Entity, &mut ButlerAi, &SimPosition, &UnitStats), With<Butler>>,
    employees: Query<(Entity, &SimPosition, &ButlerEmployeeSensor)>,
) {
    for (butler_entity, mut ai, butler_position, stats) in butlers.iter_mut() {
        if ai.state != ButlerState::Murder {
            continue;
        }

        if ai.attack_cooldown_ticks > 0 {
            ai.attack_cooldown_ticks -= 1;
            continue;
        }

        let Some((employee_entity, employee_position, sensor)) =
            employee_by_stable_id(ai.target_stable_id, &employees)
        else {
            continue;
        };

        if distance_squared(*butler_position, employee_position) > stats.attack_range * stats.attack_range {
            continue;
        }

        damage_events.send(IncomingDamageEvent {
            target: employee_entity,
            raw_amount: stats.attack_damage,
            damage_type: DamageType::Standard,
            source: butler_entity,
        });
        ai.killed_employee_stable_id = sensor.stable_id;
        ai.attack_cooldown_ticks = fixed_seconds_to_ticks(stats.attack_speed, sim_hz.0);
    }
}

fn butler_flee_after_kill(
    sim_hz: Res<SimHz>,
    mut killed_events: EventReader<EntityKilledEvent>,
    mut state_events: EventWriter<ButlerStateChangedEvent>,
    mut butlers: Query<(Entity, &mut ButlerAi, &mut UnitStats), With<Butler>>,
    employees: Query<&ButlerEmployeeSensor>,
) {
    for _event in killed_events.read() {
        for (butler_entity, mut ai, mut stats) in butlers.iter_mut() {
            if ai.state != ButlerState::Murder {
                continue;
            }

            if let Some(target) = isolated_visible_employee(&employees) {
                ai.target_stable_id = target.stable_id;
                ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_LINGER_SECONDS, sim_hz.0);
                stats.move_speed = BUTLER_ROAM_SPEED;
                set_butler_state(
                    butler_entity,
                    &mut ai,
                    ButlerState::Lingering,
                    &mut state_events,
                );
                continue;
            }

            ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_LEAVE_TIME_SECONDS, sim_hz.0);
            stats.move_speed = BUTLER_RUN_SPEED;
            set_butler_state(
                butler_entity,
                &mut ai,
                ButlerState::Fleeing,
                &mut state_events,
            );
        }
    }

    for (butler_entity, mut ai, mut stats) in butlers.iter_mut() {
        if ai.state != ButlerState::Fleeing {
            continue;
        }

        if let Some(target) = isolated_visible_employee(&employees) {
            ai.target_stable_id = target.stable_id;
            ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_LINGER_SECONDS, sim_hz.0);
            stats.move_speed = BUTLER_ROAM_SPEED;
            set_butler_state(
                butler_entity,
                &mut ai,
                ButlerState::Lingering,
                &mut state_events,
            );
            continue;
        }

        if ai.timer_ticks > 0 {
            ai.timer_ticks -= 1;
            continue;
        }

        ai.target_stable_id = 0;
        stats.move_speed = BUTLER_ROAM_SPEED;
        set_butler_state(
            butler_entity,
            &mut ai,
            ButlerState::Roaming,
            &mut state_events,
        );
    }
}

fn butler_weapon_hit(
    sim_hz: Res<SimHz>,
    mut events: EventReader<ButlerWeaponHitEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut state_events: EventWriter<ButlerStateChangedEvent>,
    mut butlers: Query<(Entity, &mut ButlerAi), With<Butler>>,
) {
    for event in events.read() {
        let Ok((butler_entity, mut ai)) = butlers.get_mut(event.butler) else {
            continue;
        };

        if event.weapon_id == KITCHEN_KNIFE_ID {
            damage_events.send(IncomingDamageEvent {
                target: butler_entity,
                raw_amount: BUTLER_KNIFE_ONE_HIT_DAMAGE,
                damage_type: DamageType::Standard,
                source: event.source,
            });
            continue;
        }

        if event.melee {
            ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_BERSERK_SECONDS, sim_hz.0);
            set_butler_state(
                butler_entity,
                &mut ai,
                ButlerState::Berserk,
                &mut state_events,
            );
        }
    }
}

fn butler_close_contact_berserk(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut roll_events: EventWriter<ButlerContactBerserkRollEvent>,
    mut state_events: EventWriter<ButlerStateChangedEvent>,
    mut butlers: Query<(Entity, &mut ButlerAi), With<Butler>>,
    employees: Query<&ButlerEmployeeSensor>,
) {
    for (butler_entity, mut ai) in butlers.iter_mut() {
        if ai.state != ButlerState::Roaming && ai.state != ButlerState::Lingering {
            continue;
        }

        let Some(employee) = close_contact_employee(&employees) else {
            continue;
        };

        let salt = 0x6275_746c_6572_0001_u64 ^ employee.stable_id;
        let mut rng = tick_rng(game_seed.0, sim_tick.0, salt);
        let roll_percent = rng.next_u32() % 100;
        let triggered = roll_percent < BUTLER_CLOSE_CONTACT_BERSERK_PERCENT;

        roll_events.send(ButlerContactBerserkRollEvent {
            butler: butler_entity,
            employee_stable_id: employee.stable_id,
            roll_percent,
            triggered,
        });

        if triggered {
            ai.target_stable_id = employee.stable_id;
            ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_CLOSE_CONTACT_BERSERK_SECONDS, sim_hz.0);
            set_butler_state(
                butler_entity,
                &mut ai,
                ButlerState::Berserk,
                &mut state_events,
            );
        }
    }
}

fn butler_berserk_retarget(
    mut butlers: Query<(&mut ButlerAi, &SimPosition), With<Butler>>,
    employees: Query<(&SimPosition, &ButlerEmployeeSensor)>,
) {
    for (mut ai, butler_position) in butlers.iter_mut() {
        if ai.state != ButlerState::Berserk {
            continue;
        }

        if let Some(sensor) = nearest_employee(*butler_position, &employees) {
            ai.target_stable_id = sensor.stable_id;
        }
    }
}

fn butler_berserk_contagion(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ButlerStateChangedEvent>,
    berserk_butlers: Query<Entity, (With<Butler>, With<ButlerActiveLineOfSight>)>,
    mut all_butlers: Query<(Entity, &mut ButlerAi), With<Butler>>,
) {
    let has_berserk = all_butlers
        .iter()
        .any(|(_entity, ai)| ai.state == ButlerState::Berserk);

    if !has_berserk {
        return;
    }

    let active_butler_count = berserk_butlers.iter().count();
    if active_butler_count < 2 {
        return;
    }

    for (butler_entity, mut ai) in all_butlers.iter_mut() {
        if ai.state == ButlerState::Berserk {
            continue;
        }

        ai.timer_ticks = fixed_seconds_to_ticks(BUTLER_BERSERK_SECONDS, sim_hz.0);
        set_butler_state(
            butler_entity,
            &mut ai,
            ButlerState::Berserk,
            &mut state_events,
        );
    }
}

fn butler_berserk_timeout(
    mut state_events: EventWriter<ButlerStateChangedEvent>,
    mut butlers: Query<(Entity, &mut ButlerAi, &mut UnitStats), With<Butler>>,
) {
    for (butler_entity, mut ai, mut stats) in butlers.iter_mut() {
        if ai.state != ButlerState::Berserk {
            continue;
        }

        if ai.timer_ticks > 0 {
            ai.timer_ticks -= 1;
            continue;
        }

        ai.target_stable_id = 0;
        stats.move_speed = BUTLER_ROAM_SPEED;
        set_butler_state(
            butler_entity,
            &mut ai,
            ButlerState::Roaming,
            &mut state_events,
        );
    }
}

fn butler_door_attempt_speed(
    mut events: EventReader<ButlerDoorAttemptEvent>,
    mut resolved_events: EventWriter<ButlerDoorAttemptResolvedEvent>,
    butlers: Query<(), With<Butler>>,
) {
    for event in events.read() {
        if butlers.get(event.butler).is_err() {
            continue;
        }

        resolved_events.send(ButlerDoorAttemptResolvedEvent {
            butler: event.butler,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, BUTLER_DOOR_OPEN_SPEED),
            bug_can_reset_timer: true,
        });
    }
}

fn butler_apply_stun_multiplier(
    mut events: EventReader<ButlerStunAppliedEvent>,
    mut adjusted_events: EventWriter<ButlerStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(ButlerStunAdjustedEvent {
            butler: event.butler,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, BUTLER_STUN_MULTIPLIER),
            stun_grenade_immune: true,
            shock_immune: true,
        });
    }
}

fn butler_death_effects(
    mut killed_events: EventReader<EntityKilledEvent>,
    mut burst_events: EventWriter<ButlerDeathBurstEvent>,
    mut drop_events: EventWriter<ButlerDropItemEvent>,
    mut spawn_events: EventWriter<ButlerSpawnEntityEvent>,
    butlers: Query<(), With<Butler>>,
) {
    for event in killed_events.read() {
        if butlers.get(event.entity).is_err() {
            continue;
        }

        burst_events.send(ButlerDeathBurstEvent {
            butler: event.entity,
            explosion_damage: BUTLER_DEATH_EXPLOSION_DAMAGE,
            can_throw_into_fall_damage: true,
        });
        drop_events.send(ButlerDropItemEvent {
            butler: event.entity,
            item_id: KITCHEN_KNIFE_ID,
        });
        spawn_events.send(ButlerSpawnEntityEvent {
            butler: event.entity,
            entity_id: MASK_HORNETS_ID,
        });
    }
}

fn butler_checksum(
    mut checksum: ResMut<SimChecksumState>,
    butlers: Query<(
        &SimPosition,
        &Health,
        &UnitStats,
        &ButlerAi,
        &ButlerRoamRoute,
        &ButlerActiveLineOfSight,
    ), With<Butler>>,
) {
    for (position, health, stats, ai, route, line_of_sight) in butlers.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(butler_state_bits(ai.state));
        checksum.accumulate(ai.target_stable_id);
        checksum.accumulate(ai.timer_ticks as u64);
        checksum.accumulate(ai.running_to_next_node as u64);
        checksum.accumulate(ai.attack_cooldown_ticks as u64);
        checksum.accumulate(ai.killed_employee_stable_id);
        checksum.accumulate(route.node_a.x.to_bits() as u64);
        checksum.accumulate(route.node_a.y.to_bits() as u64);
        checksum.accumulate(route.node_b.x.to_bits() as u64);
        checksum.accumulate(route.node_b.y.to_bits() as u64);
        checksum.accumulate(route.destination_index as u64);
        checksum.accumulate(route.near_node_distance.to_bits() as u64);
        checksum.accumulate(line_of_sight.sees_through_doors as u64);
    }
}

fn visible_employee_count(employees: &Query<&ButlerEmployeeSensor>) -> usize {
    employees
        .iter()
        .filter(|employee| employee.can_be_seen_by_butler)
        .count()
}

fn isolated_visible_employee(
    employees: &Query<&ButlerEmployeeSensor>,
) -> Option<ButlerEmployeeSensor> {
    let mut best: Option<ButlerEmployeeSensor> = None;

    for employee in employees.iter() {
        if !employee.can_be_seen_by_butler || !employee.is_isolated {
            continue;
        }

        if let Some(best_employee) = best {
            if employee.stable_id >= best_employee.stable_id {
                continue;
            }
        }

        best = Some(*employee);
    }

    best
}

fn close_contact_employee(employees: &Query<&ButlerEmployeeSensor>) -> Option<ButlerEmployeeSensor> {
    let mut best: Option<ButlerEmployeeSensor> = None;

    for employee in employees.iter() {
        if !employee.in_close_contact {
            continue;
        }

        if let Some(best_employee) = best {
            if employee.stable_id >= best_employee.stable_id {
                continue;
            }
        }

        best = Some(*employee);
    }

    best
}

fn target_has_seen_other_employee(
    target_stable_id: u64,
    employees: &Query<&ButlerEmployeeSensor>,
) -> bool {
    employees
        .iter()
        .any(|employee| employee.stable_id == target_stable_id && employee.has_seen_another_employee)
}

fn employee_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &SimPosition, &ButlerEmployeeSensor)>,
) -> Option<(Entity, SimPosition, ButlerEmployeeSensor)> {
    for (entity, position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((entity, *position, *sensor));
        }
    }

    None
}

fn nearest_employee(
    origin: SimPosition,
    employees: &Query<(&SimPosition, &ButlerEmployeeSensor)>,
) -> Option<ButlerEmployeeSensor> {
    let mut best: Option<(I32F32, ButlerEmployeeSensor)> = None;

    for (position, sensor) in employees.iter() {
        let distance = distance_squared(origin, *position);

        if let Some((best_distance, best_sensor)) = best {
            if distance > best_distance {
                continue;
            }

            if distance == best_distance && sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((distance, *sensor));
    }

    best.map(|(_distance, sensor)| sensor)
}

fn set_butler_state(
    butler: Entity,
    ai: &mut ButlerAi,
    next: ButlerState,
    events: &mut EventWriter<ButlerStateChangedEvent>,
) {
    if ai.state == next {
        return;
    }

    let previous = ai.state;
    ai.state = next;
    events.send(ButlerStateChangedEvent {
        butler,
        from: previous,
        to: next,
    });
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
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

fn butler_state_bits(state: ButlerState) -> u64 {
    match state {
        ButlerState::Roaming => 0,
        ButlerState::Lingering => 1,
        ButlerState::Murder => 2,
        ButlerState::Fleeing => 3,
        ButlerState::Berserk => 4,
    }
}