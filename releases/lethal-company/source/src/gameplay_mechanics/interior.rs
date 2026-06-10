// Sources: vault/gameplay_mechanics/interior.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const INTERIOR_ID: &str = "interior";
pub const INTERIOR_NAME: &str = "Interior";
pub const INTERIOR_TYPE: &str = "gameplay_mechanics";
pub const INTERIOR_SUBTYPE: &str = "map_layout";
pub const INTERIOR_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Interior";
pub const INTERIOR_SOURCE_REVISION: u32 = 21352;
pub const INTERIOR_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const INTERIOR_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const INTERIOR_DEPENDS_ON: [&str; 14] = [
    "moons",
    "scrap",
    "the_company",
    "entities",
    "scanner",
    "fire_exit",
    "breaker_box",
    "apparatus",
    "landmine",
    "turret",
    "version_60",
    "key",
    "elevator",
    "maneater",
];

pub const INTERIOR_OVERVIEW: &str = "The [[Interior]] is the main indoor map space on most [[Moons]], where employees scavenger [[scrap]] for [[The Company]] while indoor [[entities]] spawn from vents.";

pub const INTERIOR_RULES: [InteriorRule; 17] = [
    InteriorRule {
        condition: "a moon is not [[March]]",
        outcome: "the interior can roll as The Factory, The Mansion, or The Mineshaft",
    },
    InteriorRule {
        condition: "a facility has a main entrance",
        outcome: "the [[scanner]] can be used to locate it, and that entrance is the primary path in and out of the facility",
    },
    InteriorRule {
        condition: "a facility has a [[Fire Exit]]",
        outcome: "it leads into a random interior location and can replace an existing doorway space",
    },
    InteriorRule {
        condition: "an indoor entity spawns",
        outcome: "it uses a vent as the spawn point",
    },
    InteriorRule {
        condition: "a vent is chosen for a spawn",
        outcome: "the vent emits increasing rumbling or growling until the delay ends and the entity appears",
    },
    InteriorRule {
        condition: "a vent has never spawned an entity during the current day",
        outcome: "it is closed",
    },
    InteriorRule {
        condition: "a vent has spawned at least one entity during the current day",
        outcome: "it is open",
    },
    InteriorRule {
        condition: "a vent is open",
        outcome: "it can still be used for additional spawns",
    },
    InteriorRule {
        condition: "a map has a [[Breaker Box]]",
        outcome: "flipping switches left restores power and flipping any switch right cuts power to the entire map",
    },
    InteriorRule {
        condition: "power is cut",
        outcome: "lights go off and secure doors remain opened until power is restored",
    },
    InteriorRule {
        condition: "the [[Apparatus]] is removed from its socket in The Factory",
        outcome: "power to the entire interior is permanently cut",
    },
    InteriorRule {
        condition: "[[Landmine]]s or [[Turret]]s are present",
        outcome: "they continue functioning without local power and still require temporary command-terminal disabling",
    },
    InteriorRule {
        condition: "the moon is [[March]]",
        outcome: "the interior is always The Factory",
    },
    InteriorRule {
        condition: "the moon is [[Experimentation]]",
        outcome: "the interior is overwhelmingly The Factory",
    },
    InteriorRule {
        condition: "the moon is [[Rend]]",
        outcome: "the interior is overwhelmingly The Mansion",
    },
    InteriorRule {
        condition: "the moon is [[Offense]]",
        outcome: "the interior is usually The Mineshaft",
    },
    InteriorRule {
        condition: "the moon is [[Liquidation]]",
        outcome: "the interior is usually The Mansion",
    },
];

pub const INTERIOR_MODIFIERS: [InteriorRule; 12] = [
    InteriorRule {
        condition: "the interior is The Factory",
        outcome: "steam leaks can spawn and temporarily fill areas with obscuring steam until the valve is pulled",
    },
    InteriorRule {
        condition: "a steam leak is fixed",
        outcome: "the steam dissipates after a few seconds",
    },
    InteriorRule {
        condition: "the interior is The Factory",
        outcome: "some [[Breaker Box]] switches may spawn already off",
    },
    InteriorRule {
        condition: "the interior is The Factory",
        outcome: "the [[Apparatus]] can spawn only there",
    },
    InteriorRule {
        condition: "the interior is The Mansion",
        outcome: "there are no secure doors, steam leaks, or an apparatus room",
    },
    InteriorRule {
        condition: "the interior is The Mansion",
        outcome: "toys and household items are more common as loot than in The Factory",
    },
    InteriorRule {
        condition: "the interior is The Mansion",
        outcome: "the [[Breaker Box]] exists",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "there are six extra loot spawns regardless of the moon's upper limit",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "steam leaks can spawn in corridor sections",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "slopes replace stairs and change movement speed depending on direction",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "blue-lit entrances may require a [[Key]]",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "fire exits can lead straight into the mines, bypassing the elevator",
    },
];

