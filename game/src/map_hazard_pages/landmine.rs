// Sources: vault/map_hazard_pages/landmine.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, IncomingDamageEvent, SimChecksumState, SimPosition, SimTick,
};

pub const LANDMINE_ID: &str = "landmine";
pub const LANDMINE_NAME: &str = "Landmine";
pub const LANDMINE_TYPE: &str = "map_hazard_pages";
pub const LANDMINE_SUBTYPE: &str = "map_hazard";
pub const LANDMINE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Landmine";
pub const LANDMINE_SOURCE_REVISION: u32 = 20471;
pub const LANDMINE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const LANDMINE_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const LANDMINE_DAMAGE_MIN: I32F32 = I32F32::lit("50");
pub const LANDMINE_DAMAGE_MAX: I32F32 = I32F32::lit("100");
pub const LANDMINE_DAMAGE_TYPE_LABEL: &str = "area_of_effect";
pub const LANDMINE_KILL_RADIUS: I32F32 = I32F32::lit("5.7");
pub const LANDMINE_INJURY_RADIUS_MIN: I32F32 = I32F32::lit("5.7");
pub const LANDMINE_INJURY_RADIUS_MAX: I32F32 = I32F32::lit("6.4");
pub const LANDMINE_CHAIN_REACTION_RADIUS: I32F32 = I32F32::lit("6");

pub const LANDMINE_DEPENDS_ON: [&str; 10] = [
    "shovel",
    "terminal",
    "ship_duty",
    "extension_ladder",
    "stun_grenade",
    "soccer_ball",
    "easter_egg",
    "player_body",
    "hoarding_bug",
    "bracken",
];

pub const LANDMINE_OCCURRENCES: [LandmineOccurrence; 11] = [
    LandmineOccurrence::new("41_experimentation", 0, 12, 2),
    LandmineOccurrence::new("220_assurance", 0, 7, 3),
    LandmineOccurrence::new("56_vow", 0, 20, 4),
    LandmineOccurrence::new("21_offense", 0, 16, 5),
    LandmineOccurrence::new("61_march", 0, 15, 6),
    LandmineOccurrence::new("20_adamance", 0, 15, 3),
    LandmineOccurrence::new("85_rend", 0, 3, 1),
    LandmineOccurrence::new("7_dine", 0, 35, 8),
    LandmineOccurrence::new("8_titan", 0, 23, 9),
    LandmineOccurrence::new("5_embrion", 0, 16, 5),
    LandmineOccurrence::new("68_artifice", 0, 35, 10),
];

pub const LANDMINE_BEHAVIORAL_MECHANICS: [LandmineBehaviorRule; 12] = [
    LandmineBehaviorRule::new(
        "an employee steps on and off the mine",
        "it detonates",
    ),
    LandmineBehaviorRule::new(
        "an employee teleports off the mine",
        "it detonates",
    ),
    LandmineBehaviorRule::new(
        "the mine is struck by a shovel or similar weapon",
        "it detonates",
    ),
    LandmineBehaviorRule::new(
        "a scrap or other item lands on the mine",
        "it may detonate inconsistently",
    ),
    LandmineBehaviorRule::new(
        "the mine detonates",
        "employees within 5.7 units die immediately",
    ),
    LandmineBehaviorRule::new(
        "the mine detonates",
        "employees beyond 5.7 units and up to 6.4 units away take 50 damage",
    ),
    LandmineBehaviorRule::new(
        "the mine detonates",
        "any other landmine within 6 units can also detonate",
    ),
    LandmineBehaviorRule::new(
        "the mine is disarmed through the ship's terminal",
        "it is only temporarily disabled",
    ),
    LandmineBehaviorRule::new(
        "an employee stays on the mine while a teammate on ship_duty disarms it or teleports them back to the ship",
        "the employee can remain safe",
    ),
    LandmineBehaviorRule::new(
        "a mine is passed through after its brief disarm window",
        "it can still explode",
    ),
    LandmineBehaviorRule::new(
        "a player uses an extension_ladder, stun_grenade, soccer_ball, or easter_egg from a distance",
        "the mine can be detonated safely",
    ),
    LandmineBehaviorRule::new(
        "a carried player_body is held by a hoarding_bug or bracken",
        "it can trigger the mine",
    ),
];

