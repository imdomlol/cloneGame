// Sources: vault/indoor_entity_pages/mask_hornets.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};

pub const MASK_HORNETS_ID: &str = "mask_hornets";
pub const MASK_HORNETS_NAME: &str = "Mask Hornets";
pub const MASK_HORNETS_TYPE: &str = "indoor_entity_pages";
pub const MASK_HORNETS_SUBTYPE: &str = "creature";
pub const MASK_HORNETS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Mask_Hornets";
pub const MASK_HORNETS_SOURCE_REVISION: u32 = 19955;
pub const MASK_HORNETS_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const MASK_HORNETS_CONFIDENCE_BASIS_POINTS: u16 = 87;

pub const MASK_HORNETS_DWELLS: &str = "Inside";
pub const MASK_HORNETS_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const MASK_HORNETS_MAX_SPAWNED: usize = 7;
pub const MASK_HORNETS_ATTACK_SPEED: I32F32 = I32F32::lit("0.5");
pub const MASK_HORNETS_DPS: I32F32 = I32F32::lit("10");
pub const MASK_HORNETS_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.6");
pub const MASK_HORNETS_STUN_GRENADE: &str = "3 seconds";
pub const MASK_HORNETS_SHOCK_RESPONSE: &str = "Susceptible";
pub const MASK_HORNETS_RADAR_PIP_SIZE: &str = "Tiny";
pub const MASK_HORNETS_SHOVEL_HP: &str = "Immune";
pub const MASK_HORNETS_CAN_SEE_THROUGH_FOG: bool = false;
pub const MASK_HORNETS_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0.08");
pub const MASK_HORNETS_CONTACT_DAMAGE: I32F32 = I32F32::lit("10");
pub const MASK_HORNETS_LEAVE_TIME: &str = "until the ship leaves the moon";

pub const MASK_HORNETS_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");
pub const MASK_HORNETS_ROAM_SPEED: I32F32 = I32F32::lit("2");
pub const MASK_HORNETS_CHASE_SPEED: I32F32 = I32F32::lit("4");
pub const MASK_HORNETS_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const MASK_HORNETS_WATCH_RANGE: I32F32 = I32F32::lit("18");
pub const MASK_HORNETS_BUTLER_ID: &str = "butler";
pub const MASK_HORNETS_CIRCUIT_BEE_ID: &str = "circuit_bee";
pub const MASK_HORNETS_EMPLOYEE_ID: &str = "employee";
pub const MASK_HORNETS_SHIP_ID: &str = "the_ship";
pub const MASK_HORNETS_MOONS_ID: &str = "moons";

pub const MASK_HORNETS_DEPENDS_ON: [&str; 5] = [
    "butler",
    "circuit_bee",
    "employee",
    "the_ship",
    "moons",
];
pub const MASK_HORNETS_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Roaming", "Chasing"];

pub const MASK_HORNETS_BEHAVIORAL_MECHANICS: [MaskHornetsBehaviorRule; 6] = [
    MaskHornetsBehaviorRule {
        condition: "a butler dies",
        outcome: "Mask Hornets spawn",
    },
    MaskHornetsBehaviorRule {
        condition: "Mask Hornets are active",
        outcome: "they roam and chase employees",
    },
    MaskHornetsBehaviorRule {
        condition: "Mask Hornets are active",
        outcome: "they remain present until the ship leaves the moon",
    },
    MaskHornetsBehaviorRule {
        condition: "Mask Hornets are active",
        outcome: "they cannot go outside",
    },
    MaskHornetsBehaviorRule {
        condition: "Mask Hornets are spawned",
        outcome: "they are invincible",
    },
    MaskHornetsBehaviorRule {
        condition: "Mask Hornets are spawned through a butler death",
        outcome: "they do not use the normal entity spawn cycle",
    },
];

pub struct MaskHornetsPlugin;

