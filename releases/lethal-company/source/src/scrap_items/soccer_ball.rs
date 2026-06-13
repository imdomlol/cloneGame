// Sources: vault/scrap_items/soccer_ball.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const SOCCER_BALL_ID: &str = "soccer_ball";
pub const SOCCER_BALL_NAME: &str = "Soccer Ball";
pub const SOCCER_BALL_TYPE: &str = "scrap_items";
pub const SOCCER_BALL_SUBTYPE: &str = "soccer_ball";
pub const SOCCER_BALL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Soccer_Ball";
pub const SOCCER_BALL_SOURCE_REVISION: u32 = 20400;
pub const SOCCER_BALL_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const SOCCER_BALL_CONFIDENCE_BASIS_POINTS: u16 = 87;

pub const SOCCER_BALL_EFFECTS: &str = "Can be kicked around by employees or entities";
pub const SOCCER_BALL_WEIGHT: I32F32 = I32F32::lit("19");
pub const SOCCER_BALL_CONDUCTIVE: bool = false;
pub const SOCCER_BALL_MIN_VALUE: I32F32 = I32F32::lit("44");
pub const SOCCER_BALL_MAX_VALUE: I32F32 = I32F32::lit("71");
pub const SOCCER_BALL_TWO_HANDED: bool = true;
pub const SOCCER_BALL_WIKI_ID: u32 = 71;
pub const SOCCER_BALL_KICK_SOUND_AMOUNT: I32F32 = I32F32::lit("60");
pub const SOCCER_BALL_LAND_SOUND_AMOUNT: I32F32 = I32F32::lit("40");
pub const SOCCER_BALL_KICK_FLIGHT_TICKS: u16 = 30;

pub const SOCCER_BALL_DEPENDS_ON: [&str; 2] = ["eyeless_dog", "the_company"];

pub const SOCCER_BALL_SPAWN_CHANCES: [SoccerBallSpawnChance; 6] = [
    SoccerBallSpawnChance { moon: "embrion", chance: I32F32::lit("3.01") },
    SoccerBallSpawnChance { moon: "artifice", chance: I32F32::lit("2.88") },
    SoccerBallSpawnChance { moon: "assurance", chance: I32F32::lit("2.7") },
    SoccerBallSpawnChance { moon: "rend", chance: I32F32::lit("2.59") },
    SoccerBallSpawnChance { moon: "march", chance: I32F32::lit("2.4") },
    SoccerBallSpawnChance { moon: "vow", chance: I32F32::lit("2.26") },
];

pub const SOCCER_BALL_BEHAVIORAL_MECHANICS: [SoccerBallBehaviorRule; 4] = [
    SoccerBallBehaviorRule {
        condition: "the Soccer Ball is kicked",
        outcome: "it moves by being kicked rather than picked up and emits a sound on each kick",
    },
    SoccerBallBehaviorRule {
        condition: "the Soccer Ball is dropped or lands after a kick",
        outcome: "it emits a landing sound",
    },
    SoccerBallBehaviorRule {
        condition: "the noise reaches eyeless_dog or any other hearing entity",
        outcome: "that entity can be alerted by the sound",
    },
    SoccerBallBehaviorRule {
        condition: "the Soccer Ball is sold to the_company",
        outcome: "it can be exchanged for credits",
    },
];

pub struct SoccerBallPlugin;

