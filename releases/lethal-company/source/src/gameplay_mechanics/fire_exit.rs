// Sources: vault/gameplay_mechanics/fire_exit.md
use bevy::prelude::*;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

pub const FIRE_EXIT_ID: &str = "fire_exit";
pub const FIRE_EXIT_NAME: &str = "Fire Exit";
pub const FIRE_EXIT_TYPE: &str = "gameplay_mechanics";
pub const FIRE_EXIT_SUBTYPE: &str = "mechanic";
pub const FIRE_EXIT_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Fire_Exit";
pub const FIRE_EXIT_SOURCE_REVISION: u32 = 18978;
pub const FIRE_EXIT_EXTRACTED_AT: &str = "2026-06-07";
pub const FIRE_EXIT_CONFIDENCE_BASIS_POINTS: u16 = 9700;

pub const FIRE_EXIT_DEPENDS_ON: [&str; 14] = [
    "41_experimentation",
    "220_assurance",
    "56_vow",
    "21_offense",
    "61_march",
    "20_adamance",
    "85_rend",
    "7_dine",
    "8_titan",
    "5_embrion",
    "68_artifice",
    "extension_ladder",
    "ship",
    "coil_head",
];

pub const FIRE_EXIT_OVERVIEW: &str = "Alternative entry and exit point for abandoned facilities and manors, with fixed exterior placements by moon and randomized interior placements.";

pub const FIRE_EXIT_INTERIOR_LOCATIONS: [FireExitInteriorLocation; 5] = [
    FireExitInteriorLocation::Staircase,
    FireExitInteriorLocation::GapRoom,
    FireExitInteriorLocation::StorageRoom,
    FireExitInteriorLocation::ManorRoom,
    FireExitInteriorLocation::ManorLibrary,
];

