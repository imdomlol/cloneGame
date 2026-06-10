// Sources: vault/indoor_entity_pages/spore_lizard.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const SPORE_LIZARD_ID: &str = "spore_lizard";
pub const SPORE_LIZARD_NAME: &str = "Spore Lizard";
pub const SPORE_LIZARD_TYPE: &str = "indoor_entity_pages";
pub const SPORE_LIZARD_SUBTYPE: &str = "entity";
pub const SPORE_LIZARD_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Spore_Lizard";
pub const SPORE_LIZARD_SOURCE_REVISION: u32 = 20527;
pub const SPORE_LIZARD_EXTRACTED_AT: &str = "2026-06-07";
pub const SPORE_LIZARD_CONFIDENCE_BASIS_POINTS: u16 = 91;

pub const SPORE_LIZARD_DWELLS: &str = "Inside";
pub const SPORE_LIZARD_DANGER: &str = "5%";
pub const SPORE_LIZARD_SCIENTIFIC_NAME: &str = "Lacerta-glomerorum";
pub const SPORE_LIZARD_HP: &str = "Immune";
pub const SPORE_LIZARD_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const SPORE_LIZARD_MAX_SPAWNED: usize = 2;
pub const SPORE_LIZARD_ATTACK_SPEED: I32F32 = I32F32::lit("1");
pub const SPORE_LIZARD_DPS: I32F32 = I32F32::lit("20");
pub const SPORE_LIZARD_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.6");
pub const SPORE_LIZARD_STUN_GRENADE: bool = false;
pub const SPORE_LIZARD_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.3");
pub const SPORE_LIZARD_ZAP_GUN_DIFFICULTY: &str = "?";
pub const SPORE_LIZARD_INTERNAL_NAME: &str = "Puffer";
pub const SPORE_LIZARD_PIP_SIZE: &str = "Medium-Large";
pub const SPORE_LIZARD_CONTACT_DAMAGE: I32F32 = I32F32::lit("20");
pub const SPORE_LIZARD_LEAVE_TIME: &str = "Doesn't leave";

pub const SPORE_LIZARD_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");
pub const SPORE_LIZARD_BACK_AWAY_SPEED: I32F32 = I32F32::lit("1");
pub const SPORE_LIZARD_DEFENSIVE_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const SPORE_LIZARD_WATCH_RANGE: I32F32 = I32F32::lit("12");
pub const SPORE_LIZARD_CORNER_ATTACK_ROLL_MODULUS: u32 = 2;
pub const SPORE_LIZARD_CORNER_ATTACK_ROLL_SUCCESS: u32 = 0;
pub const SPORE_LIZARD_CORNER_ATTACK_SALT: u64 = 0x7370_6f72_655f_6174;

pub const SPORE_LIZARD_DEPENDS_ON: [&str; 1] = ["lethal_company"];
pub const SPORE_LIZARD_FRONTMATTER_BEHAVIOR: [&str; 1] = ["Territorial"];

pub const SPORE_LIZARD_BEHAVIORAL_MECHANICS: [SporeLizardBehaviorRule; 7] = [
    SporeLizardBehaviorRule {
        condition: "a player approaches the Spore Lizard",
        outcome: "it becomes frightened and backs away from the player",
    },
    SporeLizardBehaviorRule {
        condition: "the Spore Lizard is cornered",
        outcome: "it releases pink spores from its tail and creates foggy visibility around itself",
    },
    SporeLizardBehaviorRule {
        condition: "the Spore Lizard is cornered",
        outcome: "it may begin attacking with 20 contact damage",
    },
    SporeLizardBehaviorRule {
        condition: "the Spore Lizard is threatened or pursued",
        outcome: "it remains defensive rather than actively hunting",
    },
    SporeLizardBehaviorRule {
        condition: "the Spore Lizard is affected by stun interactions",
        outcome: "its stun duration is multiplied by 0.6",
    },
    SporeLizardBehaviorRule {
        condition: "the Spore Lizard interacts with doors",
        outcome: "its door movement speed is multiplied by 0.3",
    },
    SporeLizardBehaviorRule {
        condition: "the Spore Lizard is present in a room",
        outcome: "it does not leave on its own",
    },
];

pub struct SporeLizardPlugin;

