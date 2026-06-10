// Sources: vault/scrap_items/brass_bell.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{NoiseEmittedEvent, SimChecksumState, SimPosition};

pub const BRASS_BELL_ID: &str = "brass_bell";
pub const BRASS_BELL_NAME: &str = "Brass bell";
pub const BRASS_BELL_TYPE: &str = "scrap_items";
pub const BRASS_BELL_SUBTYPE: &str = "bell";
pub const BRASS_BELL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Brass_Bell";
pub const BRASS_BELL_SOURCE_REVISION: u32 = 20390;
pub const BRASS_BELL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const BRASS_BELL_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const BRASS_BELL_EFFECTS: &str = "Makes bell chime sounds when dropped";
pub const BRASS_BELL_WEIGHT: I32F32 = I32F32::lit("24");
pub const BRASS_BELL_CONDUCTIVE: bool = true;
pub const BRASS_BELL_MIN_VALUE: I32F32 = I32F32::lit("48");
pub const BRASS_BELL_MAX_VALUE: I32F32 = I32F32::lit("79");
pub const BRASS_BELL_TWO_HANDED: bool = false;
pub const BRASS_BELL_ITEM_ID: u32 = 18;
pub const BRASS_BELL_CHIME_SOUND_AMOUNT: I32F32 = I32F32::lit("24");
pub const BRASS_BELL_HELD_MOVEMENT_SOUND_AMOUNT: I32F32 = I32F32::lit("24");

pub const BRASS_BELL_DEPENDS_ON: [&str; 0] = [];

pub const BRASS_BELL_AUDIO: [BrassBellAudioCue; 2] = [
    BrassBellAudioCue {
        key: "dropped",
        description: "Sound when a Bell is dropped.",
    },
    BrassBellAudioCue {
        key: "moving_held",
        description: "Sound when moving and holding a Bell.",
    },
];

pub const BRASS_BELL_SPAWN_CHANCES: [BrassBellSpawnChance; 6] = [
    BrassBellSpawnChance {
        moon: "rend",
        chance: I32F32::lit("4.45"),
    },
    BrassBellSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("4.14"),
    },
    BrassBellSpawnChance {
        moon: "titan",
        chance: I32F32::lit("3.98"),
    },
    BrassBellSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("3.91"),
    },
    BrassBellSpawnChance {
        moon: "vow",
        chance: I32F32::lit("3.39"),
    },
    BrassBellSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("1.88"),
    },
];

pub const BRASS_BELL_BEHAVIORAL_MECHANICS: [BrassBellBehaviorRule; 3] = [
    BrassBellBehaviorRule {
        condition: "the bell is dropped",
        outcome: "it plays a bell chime sound",
    },
    BrassBellBehaviorRule {
        condition: "the bell is held by a moving employee",
        outcome: "it plays a distinct held-movement sound",
    },
    BrassBellBehaviorRule {
        condition: "an entity can hear the sound",
        outcome: "the chime can draw attention to the bell",
    },
];

pub struct BrassBellPlugin;

