// Sources: vault/harmless_entity_pages/vain_shroud.md, vault/store_items/weed_killer.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, SimHz, SimPosition, SimTick, UnitStats};

pub const VAIN_SHROUD_ID: &str = "vain_shroud";
pub const VAIN_SHROUD_NAME: &str = "Vain Shroud";
pub const VAIN_SHROUD_TYPE: &str = "harmless_entity_pages";
pub const VAIN_SHROUD_SUBTYPE: &str = "weed";
pub const VAIN_SHROUD_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Vain_Shroud";
pub const VAIN_SHROUD_SOURCE_REVISION: u32 = 21367;
pub const VAIN_SHROUD_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const VAIN_SHROUD_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const VAIN_SHROUD_DANGER: &str = "N/A";
pub const VAIN_SHROUD_SCIENTIFIC_NAME: &str = "Phlebodium ruber";
pub const VAIN_SHROUD_HP: I32F32 = I32F32::lit("3");
pub const VAIN_SHROUD_CONTACT_DAMAGE: I32F32 = I32F32::lit("0");
pub const VAIN_SHROUD_WEED_KILLER_SHRINK_PER_SPRAY: I32F32 = I32F32::lit("1");
pub const VAIN_SHROUD_MIN_SIZE: I32F32 = I32F32::lit("1");
pub const VAIN_SHROUD_DEFAULT_SIZE: I32F32 = I32F32::lit("3");
pub const VAIN_SHROUD_DEFAULT_GROUP_SIZE: u16 = 6;
pub const VAIN_SHROUD_SPAWN_CHANCE_PERCENT: u16 = 20;

pub const WEED_KILLER_ID: &str = "weed_killer";
pub const WEED_KILLER_BUY_CREDITS: u16 = 20;
pub const WEED_KILLER_SELL_CREDITS: u16 = 0;
pub const WEED_KILLER_WEIGHT: u16 = 0;
pub const WEED_KILLER_BATTERY_LIFE_SECONDS: u16 = 30;
pub const WEED_KILLER_CONDUCTIVE: bool = false;

pub const VAIN_SHROUD_DEPENDS_ON: [&str; 3] = ["kidnapper_fox", "employee", "weed_killer"];

pub const VAIN_SHROUD_OCCURRENCE: [VainShroudOccurrence; 11] = [
    VainShroudOccurrence {
        moon: "Embrion",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Dine",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Titan",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Adamance",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Offense",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Experimentation",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Vow",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "March",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Artifice",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Rend",
        spawn_chance: 20,
    },
    VainShroudOccurrence {
        moon: "Assurance",
        spawn_chance: 20,
    },
];

pub const VAIN_SHROUD_BEHAVIORAL_MECHANICS: [VainShroudBehaviorRule; 6] = [
    VainShroudBehaviorRule {
        condition: "Vain Shrouds are present",
        outcome: "Kidnapper Fox can spawn",
    },
    VainShroudBehaviorRule {
        condition: "Employees remove Vain Shrouds with Weed Killer before a Kidnapper Fox appears",
        outcome: "the fox stops spawning",
    },
    VainShroudBehaviorRule {
        condition: "Employees remove Vain Shrouds with Weed Killer after a Kidnapper Fox has already spawned",
        outcome: "its prey is harder to kill",
    },
    VainShroudBehaviorRule {
        condition: "Vain Shrouds are present",
        outcome: "they cannot hurt the player",
    },
    VainShroudBehaviorRule {
        condition: "the listed moons are active",
        outcome: "the base spawn chance is 20%",
    },
    VainShroudBehaviorRule {
        condition: "Vain Shrouds generate",
        outcome: "they commonly spawn in large groups and can vary in size",
    },
];

pub struct VainShroudPlugin;