pub const INTERIOR_STRATEGY: [InteriorRule; 7] = [
    InteriorRule {
        condition: "you are searching for the main route",
        outcome: "use the [[scanner]] to find the main entrance first",
    },
    InteriorRule {
        condition: "you enter through a [[Fire Exit]]",
        outcome: "carry a [[Key]] or lockpick in case the route forward is blocked by locked doors",
    },
    InteriorRule {
        condition: "you are in The Factory",
        outcome: "watch for steam leaks before committing to long hallways",
    },
    InteriorRule {
        condition: "you are in The Factory",
        outcome: "use the [[Apparatus]] room as a high-value target because the [[Apparatus]] is only found there",
    },
    InteriorRule {
        condition: "you are in the Mansion",
        outcome: "expect more open rooms and fewer linear choke points than The Factory",
    },
    InteriorRule {
        condition: "you are in the Mineshaft",
        outcome: "use blue-lit tunnel entrances to locate cave sections and the highest scrap density",
    },
    InteriorRule {
        condition: "you are in the Mineshaft",
        outcome: "move quickly through [[Underwater Paths]] because oxygen time is limited",
    },
];

pub const INTERIOR_NOTES: [InteriorRule; 8] = [
    InteriorRule {
        condition: "a room is the Yellow Office Room",
        outcome: "its lights stay on even when the breaker is off or the [[Apparatus]] is removed",
    },
    InteriorRule {
        condition: "a fire exit appears in certain Storage Room variants",
        outcome: "it can spawn there, but not in the Stair Storage Room variant",
    },
    InteriorRule {
        condition: "a room is a Locker Room",
        outcome: "falling into the central pit is fatal",
    },
    InteriorRule {
        condition: "a room is a Factory [[Locker Room]]",
        outcome: "lockers may contain concealed scrap",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "entities may be blocked by crates, pipes, and minecarts in some locations",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft AND the fire exit is obstructed by pipes or crates",
        outcome: "the fire exit can become unusable",
    },
    InteriorRule {
        condition: "a door opens toward you in the Mineshaft",
        outcome: "you are likely moving toward the [[Elevator]]",
    },
    InteriorRule {
        condition: "a door opens away from you in the Mineshaft",
        outcome: "you are likely moving away from the [[Elevator]]",
    },
];

pub const INTERIOR_BEHAVIORAL_MECHANICS: [InteriorRule; 21] = [
    InteriorRule {
        condition: "an indoor entity spawn is being resolved",
        outcome: "the entity uses a vent spawn point",
    },
    InteriorRule {
        condition: "a vent is selected for spawning",
        outcome: "the vent produces increasing rumbling or growling during the delay",
    },
    InteriorRule {
        condition: "the vent has not spawned an entity yet that day",
        outcome: "the vent is closed",
    },
    InteriorRule {
        condition: "the vent has spawned at least one entity that day",
        outcome: "the vent is open for the rest of the day",
    },
    InteriorRule {
        condition: "a vent is open",
        outcome: "it can still spawn additional entities",
    },
    InteriorRule {
        condition: "a map contains a [[Breaker Box]]",
        outcome: "switches left turn power on and switches right cut power off",
    },
    InteriorRule {
        condition: "power is off",
        outcome: "lights in that map area are off and secure doors remain opened",
    },
    InteriorRule {
        condition: "power is restored",
        outcome: "secure doors become functional again",
    },
    InteriorRule {
        condition: "the [[Apparatus]] is removed in The Factory",
        outcome: "power to the entire interior is permanently cut",
    },
    InteriorRule {
        condition: "[[Landmine]]s or [[Turret]]s are present",
        outcome: "they continue to function without power in their sector",
    },
    InteriorRule {
        condition: "the interior is The Factory",
        outcome: "steam leaks can occur and temporarily reduce visibility",
    },
    InteriorRule {
        condition: "a steam leak is fixed",
        outcome: "the affected steam clears after a few seconds",
    },
    InteriorRule {
        condition: "the interior is The Factory",
        outcome: "some breaker switches can spawn off at map start",
    },
    InteriorRule {
        condition: "the interior is The Factory",
        outcome: "the [[Apparatus]] can spawn there",
    },
    InteriorRule {
        condition: "the interior is The Mansion",
        outcome: "secure doors, steam leaks, and an apparatus room do not exist",
    },
    InteriorRule {
        condition: "the interior is The Mansion",
        outcome: "the [[Breaker Box]] still exists",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "there are always six extra loot spawns",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "slopes slow upward travel and speed up downward travel",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "cave entrances may use blue lights and may require a [[Key]]",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "the fire exit may bypass the elevator and lead directly into the mines",
    },
    InteriorRule {
        condition: "the interior is The Mineshaft",
        outcome: "some routes can be blocked by crates or pipes and become unusable",
    },
];