impl Plugin for BrassBellPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBrassBellEvent>()
            .add_event::<BrassBellDroppedEvent>()
            .add_event::<BrassBellHeldMovementSoundEvent>()
            .add_event::<BrassBellAudibleAlertEvent>()
            .add_event::<BrassBellSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_brass_bell,
                    brass_bell_pickup_item_bar_bridge,
                    brass_bell_drop_sound,
                    brass_bell_held_movement_sound,
                    brass_bell_emit_audible_alerts,
                    brass_bell_sell_bridge,
                    brass_bell_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrassBellBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrassBellAudioCue {
    pub key: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrassBellSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrassBell {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrassBellScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
    pub item_id: u32,
}

impl Default for BrassBellScrap {
    fn default() -> Self {
        Self {
            min_value: BRASS_BELL_MIN_VALUE,
            max_value: BRASS_BELL_MAX_VALUE,
            weight: BRASS_BELL_WEIGHT,
            conductive: BRASS_BELL_CONDUCTIVE,
            two_handed: BRASS_BELL_TWO_HANDED,
            item_id: BRASS_BELL_ITEM_ID,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrassBellHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
    pub is_moving: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrassBellSoundState {
    pub was_held: bool,
    pub dropped_chimes: u64,
    pub held_movement_sounds: u64,
}

#[derive(Bundle)]
pub struct BrassBellBundle {
    pub name: Name,
    pub brass_bell: BrassBell,
    pub scrap: BrassBellScrap,
    pub position: SimPosition,
    pub held_by: BrassBellHeldBy,
    pub sound_state: BrassBellSoundState,
}

impl BrassBellBundle {
    pub fn new(event: SpawnBrassBellEvent) -> Self {
        Self {
            name: Name::new(BRASS_BELL_NAME),
            brass_bell: BrassBell {
                stable_id: event.stable_id,
            },
            scrap: BrassBellScrap::default(),
            position: event.position,
            held_by: BrassBellHeldBy::default(),
            sound_state: BrassBellSoundState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnBrassBellEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrassBellDroppedEvent {
    pub brass_bell_entity: Entity,
    pub brass_bell_stable_id: u64,
    pub previous_employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrassBellHeldMovementSoundEvent {
    pub brass_bell_entity: Entity,
    pub brass_bell_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrassBellAudibleAlertEvent {
    pub source: Entity,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrassBellSoldEvent {
    pub brass_bell_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn brass_bell_value_range() -> (I32F32, I32F32) {
    (BRASS_BELL_MIN_VALUE, BRASS_BELL_MAX_VALUE)
}

pub fn brass_bell_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    BRASS_BELL_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_brass_bell(mut commands: Commands, mut events: EventReader<SpawnBrassBellEvent>) {
    for event in events.read() {
        commands.spawn(BrassBellBundle::new(*event));
    }
}

fn brass_bell_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    bells: Query<(&BrassBell, &BrassBellHeldBy), Changed<BrassBellHeldBy>>,
) {
    for (bell, held_by) in &bells {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: BRASS_BELL_ID,
                two_handed: BRASS_BELL_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = bell.stable_id;
        }
    }
}

fn brass_bell_drop_sound(
    mut dropped_events: EventWriter<BrassBellDroppedEvent>,
    mut bells: Query<(
        Entity,
        &BrassBell,
        &BrassBellHeldBy,
        &SimPosition,
        &mut BrassBellSoundState,
    )>,
) {
    for (entity, bell, held_by, position, mut sound_state) in &mut bells {
        if sound_state.was_held && !held_by.is_held {
            sound_state.dropped_chimes = sound_state.dropped_chimes.wrapping_add(1);
            dropped_events.send(BrassBellDroppedEvent {
                brass_bell_entity: entity,
                brass_bell_stable_id: bell.stable_id,
                previous_employee_id: held_by.employee_id,
                position: *position,
            });
        }

        sound_state.was_held = held_by.is_held;
    }
}

fn brass_bell_held_movement_sound(
    mut movement_sound_events: EventWriter<BrassBellHeldMovementSoundEvent>,
    mut bells: Query<(
        Entity,
        &BrassBell,
        &BrassBellHeldBy,
        &SimPosition,
        &mut BrassBellSoundState,
    )>,
) {
    for (entity, bell, held_by, position, mut sound_state) in &mut bells {
        if !held_by.is_held || !held_by.is_moving {
            continue;
        }

        sound_state.held_movement_sounds = sound_state.held_movement_sounds.wrapping_add(1);
        movement_sound_events.send(BrassBellHeldMovementSoundEvent {
            brass_bell_entity: entity,
            brass_bell_stable_id: bell.stable_id,
            employee_id: held_by.employee_id,
            position: *position,
        });
    }
}

fn brass_bell_emit_audible_alerts(
    mut dropped_events: EventReader<BrassBellDroppedEvent>,
    mut movement_sound_events: EventReader<BrassBellHeldMovementSoundEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<BrassBellAudibleAlertEvent>,
) {
    for event in dropped_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.brass_bell_entity,
            position: event.position,
            amount: BRASS_BELL_CHIME_SOUND_AMOUNT,
        });

        alert_events.send(BrassBellAudibleAlertEvent {
            source: event.brass_bell_entity,
            position: event.position,
            amount: BRASS_BELL_CHIME_SOUND_AMOUNT,
        });
    }

    for event in movement_sound_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.brass_bell_entity,
            position: event.position,
            amount: BRASS_BELL_HELD_MOVEMENT_SOUND_AMOUNT,
        });

        alert_events.send(BrassBellAudibleAlertEvent {
            source: event.brass_bell_entity,
            position: event.position,
            amount: BRASS_BELL_HELD_MOVEMENT_SOUND_AMOUNT,
        });
    }
}

fn brass_bell_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<BrassBellSoldEvent>,
    bells: Query<(&BrassBell, &BrassBellScrap)>,
) {
    for event in sell_events.read() {
        for (bell, scrap) in &bells {
            if bell.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(BrassBellSoldEvent {
                brass_bell_stable_id: bell.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn brass_bell_checksum(
    mut checksum: ResMut<SimChecksumState>,
    bells: Query<(
        &BrassBell,
        &BrassBellScrap,
        &SimPosition,
        &BrassBellHeldBy,
        &BrassBellSoundState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, BRASS_BELL_ID);
    accumulate_str(&mut checksum, 0x1001, BRASS_BELL_NAME);
    accumulate_str(&mut checksum, 0x1002, BRASS_BELL_TYPE);
    accumulate_str(&mut checksum, 0x1003, BRASS_BELL_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, BRASS_BELL_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, BRASS_BELL_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, BRASS_BELL_EXTRACTED_AT);

    checksum.accumulate(BRASS_BELL_SOURCE_REVISION as u64);
    checksum.accumulate(BRASS_BELL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(BRASS_BELL_WEIGHT.to_bits() as u64);
    checksum.accumulate(BRASS_BELL_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(BRASS_BELL_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(BRASS_BELL_CONDUCTIVE as u64);
    checksum.accumulate(BRASS_BELL_TWO_HANDED as u64);
    checksum.accumulate(BRASS_BELL_ITEM_ID as u64);
    checksum.accumulate(BRASS_BELL_CHIME_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(BRASS_BELL_HELD_MOVEMENT_SOUND_AMOUNT.to_bits() as u64);

    for dependency in BRASS_BELL_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for audio in BRASS_BELL_AUDIO {
        accumulate_str(&mut checksum, 0x3000, audio.key);
        accumulate_str(&mut checksum, 0x3001, audio.description);
    }

    for spawn_chance in BRASS_BELL_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in BRASS_BELL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (bell, scrap, position, held_by, sound_state) in &bells {
        checksum.accumulate(bell.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(scrap.item_id as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(held_by.is_moving as u64);
        checksum.accumulate(sound_state.was_held as u64);
        checksum.accumulate(sound_state.dropped_chimes);
        checksum.accumulate(sound_state.held_movement_sounds);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}