impl Plugin for SoccerBallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSoccerBallEvent>()
            .add_event::<SoccerBallKickedEvent>()
            .add_event::<SoccerBallLandedEvent>()
            .add_event::<SoccerBallAudibleAlertEvent>()
            .add_event::<SoccerBallSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_soccer_ball,
                    soccer_ball_pickup_item_bar_bridge,
                    soccer_ball_kick_from_item_bar,
                    soccer_ball_emit_kick_noise,
                    soccer_ball_advance_flight,
                    soccer_ball_emit_land_noise,
                    soccer_ball_sell_bridge,
                    soccer_ball_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoccerBallBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoccerBallSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoccerBall {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoccerBallScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for SoccerBallScrap {
    fn default() -> Self {
        Self {
            min_value: SOCCER_BALL_MIN_VALUE,
            max_value: SOCCER_BALL_MAX_VALUE,
            weight: SOCCER_BALL_WEIGHT,
            conductive: SOCCER_BALL_CONDUCTIVE,
            two_handed: SOCCER_BALL_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoccerBallHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoccerBallKickState {
    pub kicks: u64,
    pub last_kicked_tick: u64,
    pub flight_ticks_remaining: u16,
}

impl Default for SoccerBallKickState {
    fn default() -> Self {
        Self {
            kicks: 0,
            last_kicked_tick: 0,
            flight_ticks_remaining: 0,
        }
    }
}

#[derive(Bundle)]
pub struct SoccerBallBundle {
    pub name: Name,
    pub soccer_ball: SoccerBall,
    pub scrap: SoccerBallScrap,
    pub position: SimPosition,
    pub held_by: SoccerBallHeldBy,
    pub kick_state: SoccerBallKickState,
}

impl SoccerBallBundle {
    pub fn new(event: SpawnSoccerBallEvent) -> Self {
        Self {
            name: Name::new(SOCCER_BALL_NAME),
            soccer_ball: SoccerBall { stable_id: event.stable_id },
            scrap: SoccerBallScrap::default(),
            position: event.position,
            held_by: SoccerBallHeldBy::default(),
            kick_state: SoccerBallKickState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnSoccerBallEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoccerBallKickedEvent {
    pub soccer_ball_entity: Entity,
    pub soccer_ball_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoccerBallLandedEvent {
    pub soccer_ball_entity: Entity,
    pub soccer_ball_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoccerBallAudibleAlertEvent {
    pub source: Entity,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoccerBallSoldEvent {
    pub soccer_ball_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn soccer_ball_value_range() -> (I32F32, I32F32) {
    (SOCCER_BALL_MIN_VALUE, SOCCER_BALL_MAX_VALUE)
}

pub fn soccer_ball_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    SOCCER_BALL_SPAWN_CHANCES
        .iter()
        .find(|sc| sc.moon == moon)
        .map(|sc| sc.chance)
}

fn spawn_soccer_ball(mut commands: Commands, mut events: EventReader<SpawnSoccerBallEvent>) {
    for event in events.read() {
        commands.spawn(SoccerBallBundle::new(*event));
    }
}

fn soccer_ball_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    soccer_balls: Query<(&SoccerBall, &SoccerBallHeldBy), Changed<SoccerBallHeldBy>>,
) {
    for (soccer_ball, held_by) in &soccer_balls {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: SOCCER_BALL_ID,
                two_handed: SOCCER_BALL_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = soccer_ball.stable_id;
        }
    }
}

fn soccer_ball_kick_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut kicked_events: EventWriter<SoccerBallKickedEvent>,
    mut soccer_balls: Query<(
        Entity,
        &SoccerBall,
        &SoccerBallHeldBy,
        &SimPosition,
        &mut SoccerBallKickState,
    )>,
    tick: Res<SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != SOCCER_BALL_ID
            || event.effect != ItemBarItemEffect::FunctionalActivated
        {
            continue;
        }
        for (entity, soccer_ball, held_by, position, mut kick_state) in &mut soccer_balls {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }
            kick_state.kicks = kick_state.kicks.wrapping_add(1);
            kick_state.last_kicked_tick = tick.0;
            kick_state.flight_ticks_remaining = SOCCER_BALL_KICK_FLIGHT_TICKS;
            kicked_events.send(SoccerBallKickedEvent {
                soccer_ball_entity: entity,
                soccer_ball_stable_id: soccer_ball.stable_id,
                employee_id: event.employee_id,
                position: *position,
            });
        }
    }
}

fn soccer_ball_emit_kick_noise(
    mut kicked_events: EventReader<SoccerBallKickedEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<SoccerBallAudibleAlertEvent>,
) {
    for event in kicked_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.soccer_ball_entity,
            position: event.position,
            amount: SOCCER_BALL_KICK_SOUND_AMOUNT,
        });
        alert_events.send(SoccerBallAudibleAlertEvent {
            source: event.soccer_ball_entity,
            position: event.position,
            amount: SOCCER_BALL_KICK_SOUND_AMOUNT,
        });
    }
}

fn soccer_ball_advance_flight(
    mut landed_events: EventWriter<SoccerBallLandedEvent>,
    mut soccer_balls: Query<(Entity, &SoccerBall, &SimPosition, &mut SoccerBallKickState)>,
) {
    for (entity, soccer_ball, position, mut kick_state) in &mut soccer_balls {
        if kick_state.flight_ticks_remaining == 0 {
            continue;
        }
        kick_state.flight_ticks_remaining -= 1;
        if kick_state.flight_ticks_remaining == 0 {
            landed_events.send(SoccerBallLandedEvent {
                soccer_ball_entity: entity,
                soccer_ball_stable_id: soccer_ball.stable_id,
                position: *position,
            });
        }
    }
}

fn soccer_ball_emit_land_noise(
    mut landed_events: EventReader<SoccerBallLandedEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<SoccerBallAudibleAlertEvent>,
) {
    for event in landed_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.soccer_ball_entity,
            position: event.position,
            amount: SOCCER_BALL_LAND_SOUND_AMOUNT,
        });
        alert_events.send(SoccerBallAudibleAlertEvent {
            source: event.soccer_ball_entity,
            position: event.position,
            amount: SOCCER_BALL_LAND_SOUND_AMOUNT,
        });
    }
}