pub const MINESHAFT_EXTRA_LOOT_SPAWNS: u8 = 6;
pub const STEAM_CLEAR_DELAY_TICKS: u32 = 60;

pub struct InteriorPlugin;

impl Plugin for InteriorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteriorState>()
            .add_event::<InteriorVentSpawnRequestEvent>()
            .add_event::<InteriorVentSpawnResolvedEvent>()
            .add_event::<InteriorPowerSwitchEvent>()
            .add_event::<InteriorApparatusRemovedEvent>()
            .add_event::<InteriorSteamLeakFixedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    interior_resolve_vent_spawn,
                    interior_apply_power_switches,
                    interior_apply_apparatus_removal,
                    interior_clear_fixed_steam_leaks,
                    interior_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InteriorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteriorFamily {
    Factory,
    Mansion,
    Mineshaft,
}

impl InteriorFamily {
    pub fn id(self) -> &'static str {
        match self {
            Self::Factory => "the_factory",
            Self::Mansion => "the_mansion",
            Self::Mineshaft => "the_mineshaft",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Factory => "The Factory",
            Self::Mansion => "The Mansion",
            Self::Mineshaft => "The Mineshaft",
        }
    }

    pub fn checksum_value(self) -> u64 {
        match self {
            Self::Factory => 1,
            Self::Mansion => 2,
            Self::Mineshaft => 3,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct InteriorState {
    pub family: InteriorFamily,
    pub has_breaker_box: bool,
    pub power_on: bool,
    pub permanent_power_cut: bool,
    pub secure_doors_functional: bool,
    pub active_steam_leaks: u32,
    pub cleared_steam_leaks: u32,
    pub next_steam_clear_tick: u64,
    pub vent_spawn_requests: u64,
    pub vent_spawns_resolved: u64,
    pub opened_vents: u32,
    pub last_spawn_vent_id: u64,
    pub mineshaft_extra_loot_spawns: u8,
}

impl Default for InteriorState {
    fn default() -> Self {
        Self {
            family: InteriorFamily::Factory,
            has_breaker_box: true,
            power_on: true,
            permanent_power_cut: false,
            secure_doors_functional: true,
            active_steam_leaks: 0,
            cleared_steam_leaks: 0,
            next_steam_clear_tick: 0,
            vent_spawn_requests: 0,
            vent_spawns_resolved: 0,
            opened_vents: 0,
            last_spawn_vent_id: 0,
            mineshaft_extra_loot_spawns: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteriorVentSpawnRequestEvent {
    pub vent_id: u64,
    pub entity_kind_id: &'static str,
    pub vent_has_spawned_today: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteriorVentSpawnResolvedEvent {
    pub vent_id: u64,
    pub entity_kind_id: &'static str,
    pub vent_was_open: bool,
    pub vent_is_open: bool,
    pub rumble_or_growl_delay_started: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteriorPowerSwitchEvent {
    pub switch_left: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteriorApparatusRemovedEvent {
    pub from_factory_socket: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteriorSteamLeakFixedEvent {
    pub leak_id: u64,
}

pub fn interior_extra_loot_spawns(family: InteriorFamily) -> u8 {
    match family {
        InteriorFamily::Mineshaft => MINESHAFT_EXTRA_LOOT_SPAWNS,
        InteriorFamily::Factory | InteriorFamily::Mansion => 0,
    }
}

pub fn interior_apparatus_can_spawn(family: InteriorFamily) -> bool {
    family == InteriorFamily::Factory
}

pub fn interior_has_secure_doors_steam_or_apparatus_room(family: InteriorFamily) -> bool {
    family != InteriorFamily::Mansion
}

pub fn interior_family_for_moon(moon_id: &str) -> InteriorFamily {
    match moon_id {
        "march" => InteriorFamily::Factory,
        "rend" | "liquidation" => InteriorFamily::Mansion,
        "offense" => InteriorFamily::Mineshaft,
        "experimentation" => InteriorFamily::Factory,
        _ => InteriorFamily::Factory,
    }
}

fn interior_resolve_vent_spawn(
    mut request_events: EventReader<InteriorVentSpawnRequestEvent>,
    mut resolved_events: EventWriter<InteriorVentSpawnResolvedEvent>,
    mut state: ResMut<InteriorState>,
) {
    for event in request_events.read() {
        state.vent_spawn_requests = state.vent_spawn_requests.wrapping_add(1);
        state.vent_spawns_resolved = state.vent_spawns_resolved.wrapping_add(1);
        state.last_spawn_vent_id = event.vent_id;

        if !event.vent_has_spawned_today {
            state.opened_vents = state.opened_vents.wrapping_add(1);
        }

        resolved_events.send(InteriorVentSpawnResolvedEvent {
            vent_id: event.vent_id,
            entity_kind_id: event.entity_kind_id,
            vent_was_open: event.vent_has_spawned_today,
            vent_is_open: true,
            rumble_or_growl_delay_started: true,
        });
    }
}

fn interior_apply_power_switches(
    mut power_events: EventReader<InteriorPowerSwitchEvent>,
    mut state: ResMut<InteriorState>,
) {
    for event in power_events.read() {
        if !state.has_breaker_box || state.permanent_power_cut {
            state.power_on = false;
            state.secure_doors_functional = false;
            continue;
        }

        state.power_on = event.switch_left;
        state.secure_doors_functional = state.power_on;
    }
}

fn interior_apply_apparatus_removal(
    mut apparatus_events: EventReader<InteriorApparatusRemovedEvent>,
    mut state: ResMut<InteriorState>,
) {
    for event in apparatus_events.read() {
        if state.family == InteriorFamily::Factory && event.from_factory_socket {
            state.permanent_power_cut = true;
            state.power_on = false;
            state.secure_doors_functional = false;
        }
    }
}

fn interior_clear_fixed_steam_leaks(
    mut fixed_events: EventReader<InteriorSteamLeakFixedEvent>,
    tick: Res<SimTick>,
    mut state: ResMut<InteriorState>,
) {
    for _event in fixed_events.read() {
        if state.active_steam_leaks > 0 {
            state.active_steam_leaks -= 1;
            state.cleared_steam_leaks = state.cleared_steam_leaks.wrapping_add(1);
            state.next_steam_clear_tick = tick.0.wrapping_add(STEAM_CLEAR_DELAY_TICKS as u64);
        }
    }
}

fn interior_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<InteriorState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(INTERIOR_SOURCE_REVISION as u64);
    checksum.accumulate(INTERIOR_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(MINESHAFT_EXTRA_LOOT_SPAWNS as u64);
    checksum.accumulate(STEAM_CLEAR_DELAY_TICKS as u64);

    accumulate_str(&mut checksum, 0x1000, INTERIOR_ID);
    accumulate_str(&mut checksum, 0x1001, INTERIOR_NAME);
    accumulate_str(&mut checksum, 0x1002, INTERIOR_TYPE);
    accumulate_str(&mut checksum, 0x1003, INTERIOR_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, INTERIOR_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, INTERIOR_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, INTERIOR_OVERVIEW);

    for dependency in INTERIOR_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in INTERIOR_RULES {
        accumulate_rule(&mut checksum, 0x3000, rule);
    }

    for modifier in INTERIOR_MODIFIERS {
        accumulate_rule(&mut checksum, 0x4000, modifier);
    }

    for strategy in INTERIOR_STRATEGY {
        accumulate_rule(&mut checksum, 0x5000, strategy);
    }

    for note in INTERIOR_NOTES {
        accumulate_rule(&mut checksum, 0x6000, note);
    }

    for mechanic in INTERIOR_BEHAVIORAL_MECHANICS {
        accumulate_rule(&mut checksum, 0x7000, mechanic);
    }

    checksum.accumulate(state.family.checksum_value());
    checksum.accumulate(state.has_breaker_box as u64);
    checksum.accumulate(state.power_on as u64);
    checksum.accumulate(state.permanent_power_cut as u64);
    checksum.accumulate(state.secure_doors_functional as u64);
    checksum.accumulate(state.active_steam_leaks as u64);
    checksum.accumulate(state.cleared_steam_leaks as u64);
    checksum.accumulate(state.next_steam_clear_tick);
    checksum.accumulate(state.vent_spawn_requests);
    checksum.accumulate(state.vent_spawns_resolved);
    checksum.accumulate(state.opened_vents as u64);
    checksum.accumulate(state.last_spawn_vent_id);
    checksum.accumulate(state.mineshaft_extra_loot_spawns as u64);
}

fn accumulate_rule(checksum: &mut SimChecksumState, salt: u64, rule: InteriorRule) {
    accumulate_str(checksum, salt, rule.condition);
    accumulate_str(checksum, salt ^ 1, rule.outcome);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}