impl Plugin for SporeLizardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSporeLizardEvent>()
            .add_event::<SporeLizardStateChangedEvent>()
            .add_event::<SporeLizardSporeCloudEvent>()
            .add_event::<SporeLizardDefensiveAttackRollEvent>()
            .add_event::<SporeLizardContactDamageEvent>()
            .add_event::<SporeLizardDoorAttemptEvent>()
            .add_event::<SporeLizardDoorAttemptResolvedEvent>()
            .add_event::<SporeLizardStunAppliedEvent>()
            .add_event::<SporeLizardStunAdjustedEvent>()
            .add_event::<SporeLizardZapGunTargetedEvent>()
            .add_event::<SporeLizardZapGunDifficultyEvent>()
            .add_event::<SporeLizardIgnoredDamageEvent>()
            .add_event::<SporeLizardRoomHoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_spore_lizard,
                    spore_lizard_frightened_back_away,
                    spore_lizard_cornered_spores,
                    spore_lizard_cornered_contact_damage,
                    spore_lizard_threatened_defensive,
                    spore_lizard_door_attempt_speed,
                    spore_lizard_apply_stun_multiplier,
                    spore_lizard_report_zap_gun_difficulty,
                    spore_lizard_ignore_damage,
                    spore_lizard_hold_room,
                    spore_lizard_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SporeLizard {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SporeLizardEmployeeSensor {
    pub stable_id: u64,
    pub approaching_spore_lizard: bool,
    pub cornering_spore_lizard: bool,
    pub threatening_or_pursuing_spore_lizard: bool,
    pub touching_spore_lizard: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardDefense {
    pub target_stable_id: u64,
    pub has_target: bool,
    pub spore_cloud_active: bool,
    pub foggy_visibility_active: bool,
    pub attacking: bool,
}

impl Default for SporeLizardDefense {
    fn default() -> Self {
        Self {
            target_stable_id: 0,
            has_target: false,
            spore_cloud_active: false,
            foggy_visibility_active: false,
            attacking: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardRoomAnchor {
    pub room_id: u64,
    pub anchor_position: SimPosition,
}

impl Default for SporeLizardRoomAnchor {
    fn default() -> Self {
        Self {
            room_id: 0,
            anchor_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SporeLizardState {
    #[default]
    Calm,
    Frightened,
    Defensive,
}

#[derive(Bundle)]
pub struct SporeLizardBundle {
    pub name: Name,
    pub spore_lizard: SporeLizard,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: SporeLizardState,
    pub defense: SporeLizardDefense,
    pub room_anchor: SporeLizardRoomAnchor,
}

impl SporeLizardBundle {
    pub fn new(event: SpawnSporeLizardEvent) -> Self {
        Self {
            name: Name::new(SPORE_LIZARD_NAME),
            spore_lizard: SporeLizard {
                stable_id: event.stable_id,
            },
            position: event.position,
            health: Health::full(SPORE_LIZARD_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: SPORE_LIZARD_BACK_AWAY_SPEED,
                attack_range: SPORE_LIZARD_DEFENSIVE_ATTACK_RANGE,
                attack_damage: SPORE_LIZARD_CONTACT_DAMAGE,
                attack_speed: SPORE_LIZARD_ATTACK_SPEED,
                watch_range: SPORE_LIZARD_WATCH_RANGE,
            },
            state: SporeLizardState::Calm,
            defense: SporeLizardDefense::default(),
            room_anchor: SporeLizardRoomAnchor {
                room_id: event.room_id,
                anchor_position: event.position,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnSporeLizardEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub room_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardStateChangedEvent {
    pub spore_lizard: Entity,
    pub from: SporeLizardState,
    pub to: SporeLizardState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardSporeCloudEvent {
    pub spore_lizard: Entity,
    pub target_stable_id: u64,
    pub foggy_visibility_active: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardDefensiveAttackRollEvent {
    pub spore_lizard: Entity,
    pub target_stable_id: u64,
    pub roll: u32,
    pub attacking: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardContactDamageEvent {
    pub spore_lizard: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardDoorAttemptEvent {
    pub spore_lizard: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardDoorAttemptResolvedEvent {
    pub spore_lizard: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardStunAppliedEvent {
    pub spore_lizard: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardStunAdjustedEvent {
    pub spore_lizard: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
    pub stun_grenade_applies: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardZapGunTargetedEvent {
    pub spore_lizard: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardZapGunDifficultyEvent {
    pub spore_lizard: Entity,
    pub difficulty_modifier: &'static str,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardIgnoredDamageEvent {
    pub spore_lizard: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SporeLizardRoomHoldEvent {
    pub spore_lizard: Entity,
    pub room_id: u64,
    pub anchor_position: SimPosition,
}

fn spawn_spore_lizard(
    mut commands: Commands,
    mut events: EventReader<SpawnSporeLizardEvent>,
    spore_lizards: Query<(), With<SporeLizard>>,
) {
    let mut spawned_count = spore_lizards.iter().count();

    for event in events.read() {
        if spawned_count >= SPORE_LIZARD_MAX_SPAWNED {
            break;
        }

        commands.spawn(SporeLizardBundle::new(*event));
        spawned_count += 1;
    }
}

fn spore_lizard_frightened_back_away(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<SporeLizardStateChangedEvent>,
    mut spore_lizards: Query<
        (
            Entity,
            &mut SimPosition,
            &mut SporeLizardState,
            &mut SporeLizardDefense,
            &UnitStats,
        ),
        With<SporeLizard>,
    >,
    employees: Query<(&SimPosition, &SporeLizardEmployeeSensor), Without<SporeLizard>>,
) {
    let Some((employee_position, employee_sensor)) = approaching_employee(&employees) else {
        return;
    };

    for (spore_lizard_entity, mut position, mut state, mut defense, stats) in spore_lizards.iter_mut()
    {
        if *state == SporeLizardState::Defensive {
            continue;
        }

        defense.has_target = true;
        defense.target_stable_id = employee_sensor.stable_id;

        move_axis_away(&mut position, employee_position, stats.move_speed / sim_hz.0);
        set_spore_lizard_state(
            spore_lizard_entity,
            &mut state,
            SporeLizardState::Frightened,
            &mut state_events,
        );
    }
}

fn spore_lizard_cornered_spores(
    mut state_events: EventWriter<SporeLizardStateChangedEvent>,
    mut spore_events: EventWriter<SporeLizardSporeCloudEvent>,
    mut spore_lizards: Query<
        (
            Entity,
            &mut SporeLizardState,
            &mut SporeLizardDefense,
        ),
        With<SporeLizard>,
    >,
    employees: Query<&SporeLizardEmployeeSensor, Without<SporeLizard>>,
) {
    let Some(employee_sensor) = cornering_employee(&employees) else {
        return;
    };

    for (spore_lizard_entity, mut state, mut defense) in spore_lizards.iter_mut() {
        defense.has_target = true;
        defense.target_stable_id = employee_sensor.stable_id;
        defense.spore_cloud_active = true;
        defense.foggy_visibility_active = true;

        spore_events.send(SporeLizardSporeCloudEvent {
            spore_lizard: spore_lizard_entity,
            target_stable_id: employee_sensor.stable_id,
            foggy_visibility_active: true,
        });

        set_spore_lizard_state(
            spore_lizard_entity,
            &mut state,
            SporeLizardState::Defensive,
            &mut state_events,
        );
    }
}

fn spore_lizard_cornered_contact_damage(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    mut roll_events: EventWriter<SporeLizardDefensiveAttackRollEvent>,
    mut contact_events: EventWriter<SporeLizardContactDamageEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut spore_lizards: Query<
        (
            Entity,
            &SporeLizard,
            &SporeLizardState,
            &mut SporeLizardDefense,
            &UnitStats,
        ),
        With<SporeLizard>,
    >,
    employees: Query<(Entity, &SporeLizardEmployeeSensor), Without<SporeLizard>>,
) {
    for (spore_lizard_entity, spore_lizard, state, mut defense, stats) in spore_lizards.iter_mut() {
        if *state != SporeLizardState::Defensive {
            continue;
        }

        let Some((employee_entity, employee_sensor)) =
            touching_cornering_employee(defense.target_stable_id, &employees)
        else {
            defense.attacking = false;
            continue;
        };

        let mut rng = tick_rng(
            game_seed.0,
            sim_tick.0,
            SPORE_LIZARD_CORNER_ATTACK_SALT ^ spore_lizard.stable_id,
        );
        let roll = rng.next_u32() % SPORE_LIZARD_CORNER_ATTACK_ROLL_MODULUS;
        let attacking = roll == SPORE_LIZARD_CORNER_ATTACK_ROLL_SUCCESS;
        defense.attacking = attacking;

        roll_events.send(SporeLizardDefensiveAttackRollEvent {
            spore_lizard: spore_lizard_entity,
            target_stable_id: employee_sensor.stable_id,
            roll,
            attacking,
        });

        if !attacking {
            continue;
        }

        contact_events.send(SporeLizardContactDamageEvent {
            spore_lizard: spore_lizard_entity,
            employee: employee_entity,
            employee_stable_id: employee_sensor.stable_id,
            damage: stats.attack_damage,
        });
        damage_events.send(IncomingDamageEvent {
            target: employee_entity,
            raw_amount: stats.attack_damage,
            damage_type: DamageType::Standard,
            source: spore_lizard_entity,
        });
    }
}

fn spore_lizard_threatened_defensive(
    mut state_events: EventWriter<SporeLizardStateChangedEvent>,
    mut spore_lizards: Query<
        (Entity, &mut SporeLizardState, &mut SporeLizardDefense),
        With<SporeLizard>,
    >,
    employees: Query<&SporeLizardEmployeeSensor, Without<SporeLizard>>,
) {
    let Some(employee_sensor) = threatening_employee(&employees) else {
        return;
    };

    for (spore_lizard_entity, mut state, mut defense) in spore_lizards.iter_mut() {
        defense.has_target = true;
        defense.target_stable_id = employee_sensor.stable_id;
        defense.attacking = false;

        set_spore_lizard_state(
            spore_lizard_entity,
            &mut state,
            SporeLizardState::Defensive,
            &mut state_events,
        );
    }
}

fn spore_lizard_door_attempt_speed(
    mut events: EventReader<SporeLizardDoorAttemptEvent>,
    mut resolved_events: EventWriter<SporeLizardDoorAttemptResolvedEvent>,
    spore_lizards: Query<(), With<SporeLizard>>,
) {
    for event in events.read() {
        if spore_lizards.get(event.spore_lizard).is_err() {
            continue;
        }

        resolved_events.send(SporeLizardDoorAttemptResolvedEvent {
            spore_lizard: event.spore_lizard,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(
                event.base_open_ticks,
                SPORE_LIZARD_DOOR_SPEED_MULTIPLIER,
            ),
        });
    }
}

fn spore_lizard_apply_stun_multiplier(
    mut events: EventReader<SporeLizardStunAppliedEvent>,
    mut adjusted_events: EventWriter<SporeLizardStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(SporeLizardStunAdjustedEvent {
            spore_lizard: event.spore_lizard,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, SPORE_LIZARD_STUN_MULTIPLIER),
            stun_grenade_applies: SPORE_LIZARD_STUN_GRENADE,
        });
    }
}

fn spore_lizard_report_zap_gun_difficulty(
    mut events: EventReader<SporeLizardZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<SporeLizardZapGunDifficultyEvent>,
    spore_lizards: Query<(), With<SporeLizard>>,
) {
    for event in events.read() {
        if spore_lizards.get(event.spore_lizard).is_err() {
            continue;
        }

        difficulty_events.send(SporeLizardZapGunDifficultyEvent {
            spore_lizard: event.spore_lizard,
            difficulty_modifier: SPORE_LIZARD_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn spore_lizard_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<SporeLizardIgnoredDamageEvent>,
    spore_lizards: Query<(), With<SporeLizard>>,
) {
    for event in damage_events.read() {
        if spore_lizards.get(event.target).is_err() {
            continue;
        }

        ignored_events.send(SporeLizardIgnoredDamageEvent {
            spore_lizard: event.target,
            source: event.source,
        });
    }
}

fn spore_lizard_hold_room(
    mut events: EventWriter<SporeLizardRoomHoldEvent>,
    spore_lizards: Query<(Entity, &SporeLizardRoomAnchor), With<SporeLizard>>,
) {
    for (spore_lizard_entity, anchor) in spore_lizards.iter() {
        events.send(SporeLizardRoomHoldEvent {
            spore_lizard: spore_lizard_entity,
            room_id: anchor.room_id,
            anchor_position: anchor.anchor_position,
        });
    }
}

fn spore_lizard_checksum(
    mut checksum: ResMut<SimChecksumState>,
    spore_lizards: Query<
        (
            &SporeLizard,
            &SimPosition,
            &Health,
            &UnitStats,
            &SporeLizardState,
            &SporeLizardDefense,
            &SporeLizardRoomAnchor,
        ),
        With<SporeLizard>,
    >,
) {
    for (spore_lizard, position, health, stats, state, defense, anchor) in spore_lizards.iter() {
        checksum.accumulate(spore_lizard.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(spore_lizard_state_bits(*state));
        checksum.accumulate(defense.target_stable_id);
        checksum.accumulate(defense.has_target as u64);
        checksum.accumulate(defense.spore_cloud_active as u64);
        checksum.accumulate(defense.foggy_visibility_active as u64);
        checksum.accumulate(defense.attacking as u64);
        checksum.accumulate(anchor.room_id);
        checksum.accumulate(anchor.anchor_position.x.to_bits() as u64);
        checksum.accumulate(anchor.anchor_position.y.to_bits() as u64);
    }
}

fn approaching_employee(
    employees: &Query<(&SimPosition, &SporeLizardEmployeeSensor), Without<SporeLizard>>,
) -> Option<(SimPosition, SporeLizardEmployeeSensor)> {
    let mut best: Option<(SimPosition, SporeLizardEmployeeSensor)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.approaching_spore_lizard {
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

fn cornering_employee(
    employees: &Query<&SporeLizardEmployeeSensor, Without<SporeLizard>>,
) -> Option<SporeLizardEmployeeSensor> {
    let mut best: Option<SporeLizardEmployeeSensor> = None;

    for sensor in employees.iter() {
        if !sensor.cornering_spore_lizard {
            continue;
        }

        if let Some(best_sensor) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some(*sensor);
    }

    best
}

fn threatening_employee(
    employees: &Query<&SporeLizardEmployeeSensor, Without<SporeLizard>>,
) -> Option<SporeLizardEmployeeSensor> {
    let mut best: Option<SporeLizardEmployeeSensor> = None;

    for sensor in employees.iter() {
        if !sensor.threatening_or_pursuing_spore_lizard {
            continue;
        }

        if let Some(best_sensor) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some(*sensor);
    }

    best
}

fn touching_cornering_employee(
    target_stable_id: u64,
    employees: &Query<(Entity, &SporeLizardEmployeeSensor), Without<SporeLizard>>,
) -> Option<(Entity, SporeLizardEmployeeSensor)> {
    let mut best: Option<(Entity, SporeLizardEmployeeSensor)> = None;

    for (employee_entity, sensor) in employees.iter() {
        if !sensor.cornering_spore_lizard || !sensor.touching_spore_lizard {
            continue;
        }

        if target_stable_id != 0 && sensor.stable_id != target_stable_id {
            continue;
        }

        if let Some((_best_entity, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((employee_entity, *sensor));
    }

    best
}

fn set_spore_lizard_state(
    spore_lizard: Entity,
    state: &mut SporeLizardState,
    next: SporeLizardState,
    events: &mut EventWriter<SporeLizardStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(SporeLizardStateChangedEvent {
        spore_lizard,
        from: previous,
        to: next,
    });
}

fn move_axis_away(position: &mut SimPosition, source: SimPosition, max_step: I32F32) {
    let dx = position.x - source.x;
    let dy = position.y - source.y;

    if fixed_abs(dx) >= fixed_abs(dy) {
        if dx < I32F32::lit("0") {
            position.x -= max_step;
        } else {
            position.x += max_step;
        }
    } else if dy < I32F32::lit("0") {
        position.y -= max_step;
    } else {
        position.y += max_step;
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

fn spore_lizard_state_bits(state: SporeLizardState) -> u64 {
    match state {
        SporeLizardState::Calm => 0,
        SporeLizardState::Frightened => 1,
        SporeLizardState::Defensive => 2,
    }
}