const LANDMINE_ITEM_INCONSISTENT_SALT: u64 = 0x6c61_6e64_6d69_6e65;

pub struct LandminePlugin;

impl Plugin for LandminePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnLandmineEvent>()
            .add_event::<LandmineSpawnCountResolvedEvent>()
            .add_event::<LandmineSteppedOnEvent>()
            .add_event::<LandmineSteppedOffEvent>()
            .add_event::<LandmineTeleportedOffEvent>()
            .add_event::<LandmineStruckEvent>()
            .add_event::<LandmineItemLandedEvent>()
            .add_event::<LandmineRemoteDetonationEvent>()
            .add_event::<LandmineDisarmRequestedEvent>()
            .add_event::<LandmineDisarmedEvent>()
            .add_event::<LandmineDetonationRequestedEvent>()
            .add_event::<LandmineDetonatedEvent>()
            .add_event::<LandmineChainReactionEvent>()
            .add_event::<LandmineBlastImmediateKillEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_landmine,
                    resolve_landmine_spawn_count,
                    landmine_record_employee_step_on,
                    landmine_employee_step_off_trigger,
                    landmine_employee_teleport_trigger,
                    landmine_weapon_strike_trigger,
                    landmine_item_landed_trigger,
                    landmine_remote_detonation_trigger,
                    landmine_disarm_from_terminal,
                    landmine_disarm_timer,
                    landmine_apply_detonation_requests,
                    landmine_apply_blast_damage,
                    landmine_chain_reaction,
                    landmine_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LandmineBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