impl Plugin for VainShroudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VainShroudState>()
            .add_event::<SpawnVainShroudEvent>()
            .add_event::<VainShroudWeedKillerSprayEvent>()
            .add_event::<VainShroudRemovedEvent>()
            .add_event::<VainShroudKidnapperFoxSpawnedEvent>()
            .add_event::<VainShroudKidnapperFoxSpawnGateEvent>()
            .add_event::<VainShroudFoxSpawnSuppressedEvent>()
            .add_event::<VainShroudFoxPreyHardenedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_vain_shroud,
                    vain_shroud_register_kidnapper_fox_spawned,
                    vain_shroud_apply_weed_killer_spray,
                    vain_shroud_recount_presence,
                    vain_shroud_update_kidnapper_fox_gate,
                    vain_shroud_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VainShroudBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VainShroudOccurrence {
    pub moon: &'static str,
    pub spawn_chance: u16,
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VainShroudState {
    pub active_count: u32,
    pub kidnapper_fox_spawned: bool,
    pub fox_spawn_suppressed: bool,
    pub prey_hardened: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VainShroud;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VainShroudStableId {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VainShroudGroup {
    pub group_id: u64,
    pub group_size: u16,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VainShroudSize {
    pub current: I32F32,
    pub initial: I32F32,
}

impl Default for VainShroudSize {
    fn default() -> Self {
        Self {
            current: VAIN_SHROUD_DEFAULT_SIZE,
            initial: VAIN_SHROUD_DEFAULT_SIZE,
        }
    }
}

#[derive(Bundle)]
pub struct VainShroudBundle {
    pub name: Name,
    pub shroud: VainShroud,
    pub stable_id: VainShroudStableId,
    pub group: VainShroudGroup,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub size: VainShroudSize,
}

impl VainShroudBundle {
    pub fn new(event: SpawnVainShroudEvent) -> Self {
        let size = clamp_size(event.size);

        Self {
            name: Name::new(VAIN_SHROUD_NAME),
            shroud: VainShroud,
            stable_id: VainShroudStableId {
                stable_id: event.stable_id,
            },
            group: VainShroudGroup {
                group_id: event.group_id,
                group_size: event.group_size,
            },
            position: event.position,
            health: Health::full(VAIN_SHROUD_HP),
            stats: UnitStats {
                move_speed: I32F32::lit("0"),
                attack_range: I32F32::lit("0"),
                attack_damage: VAIN_SHROUD_CONTACT_DAMAGE,
                attack_speed: I32F32::lit("0"),
                watch_range: I32F32::lit("0"),
            },
            size: VainShroudSize {
                current: size,
                initial: size,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnVainShroudEvent {
    pub position: SimPosition,
    pub stable_id: u64,
    pub group_id: u64,
    pub group_size: u16,
    pub size: I32F32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct VainShroudWeedKillerSprayEvent {
    pub shroud: Entity,
    pub employee: Entity,
    pub amount: I32F32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct VainShroudRemovedEvent {
    pub shroud: Entity,
    pub employee: Entity,
    pub stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VainShroudKidnapperFoxSpawnedEvent {
    pub fox: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VainShroudKidnapperFoxSpawnGateEvent {
    pub can_spawn: bool,
    pub active_shrouds: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VainShroudFoxSpawnSuppressedEvent;

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VainShroudFoxPreyHardenedEvent;

fn spawn_vain_shroud(
    mut commands: Commands,
    mut events: EventReader<SpawnVainShroudEvent>,
    mut state: ResMut<VainShroudState>,
) {
    for event in events.read() {
        commands.spawn(VainShroudBundle::new(*event));
        state.active_count = state.active_count.saturating_add(1);
        state.fox_spawn_suppressed = false;
    }
}

fn vain_shroud_register_kidnapper_fox_spawned(
    mut events: EventReader<VainShroudKidnapperFoxSpawnedEvent>,
    mut state: ResMut<VainShroudState>,
) {
    for _ in events.read() {
        state.kidnapper_fox_spawned = true;
    }
}

fn vain_shroud_apply_weed_killer_spray(
    mut commands: Commands,
    mut spray_events: EventReader<VainShroudWeedKillerSprayEvent>,
    mut removed_events: EventWriter<VainShroudRemovedEvent>,
    mut shrouds: Query<(Entity, &VainShroudStableId, &mut VainShroudSize), With<VainShroud>>,
) {
    for event in spray_events.read() {
        for (entity, stable_id, mut size) in shrouds.iter_mut() {
            if event.shroud != entity {
                continue;
            }

            let spray_amount = if event.amount <= I32F32::lit("0") {
                VAIN_SHROUD_WEED_KILLER_SHRINK_PER_SPRAY
            } else {
                event.amount
            };

            size.current -= spray_amount;

            if size.current <= I32F32::lit("0") {
                removed_events.send(VainShroudRemovedEvent {
                    shroud: entity,
                    employee: event.employee,
                    stable_id: stable_id.stable_id,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}

fn vain_shroud_recount_presence(
    mut state: ResMut<VainShroudState>,
    mut removed_events: EventReader<VainShroudRemovedEvent>,
    mut suppressed_events: EventWriter<VainShroudFoxSpawnSuppressedEvent>,
    mut hardened_events: EventWriter<VainShroudFoxPreyHardenedEvent>,
    shrouds: Query<(), With<VainShroud>>,
) {
    let removed_count = removed_events.read().count() as u32;
    state.active_count = shrouds.iter().count() as u32;

    if removed_count == 0 || state.active_count != 0 {
        return;
    }

    if state.kidnapper_fox_spawned {
        if !state.prey_hardened {
            state.prey_hardened = true;
            hardened_events.send(VainShroudFoxPreyHardenedEvent);
        }
    } else if !state.fox_spawn_suppressed {
        state.fox_spawn_suppressed = true;
        suppressed_events.send(VainShroudFoxSpawnSuppressedEvent);
    }
}

fn vain_shroud_update_kidnapper_fox_gate(
    state: Res<VainShroudState>,
    mut events: EventWriter<VainShroudKidnapperFoxSpawnGateEvent>,
) {
    events.send(VainShroudKidnapperFoxSpawnGateEvent {
        can_spawn: state.active_count > 0 && !state.fox_spawn_suppressed,
        active_shrouds: state.active_count,
    });
}

fn vain_shroud_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    state: Res<VainShroudState>,
    shrouds: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &VainShroudStableId,
            &VainShroudGroup,
            &VainShroudSize,
        ),
        With<VainShroud>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(VAIN_SHROUD_SOURCE_REVISION as u64);
    checksum.accumulate(VAIN_SHROUD_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(VAIN_SHROUD_HP.to_bits() as u64);
    checksum.accumulate(VAIN_SHROUD_CONTACT_DAMAGE.to_bits() as u64);
    checksum.accumulate(VAIN_SHROUD_WEED_KILLER_SHRINK_PER_SPRAY.to_bits() as u64);
    checksum.accumulate(VAIN_SHROUD_MIN_SIZE.to_bits() as u64);
    checksum.accumulate(VAIN_SHROUD_DEFAULT_SIZE.to_bits() as u64);
    checksum.accumulate(VAIN_SHROUD_DEFAULT_GROUP_SIZE as u64);
    checksum.accumulate(VAIN_SHROUD_SPAWN_CHANCE_PERCENT as u64);
    checksum.accumulate(WEED_KILLER_BUY_CREDITS as u64);
    checksum.accumulate(WEED_KILLER_SELL_CREDITS as u64);
    checksum.accumulate(WEED_KILLER_WEIGHT as u64);
    checksum.accumulate(WEED_KILLER_BATTERY_LIFE_SECONDS as u64);
    checksum.accumulate(WEED_KILLER_CONDUCTIVE as u64);
    checksum.accumulate(state.active_count as u64);
    checksum.accumulate(state.kidnapper_fox_spawned as u64);
    checksum.accumulate(state.fox_spawn_suppressed as u64);
    checksum.accumulate(state.prey_hardened as u64);

    accumulate_str(&mut checksum, 0x1000, VAIN_SHROUD_ID);
    accumulate_str(&mut checksum, 0x1001, VAIN_SHROUD_NAME);
    accumulate_str(&mut checksum, 0x1002, VAIN_SHROUD_TYPE);
    accumulate_str(&mut checksum, 0x1003, VAIN_SHROUD_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, VAIN_SHROUD_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, VAIN_SHROUD_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, VAIN_SHROUD_DANGER);
    accumulate_str(&mut checksum, 0x1007, VAIN_SHROUD_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1008, WEED_KILLER_ID);

    for dependency in VAIN_SHROUD_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for occurrence in VAIN_SHROUD_OCCURRENCE {
        accumulate_str(&mut checksum, 0x3000, occurrence.moon);
        checksum.accumulate(occurrence.spawn_chance as u64);
    }

    for rule in VAIN_SHROUD_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (position, health, stats, stable_id, group, size) in shrouds.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(stable_id.stable_id);
        checksum.accumulate(group.group_id);
        checksum.accumulate(group.group_size as u64);
        checksum.accumulate(size.current.to_bits() as u64);
        checksum.accumulate(size.initial.to_bits() as u64);
    }
}

fn clamp_size(size: I32F32) -> I32F32 {
    if size < VAIN_SHROUD_MIN_SIZE {
        VAIN_SHROUD_MIN_SIZE
    } else {
        size
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}