pub const FIRE_EXIT_EXTERIOR_PLACEMENTS: [FireExitExteriorPlacement; 13] = [
    FireExitExteriorPlacement {
        moon_id: "41_experimentation",
        count_for_moon: 1,
        placement: "up a fire escape to the left of the main entrance",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "220_assurance",
        count_for_moon: 1,
        placement: "at one end of the pipeline, accessible from a wooden scaffolding",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "56_vow",
        count_for_moon: 1,
        placement: "at the toe of the dam",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "21_offense",
        count_for_moon: 1,
        placement: "at one end of the pipeline",
        hidden: false,
        requires_extension_ladder_after_ship_landed: true,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "61_march",
        count_for_moon: 3,
        placement: "around the back of the ship and directly straight, on the backside of a hill",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "61_march",
        count_for_moon: 3,
        placement: "out the ship and directly left",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "61_march",
        count_for_moon: 3,
        placement: "out the ship and around the left side of the lake pit",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "20_adamance",
        count_for_moon: 1,
        placement: "at one end of the gorge",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: true,
    },
    FireExitExteriorPlacement {
        moon_id: "85_rend",
        count_for_moon: 1,
        placement: "out the ship and directly straight, at the bottom of a ravine",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "7_dine",
        count_for_moon: 1,
        placement: "just forward and left from the ship",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "8_titan",
        count_for_moon: 1,
        placement: "very near the main entrance",
        hidden: false,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "5_embrion",
        count_for_moon: 1,
        placement: "behind the huge rock to the right of the ship, then 90 degrees to the left",
        hidden: true,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
    FireExitExteriorPlacement {
        moon_id: "68_artifice",
        count_for_moon: 1,
        placement: "very near the main entrance, to the right",
        hidden: true,
        requires_extension_ladder_after_ship_landed: false,
        inaccessible_from_inside_gorge: false,
    },
];

pub const FIRE_EXIT_BEHAVIORAL_MECHANICS: [FireExitBehaviorRule; 20] = [
    FireExitBehaviorRule {
        condition: "a fire exit is inside an abandoned facility or manor",
        outcome: "its interior location is chosen randomly",
    },
    FireExitBehaviorRule {
        condition: "a fire exit appears in a facility interior",
        outcome: "it can occur in staircases, gap rooms, storage rooms, or any room in the manor",
    },
    FireExitBehaviorRule {
        condition: "a fire exit appears in a manor room",
        outcome: "libraries can hide it",
    },
    FireExitBehaviorRule {
        condition: "a fire exit is present",
        outcome: "it is not guaranteed to have an unlocked line of doors connecting it to the rest of the interior",
    },
    FireExitBehaviorRule {
        condition: "the moon is 41_experimentation",
        outcome: "there is 1 exterior fire exit up a fire escape to the left of the main entrance",
    },
    FireExitBehaviorRule {
        condition: "the moon is 220_assurance",
        outcome: "there is 1 exterior fire exit at one end of the pipeline, accessible from a wooden scaffolding",
    },
    FireExitBehaviorRule {
        condition: "the moon is 56_vow",
        outcome: "there is 1 exterior fire exit at the toe of the dam",
    },
    FireExitBehaviorRule {
        condition: "the moon is 21_offense",
        outcome: "there is 1 exterior fire exit at one end of the pipeline, and it is inaccessible without an extension_ladder after the ship has landed",
    },
    FireExitBehaviorRule {
        condition: "the moon is 61_march",
        outcome: "there are 3 exterior fire exits",
    },
    FireExitBehaviorRule {
        condition: "the moon is 61_march",
        outcome: "one fire exit is around the back of the ship and directly straight, on the backside of a hill",
    },
    FireExitBehaviorRule {
        condition: "the moon is 61_march",
        outcome: "one fire exit is out the ship and directly left",
    },
    FireExitBehaviorRule {
        condition: "the moon is 61_march",
        outcome: "one fire exit is out the ship and around the left side of the lake pit",
    },
    FireExitBehaviorRule {
        condition: "the moon is 20_adamance",
        outcome: "there is 1 exterior fire exit at one end of the gorge, and it is inaccessible from inside the gorge",
    },
    FireExitBehaviorRule {
        condition: "the moon is 85_rend",
        outcome: "there is 1 exterior fire exit out the ship and directly straight, at the bottom of a ravine",
    },
    FireExitBehaviorRule {
        condition: "the moon is 7_dine",
        outcome: "there is 1 exterior fire exit just forward and left from the ship",
    },
    FireExitBehaviorRule {
        condition: "the moon is 8_titan",
        outcome: "there is 1 exterior fire exit very near the main entrance",
    },
    FireExitBehaviorRule {
        condition: "the moon is 5_embrion",
        outcome: "there is 1 hidden exterior fire exit behind the huge rock to the right of the ship, then 90 degrees to the left",
    },
    FireExitBehaviorRule {
        condition: "the moon is 68_artifice",
        outcome: "there is 1 hidden exterior fire exit very near the main entrance, to the right",
    },
    FireExitBehaviorRule {
        condition: "each moon page",
        outcome: "the map marks the fire exits",
    },
    FireExitBehaviorRule {
        condition: "an employee enters through the door",
        outcome: "they face it immediately and become vulnerable to threats like coil_heads",
    },
];

pub struct FireExitPlugin;

impl Plugin for FireExitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireExitState>()
            .add_event::<FireExitInteriorPlacementRequestEvent>()
            .add_event::<FireExitInteriorPlacementResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    fire_exit_resolve_interior_placement_requests,
                    fire_exit_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FireExitBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FireExitExteriorPlacement {
    pub moon_id: &'static str,
    pub count_for_moon: u8,
    pub placement: &'static str,
    pub hidden: bool,
    pub requires_extension_ladder_after_ship_landed: bool,
    pub inaccessible_from_inside_gorge: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FireExitInteriorLocation {
    Staircase,
    GapRoom,
    StorageRoom,
    ManorRoom,
    ManorLibrary,
}

impl FireExitInteriorLocation {
    pub fn stable_id(self) -> u64 {
        match self {
            Self::Staircase => 1,
            Self::GapRoom => 2,
            Self::StorageRoom => 3,
            Self::ManorRoom => 4,
            Self::ManorLibrary => 5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Staircase => "staircase",
            Self::GapRoom => "gap_room",
            Self::StorageRoom => "storage_room",
            Self::ManorRoom => "manor_room",
            Self::ManorLibrary => "manor_library",
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct FireExitState {
    pub interior_locations_are_randomized: bool,
    pub facility_fire_exits_can_spawn_in_staircases: bool,
    pub facility_fire_exits_can_spawn_in_gap_rooms: bool,
    pub facility_fire_exits_can_spawn_in_storage_rooms: bool,
    pub facility_fire_exits_can_spawn_in_manor_rooms: bool,
    pub manor_libraries_can_hide_fire_exits: bool,
    pub unlocked_door_path_guaranteed: bool,
    pub placement_requests: u64,
    pub placements_resolved: u64,
    pub last_request_id: u64,
    pub last_selected_location: FireExitInteriorLocation,
}

impl Default for FireExitState {
    fn default() -> Self {
        Self {
            interior_locations_are_randomized: true,
            facility_fire_exits_can_spawn_in_staircases: true,
            facility_fire_exits_can_spawn_in_gap_rooms: true,
            facility_fire_exits_can_spawn_in_storage_rooms: true,
            facility_fire_exits_can_spawn_in_manor_rooms: true,
            manor_libraries_can_hide_fire_exits: true,
            unlocked_door_path_guaranteed: false,
            placement_requests: 0,
            placements_resolved: 0,
            last_request_id: 0,
            last_selected_location: FireExitInteriorLocation::Staircase,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireExitInteriorPlacementRequestEvent {
    pub request_id: u64,
    pub moon_id: &'static str,
    pub facility_seed_salt: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireExitInteriorPlacementResolvedEvent {
    pub request_id: u64,
    pub moon_id: &'static str,
    pub selected_location: FireExitInteriorLocation,
}

pub fn fire_exit_exterior_placements() -> &'static [FireExitExteriorPlacement] {
    &FIRE_EXIT_EXTERIOR_PLACEMENTS
}

pub fn fire_exit_exterior_placements_for_moon(
    moon_id: &str,
) -> Vec<FireExitExteriorPlacement> {
    FIRE_EXIT_EXTERIOR_PLACEMENTS
        .iter()
        .copied()
        .filter(|placement| placement.moon_id == moon_id)
        .collect()
}

pub fn fire_exit_behavioral_mechanics() -> &'static [FireExitBehaviorRule] {
    &FIRE_EXIT_BEHAVIORAL_MECHANICS
}

pub fn fire_exit_interior_locations() -> &'static [FireExitInteriorLocation] {
    &FIRE_EXIT_INTERIOR_LOCATIONS
}

fn fire_exit_resolve_interior_placement_requests(
    mut request_events: EventReader<FireExitInteriorPlacementRequestEvent>,
    mut resolved_events: EventWriter<FireExitInteriorPlacementResolvedEvent>,
    mut state: ResMut<FireExitState>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for event in request_events.read() {
        state.placement_requests = state.placement_requests.wrapping_add(1);
        state.placements_resolved = state.placements_resolved.wrapping_add(1);
        state.last_request_id = event.request_id;

        let salt = 0xF1EE_0001_u64 ^ event.facility_seed_salt ^ event.request_id;
        let mut rng = tick_rng(game_seed.0, tick.0, salt);
        let selected_index = (rng.next_u32() as usize) % FIRE_EXIT_INTERIOR_LOCATIONS.len();
        let selected_location = FIRE_EXIT_INTERIOR_LOCATIONS[selected_index];

        state.last_selected_location = selected_location;

        resolved_events.send(FireExitInteriorPlacementResolvedEvent {
            request_id: event.request_id,
            moon_id: event.moon_id,
            selected_location,
        });
    }
}

fn fire_exit_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<FireExitState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(FIRE_EXIT_SOURCE_REVISION as u64);
    checksum.accumulate(FIRE_EXIT_CONFIDENCE_BASIS_POINTS as u64);

    accumulate_str(&mut checksum, 0x0100, FIRE_EXIT_ID);
    accumulate_str(&mut checksum, 0x0101, FIRE_EXIT_NAME);
    accumulate_str(&mut checksum, 0x0102, FIRE_EXIT_TYPE);
    accumulate_str(&mut checksum, 0x0103, FIRE_EXIT_SUBTYPE);
    accumulate_str(&mut checksum, 0x0104, FIRE_EXIT_SOURCE_URL);
    accumulate_str(&mut checksum, 0x0105, FIRE_EXIT_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x0106, FIRE_EXIT_OVERVIEW);

    for dependency in FIRE_EXIT_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in FIRE_EXIT_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for placement in FIRE_EXIT_EXTERIOR_PLACEMENTS {
        accumulate_str(&mut checksum, 0x3000, placement.moon_id);
        checksum.accumulate(0x3001 ^ placement.count_for_moon as u64);
        accumulate_str(&mut checksum, 0x3002, placement.placement);
        checksum.accumulate(0x3003 ^ placement.hidden as u64);
        checksum.accumulate(
            0x3004 ^ placement.requires_extension_ladder_after_ship_landed as u64,
        );
        checksum.accumulate(0x3005 ^ placement.inaccessible_from_inside_gorge as u64);
    }

    for location in FIRE_EXIT_INTERIOR_LOCATIONS {
        checksum.accumulate(0x4000 ^ location.stable_id());
        accumulate_str(&mut checksum, 0x4001, location.label());
    }

    checksum.accumulate(state.interior_locations_are_randomized as u64);
    checksum.accumulate(state.facility_fire_exits_can_spawn_in_staircases as u64);
    checksum.accumulate(state.facility_fire_exits_can_spawn_in_gap_rooms as u64);
    checksum.accumulate(state.facility_fire_exits_can_spawn_in_storage_rooms as u64);
    checksum.accumulate(state.facility_fire_exits_can_spawn_in_manor_rooms as u64);
    checksum.accumulate(state.manor_libraries_can_hide_fire_exits as u64);
    checksum.accumulate(state.unlocked_door_path_guaranteed as u64);
    checksum.accumulate(state.placement_requests);
    checksum.accumulate(state.placements_resolved);
    checksum.accumulate(state.last_request_id);
    checksum.accumulate(state.last_selected_location.stable_id());
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}