impl LandmineBehaviorRule {
    pub const fn new(condition: &'static str, outcome: &'static str) -> Self {
        Self { condition, outcome }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LandmineOccurrence {
    pub moon: &'static str,
    pub min_landmines: u16,
    pub max_landmines: u16,
    pub average_landmines: u16,
}

impl LandmineOccurrence {
    pub const fn new(
        moon: &'static str,
        min_landmines: u16,
        max_landmines: u16,
        average_landmines: u16,
    ) -> Self {
        Self {
            moon,
            min_landmines,
            max_landmines,
            average_landmines,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Landmine {
    pub stable_id: u64,
    pub armed: bool,
    pub detonated: bool,
    pub stepped_on_by: u64,
    pub temporary_disabled_ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LandmineBlastTarget {
    pub stable_id: u64,
    pub is_employee: bool,
    pub is_player_body: bool,
    pub carried_by_hoarding_bug: bool,
    pub carried_by_bracken: bool,
}

#[derive(Bundle, Clone, Copy, Debug)]
pub struct LandmineBundle {
    pub landmine: Landmine,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnLandmineEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineSpawnCountResolvedEvent {
    pub moon: &'static str,
    pub min_landmines: u16,
    pub max_landmines: u16,
    pub average_landmines: u16,
    pub resolved_landmines: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineSteppedOnEvent {
    pub mine: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineSteppedOffEvent {
    pub mine: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineTeleportedOffEvent {
    pub mine: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineStruckEvent {
    pub mine: Entity,
    pub weapon_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineItemLandedEvent {
    pub mine: Entity,
    pub item_stable_id: u64,
    pub item_id: &'static str,
    pub force_detonation_result: Option<bool>,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineRemoteDetonationEvent {
    pub mine: Entity,
    pub tool_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineDisarmRequestedEvent {
    pub mine: Entity,
    pub source_id: &'static str,
    pub duration_ticks: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineDisarmedEvent {
    pub mine: Entity,
    pub duration_ticks: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineDetonationRequestedEvent {
    pub mine: Entity,
    pub trigger: LandmineTrigger,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineDetonatedEvent {
    pub mine: Entity,
    pub mine_stable_id: u64,
    pub position: SimPosition,
    pub trigger: LandmineTrigger,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineChainReactionEvent {
    pub source_mine: Entity,
    pub chained_mine: Entity,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmineBlastImmediateKillEvent {
    pub mine: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandmineTrigger {
    EmployeeSteppedOff,
    EmployeeTeleportedOff,
    WeaponStrike,
    ItemLanded,
    RemoteTool,
    PlayerBodyCarriedByHoardingBug,
    PlayerBodyCarriedByBracken,
    ChainReaction,
}

fn spawn_landmine(mut commands: Commands, mut events: EventReader<SpawnLandmineEvent>) {
    for event in events.read() {
        commands.spawn(LandmineBundle {
            landmine: Landmine {
                stable_id: event.stable_id,
                armed: true,
                detonated: false,
                stepped_on_by: 0,
                temporary_disabled_ticks_remaining: 0,
            },
            position: event.position,
        });
    }
}

fn resolve_landmine_spawn_count(mut events: EventWriter<LandmineSpawnCountResolvedEvent>) {
    for occurrence in LANDMINE_OCCURRENCES {
        events.send(LandmineSpawnCountResolvedEvent {
            moon: occurrence.moon,
            min_landmines: occurrence.min_landmines,
            max_landmines: occurrence.max_landmines,
            average_landmines: occurrence.average_landmines,
            resolved_landmines: occurrence.average_landmines,
        });
    }
}

fn landmine_record_employee_step_on(
    mut events: EventReader<LandmineSteppedOnEvent>,
    mut mines: Query<&mut Landmine>,
) {
    for event in events.read() {
        if let Ok(mut mine) = mines.get_mut(event.mine) {
            if mine.armed && !mine.detonated {
                mine.stepped_on_by = event.employee_stable_id;
            }
        }
    }
}

fn landmine_employee_step_off_trigger(
    mut events: EventReader<LandmineSteppedOffEvent>,
    mines: Query<&Landmine>,
    mut detonations: EventWriter<LandmineDetonationRequestedEvent>,
) {
    for event in events.read() {
        if let Ok(mine) = mines.get(event.mine) {
            if mine.armed
                && !mine.detonated
                && mine.temporary_disabled_ticks_remaining == 0
                && mine.stepped_on_by == event.employee_stable_id
            {
                detonations.send(LandmineDetonationRequestedEvent {
                    mine: event.mine,
                    trigger: LandmineTrigger::EmployeeSteppedOff,
                });
            }
        }
    }
}

fn landmine_employee_teleport_trigger(
    mut events: EventReader<LandmineTeleportedOffEvent>,
    mines: Query<&Landmine>,
    mut detonations: EventWriter<LandmineDetonationRequestedEvent>,
) {
    for event in events.read() {
        if let Ok(mine) = mines.get(event.mine) {
            if mine.armed
                && !mine.detonated
                && mine.temporary_disabled_ticks_remaining == 0
                && mine.stepped_on_by == event.employee_stable_id
            {
                detonations.send(LandmineDetonationRequestedEvent {
                    mine: event.mine,
                    trigger: LandmineTrigger::EmployeeTeleportedOff,
                });
            }
        }
    }
}

fn landmine_weapon_strike_trigger(
    mut events: EventReader<LandmineStruckEvent>,
    mines: Query<&Landmine>,
    mut detonations: EventWriter<LandmineDetonationRequestedEvent>,
) {
    for event in events.read() {
        if let Ok(mine) = mines.get(event.mine) {
            if mine.armed && !mine.detonated && mine.temporary_disabled_ticks_remaining == 0 {
                detonations.send(LandmineDetonationRequestedEvent {
                    mine: event.mine,
                    trigger: LandmineTrigger::WeaponStrike,
                });
            }
        }
    }
}

fn landmine_item_landed_trigger(
    mut events: EventReader<LandmineItemLandedEvent>,
    mines: Query<&Landmine>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut detonations: EventWriter<LandmineDetonationRequestedEvent>,
) {
    for event in events.read() {
        if let Ok(mine) = mines.get(event.mine) {
            if !mine.armed || mine.detonated || mine.temporary_disabled_ticks_remaining > 0 {
                continue;
            }

            let should_detonate = match event.force_detonation_result {
                Some(value) => value,
                None => {
                    let salt = LANDMINE_ITEM_INCONSISTENT_SALT ^ event.item_stable_id;
                    tick_rng(seed.0, tick.0, salt).next_u32() % 2 == 0
                }
            };

            if should_detonate {
                detonations.send(LandmineDetonationRequestedEvent {
                    mine: event.mine,
                    trigger: LandmineTrigger::ItemLanded,
                });
            }
        }
    }
}

fn landmine_remote_detonation_trigger(
    mut events: EventReader<LandmineRemoteDetonationEvent>,
    mines: Query<&Landmine>,
    mut detonations: EventWriter<LandmineDetonationRequestedEvent>,
) {
    for event in events.read() {
        if let Ok(mine) = mines.get(event.mine) {
            if mine.armed && !mine.detonated {
                detonations.send(LandmineDetonationRequestedEvent {
                    mine: event.mine,
                    trigger: LandmineTrigger::RemoteTool,
                });
            }
        }
    }
}

fn landmine_disarm_from_terminal(
    mut requests: EventReader<LandmineDisarmRequestedEvent>,
    mut mines: Query<&mut Landmine>,
    mut disarmed: EventWriter<LandmineDisarmedEvent>,
) {
    for request in requests.read() {
        if request.source_id != "terminal" {
            continue;
        }

        if let Ok(mut mine) = mines.get_mut(request.mine) {
            if mine.armed && !mine.detonated {
                mine.temporary_disabled_ticks_remaining = request.duration_ticks;
                disarmed.send(LandmineDisarmedEvent {
                    mine: request.mine,
                    duration_ticks: request.duration_ticks,
                });
            }
        }
    }
}

fn landmine_disarm_timer(mut mines: Query<&mut Landmine>) {
    for mut mine in &mut mines {
        if mine.temporary_disabled_ticks_remaining > 0 {
            mine.temporary_disabled_ticks_remaining -= 1;
        }
    }
}

fn landmine_apply_detonation_requests(
    mut requests: EventReader<LandmineDetonationRequestedEvent>,
    mut mines: Query<(&mut Landmine, &SimPosition)>,
    mut detonated: EventWriter<LandmineDetonatedEvent>,
) {
    for request in requests.read() {
        if let Ok((mut mine, position)) = mines.get_mut(request.mine) {
            if mine.armed && !mine.detonated && mine.temporary_disabled_ticks_remaining == 0 {
                mine.detonated = true;
                mine.armed = false;
                detonated.send(LandmineDetonatedEvent {
                    mine: request.mine,
                    mine_stable_id: mine.stable_id,
                    position: *position,
                    trigger: request.trigger,
                });
            }
        }
    }
}

fn landmine_apply_blast_damage(
    mut detonations: EventReader<LandmineDetonatedEvent>,
    targets: Query<(Entity, &LandmineBlastTarget, &SimPosition)>,
    mut damage: EventWriter<IncomingDamageEvent>,
    mut immediate_kills: EventWriter<LandmineBlastImmediateKillEvent>,
) {
    let mut sorted_targets: Vec<(u64, Entity, LandmineBlastTarget, SimPosition)> = targets
        .iter()
        .map(|(entity, target, position)| (target.stable_id, entity, *target, *position))
        .collect();
    sorted_targets.sort_by_key(|(stable_id, _, _, _)| *stable_id);

    for detonation in detonations.read() {
        for (_, entity, target, position) in &sorted_targets {
            let distance_sq = distance_squared(detonation.position, *position);
            if target.is_employee && distance_sq <= LANDMINE_KILL_RADIUS * LANDMINE_KILL_RADIUS {
                damage.send(IncomingDamageEvent {
                    target: *entity,
                    raw_amount: LANDMINE_DAMAGE_MAX,
                    damage_type: DamageType::Standard,
                    source: detonation.mine,
                });
                immediate_kills.send(LandmineBlastImmediateKillEvent {
                    mine: detonation.mine,
                    target: *entity,
                    target_stable_id: target.stable_id,
                });
            } else if distance_sq > LANDMINE_INJURY_RADIUS_MIN * LANDMINE_INJURY_RADIUS_MIN
                && distance_sq <= LANDMINE_INJURY_RADIUS_MAX * LANDMINE_INJURY_RADIUS_MAX
            {
                damage.send(IncomingDamageEvent {
                    target: *entity,
                    raw_amount: LANDMINE_DAMAGE_MIN,
                    damage_type: DamageType::Standard,
                    source: detonation.mine,
                });
            }
        }
    }
}

fn landmine_chain_reaction(
    mut detonations: EventReader<LandmineDetonatedEvent>,
    mines: Query<(Entity, &Landmine, &SimPosition)>,
    mut chain_events: EventWriter<LandmineChainReactionEvent>,
    mut requests: EventWriter<LandmineDetonationRequestedEvent>,
) {
    let mut sorted_mines: Vec<(u64, Entity, Landmine, SimPosition)> = mines
        .iter()
        .map(|(entity, mine, position)| (mine.stable_id, entity, *mine, *position))
        .collect();
    sorted_mines.sort_by_key(|(stable_id, _, _, _)| *stable_id);

    for detonation in detonations.read() {
        for (_, entity, mine, position) in &sorted_mines {
            if *entity == detonation.mine || !mine.armed || mine.detonated {
                continue;
            }

            if distance_squared(detonation.position, *position)
                <= LANDMINE_CHAIN_REACTION_RADIUS * LANDMINE_CHAIN_REACTION_RADIUS
            {
                chain_events.send(LandmineChainReactionEvent {
                    source_mine: detonation.mine,
                    chained_mine: *entity,
                });
                requests.send(LandmineDetonationRequestedEvent {
                    mine: *entity,
                    trigger: LandmineTrigger::ChainReaction,
                });
            }
        }
    }
}

fn landmine_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    mines: Query<(&Landmine, &SimPosition)>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(LANDMINE_SOURCE_REVISION as u64);
    checksum.accumulate(LANDMINE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(LANDMINE_DAMAGE_MIN.to_bits() as u64);
    checksum.accumulate(LANDMINE_DAMAGE_MAX.to_bits() as u64);
    checksum.accumulate(LANDMINE_KILL_RADIUS.to_bits() as u64);
    checksum.accumulate(LANDMINE_INJURY_RADIUS_MIN.to_bits() as u64);
    checksum.accumulate(LANDMINE_INJURY_RADIUS_MAX.to_bits() as u64);
    checksum.accumulate(LANDMINE_CHAIN_REACTION_RADIUS.to_bits() as u64);

    for dependency in LANDMINE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in LANDMINE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for occurrence in LANDMINE_OCCURRENCES {
        accumulate_str(&mut checksum, 0x3000, occurrence.moon);
        checksum.accumulate(occurrence.min_landmines as u64);
        checksum.accumulate(occurrence.max_landmines as u64);
        checksum.accumulate(occurrence.average_landmines as u64);
    }

    for (mine, position) in &mines {
        checksum.accumulate(mine.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(mine.armed as u64);
        checksum.accumulate(mine.detonated as u64);
        checksum.accumulate(mine.stepped_on_by);
        checksum.accumulate(mine.temporary_disabled_ticks_remaining as u64);
    }
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}