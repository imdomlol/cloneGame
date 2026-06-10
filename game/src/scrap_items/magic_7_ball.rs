// Sources: vault/scrap_items/magic_7_ball.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const MAGIC_7_BALL_ID: &str = "magic_7_ball";
pub const MAGIC_7_BALL_NAME: &str = "Magic 7 ball";
pub const MAGIC_7_BALL_TYPE: &str = "scrap_items";
pub const MAGIC_7_BALL_SUBTYPE: &str = "scrap";
pub const MAGIC_7_BALL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Magic_7_Ball";
pub const MAGIC_7_BALL_SOURCE_REVISION: u32 = 20243;
pub const MAGIC_7_BALL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const MAGIC_7_BALL_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const MAGIC_7_BALL_EFFECTS: &str = "No gameplay effect.";
pub const MAGIC_7_BALL_WEIGHT: I32F32 = I32F32::lit("16");
pub const MAGIC_7_BALL_CONDUCTIVE: bool = false;
pub const MAGIC_7_BALL_TWO_HANDED: bool = false;
pub const MAGIC_7_BALL_MIN_VALUE: I32F32 = I32F32::lit("36");
pub const MAGIC_7_BALL_MAX_VALUE: I32F32 = I32F32::lit("71");

pub const MAGIC_7_BALL_DEPENDS_ON: [&str; 3] = ["scrap", "the_company", "credits"];

pub const MAGIC_7_BALL_BEHAVIORAL_MECHANICS: [Magic7BallBehaviorRule; 3] = [
    Magic7BallBehaviorRule {
        condition: "carried",
        outcome: "it provides no special gameplay effect",
    },
    Magic7BallBehaviorRule {
        condition: "sold to the_company",
        outcome: "it converts into credits as a scrap sale item",
    },
    Magic7BallBehaviorRule {
        condition: "spawned",
        outcome: "its value range is bounded by min: 36 and max: 71",
    },
];

pub struct Magic7BallPlugin;

impl Plugin for Magic7BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnMagic7BallEvent>()
            .add_event::<Magic7BallSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_magic_7_ball,
                    magic_7_ball_pickup_item_bar_bridge,
                    magic_7_ball_sell_bridge,
                    magic_7_ball_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Magic7BallBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Magic7Ball {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Magic7BallScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for Magic7BallScrap {
    fn default() -> Self {
        Self {
            min_value: MAGIC_7_BALL_MIN_VALUE,
            max_value: MAGIC_7_BALL_MAX_VALUE,
            weight: MAGIC_7_BALL_WEIGHT,
            conductive: MAGIC_7_BALL_CONDUCTIVE,
            two_handed: MAGIC_7_BALL_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Magic7BallHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Magic7BallNoGameplayEffect {
    pub carried_ticks: u64,
}

#[derive(Bundle)]
pub struct Magic7BallBundle {
    pub name: Name,
    pub magic_7_ball: Magic7Ball,
    pub scrap: Magic7BallScrap,
    pub position: SimPosition,
    pub held_by: Magic7BallHeldBy,
    pub no_gameplay_effect: Magic7BallNoGameplayEffect,
}

impl Magic7BallBundle {
    pub fn new(event: SpawnMagic7BallEvent) -> Self {
        Self {
            name: Name::new(MAGIC_7_BALL_NAME),
            magic_7_ball: Magic7Ball {
                stable_id: event.stable_id,
            },
            scrap: Magic7BallScrap::default(),
            position: event.position,
            held_by: Magic7BallHeldBy::default(),
            no_gameplay_effect: Magic7BallNoGameplayEffect::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnMagic7BallEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Magic7BallSoldEvent {
    pub magic_7_ball_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn magic_7_ball_value_range() -> (I32F32, I32F32) {
    (MAGIC_7_BALL_MIN_VALUE, MAGIC_7_BALL_MAX_VALUE)
}

fn spawn_magic_7_ball(mut commands: Commands, mut events: EventReader<SpawnMagic7BallEvent>) {
    for event in events.read() {
        commands.spawn(Magic7BallBundle::new(*event));
    }
}

fn magic_7_ball_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    magic_7_balls: Query<(&Magic7Ball, &Magic7BallHeldBy), Changed<Magic7BallHeldBy>>,
) {
    for (magic_7_ball, held_by) in &magic_7_balls {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: MAGIC_7_BALL_ID,
                two_handed: MAGIC_7_BALL_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = magic_7_ball.stable_id;
        }
    }
}

fn magic_7_ball_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<Magic7BallSoldEvent>,
    magic_7_balls: Query<(&Magic7Ball, &Magic7BallScrap)>,
) {
    for event in sell_events.read() {
        for (magic_7_ball, scrap) in &magic_7_balls {
            if magic_7_ball.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(Magic7BallSoldEvent {
                magic_7_ball_stable_id: magic_7_ball.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn magic_7_ball_checksum(
    mut checksum: ResMut<SimChecksumState>,
    magic_7_balls: Query<(
        &Magic7Ball,
        &Magic7BallScrap,
        &SimPosition,
        &Magic7BallHeldBy,
        &Magic7BallNoGameplayEffect,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, MAGIC_7_BALL_ID);
    accumulate_str(&mut checksum, 0x1001, MAGIC_7_BALL_NAME);
    accumulate_str(&mut checksum, 0x1002, MAGIC_7_BALL_TYPE);
    accumulate_str(&mut checksum, 0x1003, MAGIC_7_BALL_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, MAGIC_7_BALL_EFFECTS);

    checksum.accumulate(MAGIC_7_BALL_SOURCE_REVISION as u64);
    checksum.accumulate(MAGIC_7_BALL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(MAGIC_7_BALL_WEIGHT.to_bits() as u64);
    checksum.accumulate(MAGIC_7_BALL_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(MAGIC_7_BALL_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(MAGIC_7_BALL_CONDUCTIVE as u64);
    checksum.accumulate(MAGIC_7_BALL_TWO_HANDED as u64);

    for dependency in MAGIC_7_BALL_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in MAGIC_7_BALL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (magic_7_ball, scrap, position, held_by, no_gameplay_effect) in &magic_7_balls {
        checksum.accumulate(magic_7_ball.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(no_gameplay_effect.carried_ticks);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}