fn soccer_ball_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<SoccerBallSoldEvent>,
    soccer_balls: Query<(&SoccerBall, &SoccerBallScrap)>,
) {
    for event in sell_events.read() {
        for (soccer_ball, scrap) in &soccer_balls {
            if soccer_ball.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(SoccerBallSoldEvent {
                soccer_ball_stable_id: soccer_ball.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn soccer_ball_checksum(
    mut checksum: ResMut<SimChecksumState>,
    soccer_balls: Query<(
        &SoccerBall,
        &SoccerBallScrap,
        &SimPosition,
        &SoccerBallHeldBy,
        &SoccerBallKickState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, SOCCER_BALL_ID);
    accumulate_str(&mut checksum, 0x1001, SOCCER_BALL_NAME);
    accumulate_str(&mut checksum, 0x1002, SOCCER_BALL_TYPE);
    accumulate_str(&mut checksum, 0x1003, SOCCER_BALL_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, SOCCER_BALL_EFFECTS);

    checksum.accumulate(SOCCER_BALL_SOURCE_REVISION as u64);
    checksum.accumulate(SOCCER_BALL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(SOCCER_BALL_WEIGHT.to_bits() as u64);
    checksum.accumulate(SOCCER_BALL_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(SOCCER_BALL_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(SOCCER_BALL_CONDUCTIVE as u64);
    checksum.accumulate(SOCCER_BALL_TWO_HANDED as u64);
    checksum.accumulate(SOCCER_BALL_WIKI_ID as u64);
    checksum.accumulate(SOCCER_BALL_KICK_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(SOCCER_BALL_LAND_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(SOCCER_BALL_KICK_FLIGHT_TICKS as u64);

    for dependency in SOCCER_BALL_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in SOCCER_BALL_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in SOCCER_BALL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (soccer_ball, scrap, position, held_by, kick_state) in &soccer_balls {
        checksum.accumulate(soccer_ball.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(kick_state.kicks);
        checksum.accumulate(kick_state.last_kicked_tick);
        checksum.accumulate(kick_state.flight_ticks_remaining as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}