impl Plugin for MaskHornetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnMaskHornetsEvent>()
            .add_event::<MaskHornetsButlerDeathEvent>()
            .add_event::<MaskHornetsStateChangedEvent>()
            .add_event::<MaskHornetsDoorAttemptEvent>()
            .add_event::<MaskHornetsDoorAttemptResolvedEvent>()
            .add_event::<MaskHornetsStunAppliedEvent>()
            .add_event::<MaskHornetsStunAdjustedEvent>()
            .add_event::<MaskHornetsZapGunTargetedEvent>()
            .add_event::<MaskHornetsZapGunSusceptibleEvent>()
            .add_event::<MaskHornetsContactDamageEvent>()
            .add_event::<MaskHornetsIgnoredDamageEvent>()
            .add_event::<MaskHornetsShipLeftMoonEvent>()
            .add_event::<MaskHornetsDespawnedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    mask_hornets_spawn_from_butler_death,
                    spawn_mask_hornets,
                    mask_hornets_roam,
                    mask_hornets_chase_employees,
                    mask_hornets_contact_damage,
                    mask_hornets_door_attempt_speed,
                    mask_hornets_apply_stun_multiplier,
                    mask_hornets_report_zap_gun_susceptible,
                    mask_hornets_ignore_damage,
                    mask_hornets_remain_until_ship_leaves_moon,
                    mask_hornets_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaskHornets;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaskHornetsEmployeeSensor {
    pub stable_id: u64,
    pub can_be_chased: bool,
    pub touching_hornets: bool,
    pub is_inside_facility: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsRoamRoute {
    pub area_a: SimPosition,
    pub area_b: SimPosition,
    pub destination_index: u8,
}

impl Default for MaskHornetsRoamRoute {
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
pub struct MaskHornetsTarget {
    pub has_target: bool,
    pub target_stable_id: u64,
    pub target_position: SimPosition,
}

impl Default for MaskHornetsTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_stable_id: 0,
            target_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaskHornetsIndoorOnly {
    pub is_inside_facility: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaskHornetsState {
    #[default]
    Roaming,
    Chasing,
}

#[derive(Bundle)]
pub struct MaskHornetsBundle {
    pub name: Name,
    pub mask_hornets: MaskHornets,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: MaskHornetsState,
    pub route: MaskHornetsRoamRoute,
    pub target: MaskHornetsTarget,
    pub indoor_only: MaskHornetsIndoorOnly,
}

impl MaskHornetsBundle {
    pub fn new(event: SpawnMaskHornetsEvent) -> Self {
        Self {
            name: Name::new(MASK_HORNETS_NAME),
            mask_hornets: MaskHornets,
            position: event.position,
            health: Health::full(MASK_HORNETS_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: MASK_HORNETS_ROAM_SPEED,
                attack_range: MASK_HORNETS_ATTACK_RANGE,
                attack_damage: MASK_HORNETS_CONTACT_DAMAGE,
                attack_speed: MASK_HORNETS_ATTACK_SPEED,
                watch_range: MASK_HORNETS_WATCH_RANGE,
            },
            state: MaskHornetsState::Roaming,
            route: MaskHornetsRoamRoute {
                area_a: event.roam_area_a,
                area_b: event.roam_area_b,
                destination_index: 1,
            },
            target: MaskHornetsTarget::default(),
            indoor_only: MaskHornetsIndoorOnly {
                is_inside_facility: true,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnMaskHornetsEvent {
    pub position: SimPosition,
    pub roam_area_a: SimPosition,
    pub roam_area_b: SimPosition,
    pub source_butler: Entity,
    pub source_butler_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsButlerDeathEvent {
    pub butler: Entity,
    pub butler_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsStateChangedEvent {
    pub hornets: Entity,
    pub from: MaskHornetsState,
    pub to: MaskHornetsState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsDoorAttemptEvent {
    pub hornets: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsDoorAttemptResolvedEvent {
    pub hornets: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsStunAppliedEvent {
    pub hornets: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsStunAdjustedEvent {
    pub hornets: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsZapGunTargetedEvent {
    pub hornets: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsZapGunSusceptibleEvent {
    pub hornets: Entity,
    pub susceptible: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsContactDamageEvent {
    pub hornets: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsIgnoredDamageEvent {
    pub hornets: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsShipLeftMoonEvent;

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskHornetsDespawnedEvent {
    pub hornets: Entity,
}

fn mask_hornets_spawn_from_butler_death(
    mut deaths: EventReader<MaskHornetsButlerDeathEvent>,
    mut spawns: EventWriter<SpawnMaskHornetsEvent>,
) {
    for event in deaths.read() {
        spawns.send(SpawnMaskHornetsEvent {
            position: event.position,
            roam_area_a: event.position,
            roam_area_b: event.position,
            source_butler: event.butler,
            source_butler_stable_id: event.butler_stable_id,
        });
    }
}

fn spawn_mask_hornets(
    mut commands: Commands,
    mut events: EventReader<SpawnMaskHornetsEvent>,
    hornets: Query<(), With<MaskHornets>>,
) {
    let mut spawned_count = hornets.iter().count();

    for event in events.read() {
        if spawned_count >= MASK_HORNETS_MAX_SPAWNED {
            break;
        }

        commands.spawn(MaskHornetsBundle::new(*event));
        spawned_count += 1;
    }
}

fn mask_hornets_roam(
    sim_hz: Res<SimHz>,
    mut hornets: Query<
        (
            &mut SimPosition,
            &MaskHornetsState,
            &UnitStats,
            &mut MaskHornetsRoamRoute,
            &MaskHornetsIndoorOnly,
        ),
        With<MaskHornets>,
    >,
) {
    for (mut position, state, stats, mut route, indoor_only) in hornets.iter_mut() {
        if *state != MaskHornetsState::Roaming || !indoor_only.is_inside_facility {
            continue;
        }

        let destination = if route.destination_index == 0 {
            route.area_a
        } else {
            route.area_b
        };

        move_axis_toward(&mut position, destination, stats.move_speed / sim_hz.0);

        if *position == destination {
            route.destination_index = 1 - route.destination_index;
        }
    }
}

fn mask_hornets_chase_employees(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<MaskHornetsStateChangedEvent>,
    mut hornets: Query<
        (
            Entity,
            &mut SimPosition,
            &mut UnitStats,
            &mut MaskHornetsState,
            &mut MaskHornetsTarget,
            &MaskHornetsIndoorOnly,
        ),
        With<MaskHornets>,
    >,
    employees: Query<(&SimPosition, &MaskHornetsEmployeeSensor), Without<MaskHornets>>,
) {
    for (hornets_entity, mut position, mut stats, mut state, mut target, indoor_only) in
        hornets.iter_mut()
    {
        if !indoor_only.is_inside_facility {
            continue;
        }

        let nearest_employee = first_chaseable_employee(&employees);

        let Some((employee_position, employee_sensor)) = nearest_employee else {
            target.has_target = false;
            target.target_stable_id = 0;
            stats.move_speed = MASK_HORNETS_ROAM_SPEED;
            set_mask_hornets_state(
                hornets_entity,
                &mut state,
                MaskHornetsState::Roaming,
                &mut state_events,
            );
            continue;
        };

        target.has_target = true;
        target.target_stable_id = employee_sensor.stable_id;
        target.target_position = employee_position;
        stats.move_speed = MASK_HORNETS_CHASE_SPEED;

        set_mask_hornets_state(
            hornets_entity,
            &mut state,
            MaskHornetsState::Chasing,
            &mut state_events,
        );
        move_axis_toward(&mut position, employee_position, stats.move_speed / sim_hz.0);
    }
}

fn mask_hornets_contact_damage(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<MaskHornetsContactDamageEvent>,
    hornets: Query<Entity, With<MaskHornets>>,
    employees: Query<(Entity, &MaskHornetsEmployeeSensor), Without<MaskHornets>>,
) {
    for hornets_entity in hornets.iter() {
        for (employee_entity, employee_sensor) in employees.iter() {
            if !employee_sensor.touching_hornets || !employee_sensor.is_inside_facility {
                continue;
            }

            contact_events.send(MaskHornetsContactDamageEvent {
                hornets: hornets_entity,
                employee: employee_entity,
                employee_stable_id: employee_sensor.stable_id,
                damage: MASK_HORNETS_CONTACT_DAMAGE,
            });
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: MASK_HORNETS_CONTACT_DAMAGE,
                damage_type: DamageType::Standard,
                source: hornets_entity,
            });
        }
    }
}

fn mask_hornets_door_attempt_speed(
    mut events: EventReader<MaskHornetsDoorAttemptEvent>,
    mut resolved_events: EventWriter<MaskHornetsDoorAttemptResolvedEvent>,
    hornets: Query<(), With<MaskHornets>>,
) {
    for event in events.read() {
        if hornets.get(event.hornets).is_err() {
            continue;
        }

        resolved_events.send(MaskHornetsDoorAttemptResolvedEvent {
            hornets: event.hornets,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(
                event.base_open_ticks,
                MASK_HORNETS_DOOR_OPEN_SPEED,
            ),
        });
    }
}

fn mask_hornets_apply_stun_multiplier(
    mut events: EventReader<MaskHornetsStunAppliedEvent>,
    mut adjusted_events: EventWriter<MaskHornetsStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(MaskHornetsStunAdjustedEvent {
            hornets: event.hornets,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, MASK_HORNETS_STUN_MULTIPLIER),
        });
    }
}

fn mask_hornets_report_zap_gun_susceptible(
    mut events: EventReader<MaskHornetsZapGunTargetedEvent>,
    mut susceptible_events: EventWriter<MaskHornetsZapGunSusceptibleEvent>,
    hornets: Query<(), With<MaskHornets>>,
) {
    for event in events.read() {
        if hornets.get(event.hornets).is_err() {
            continue;
        }

        susceptible_events.send(MaskHornetsZapGunSusceptibleEvent {
            hornets: event.hornets,
            susceptible: true,
        });
    }
}

fn mask_hornets_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<MaskHornetsIgnoredDamageEvent>,
    hornets: Query<(), With<MaskHornets>>,
) {
    for event in damage_events.read() {
        if hornets.get(event.target).is_err() {
            continue;
        }

        ignored_events.send(MaskHornetsIgnoredDamageEvent {
            hornets: event.target,
            source: event.source,
        });
    }
}

fn mask_hornets_remain_until_ship_leaves_moon(
    mut commands: Commands,
    mut events: EventReader<MaskHornetsShipLeftMoonEvent>,
    mut despawned_events: EventWriter<MaskHornetsDespawnedEvent>,
    hornets: Query<Entity, With<MaskHornets>>,
) {
    if events.read().next().is_none() {
        return;
    }

    for hornets_entity in hornets.iter() {
        commands.entity(hornets_entity).despawn();
        despawned_events.send(MaskHornetsDespawnedEvent {
            hornets: hornets_entity,
        });
    }
}

fn mask_hornets_checksum(
    mut checksum: ResMut<SimChecksumState>,
    hornets: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &MaskHornetsState,
            &MaskHornetsRoamRoute,
            &MaskHornetsTarget,
            &MaskHornetsIndoorOnly,
        ),
        With<MaskHornets>,
    >,
) {
    for (position, health, stats, state, route, target, indoor_only) in hornets.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(mask_hornets_state_bits(*state));
        checksum.accumulate(route.area_a.x.to_bits() as u64);
        checksum.accumulate(route.area_a.y.to_bits() as u64);
        checksum.accumulate(route.area_b.x.to_bits() as u64);
        checksum.accumulate(route.area_b.y.to_bits() as u64);
        checksum.accumulate(route.destination_index as u64);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.target_position.x.to_bits() as u64);
        checksum.accumulate(target.target_position.y.to_bits() as u64);
        checksum.accumulate(indoor_only.is_inside_facility as u64);
    }
}

fn first_chaseable_employee(
    employees: &Query<(&SimPosition, &MaskHornetsEmployeeSensor), Without<MaskHornets>>,
) -> Option<(SimPosition, MaskHornetsEmployeeSensor)> {
    let mut best: Option<(SimPosition, MaskHornetsEmployeeSensor)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.can_be_chased || !sensor.is_inside_facility {
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

fn set_mask_hornets_state(
    hornets: Entity,
    state: &mut MaskHornetsState,
    next: MaskHornetsState,
    events: &mut EventWriter<MaskHornetsStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(MaskHornetsStateChangedEvent {
        hornets,
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

fn mask_hornets_state_bits(state: MaskHornetsState) -> u64 {
    match state {
        MaskHornetsState::Roaming => 0,
        MaskHornetsState::Chasing => 1,
    }
}