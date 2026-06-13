// Sources: vault/scrap_items/player_body.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const PLAYER_BODY_ID: &str = "player_body";
pub const PLAYER_BODY_NAME: &str = "Player Body";
pub const PLAYER_BODY_TYPE: &str = "scrap_items";
pub const PLAYER_BODY_SUBTYPE: &str = "special";
pub const PLAYER_BODY_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Player_Body";
pub const PLAYER_BODY_SOURCE_REVISION: u32 = 21202;
pub const PLAYER_BODY_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const PLAYER_BODY_CONFIDENCE_BASIS_POINTS: u16 = 93;
pub const PLAYER_BODY_EFFECTS: &str = "Appears as a ragdoll when an employee dies.";
pub const PLAYER_BODY_WEIGHT: I32F32 = I32F32::lit("11");
pub const PLAYER_BODY_CONDUCTIVE: bool = false;
pub const PLAYER_BODY_SELL_VALUE: I32F32 = I32F32::lit("5");
pub const PLAYER_BODY_TWO_HANDED: bool = true;
pub const PLAYER_BODY_FINE_RETURNED_BASIS_POINTS: u32 = 800;
pub const PLAYER_BODY_FINE_NOT_RETURNED_BASIS_POINTS: u32 = 2000;

pub const PLAYER_BODY_DEPENDS_ON: [&str; 35] = [
    "scrap", "lethal_company", "employee", "entity", "credits",
    "the_ship", "the_company", "the_company_monster", "company_cruiser",
    "easter_egg", "jetpack", "landmine", "shovel", "old_bird",
    "dropship", "extension_ladder", "spike_trap", "circuit_bee",
    "mask_hornet", "weather", "interior", "forest_keeper",
    "double_barrel", "nutcracker", "feiopar", "barber", "kitchen_knife",
    "bracken", "coil_head", "cadaver_bloom", "snare_flea", "dramatic_mask",
    "baboon_hawk", "ghost_girl", "hygrodere",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerBodyBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

pub const PLAYER_BODY_BEHAVIORAL_MECHANICS: [PlayerBodyBehaviorRule; 16] = [
    PlayerBodyBehaviorRule { condition: "an employee dies", outcome: "a player_body spawns at the death location and carried items drop there" },
    PlayerBodyBehaviorRule { condition: "the death occurred on the job and the body is returned to the_ship", outcome: "the crew pays an insurance penalty of 8% of total credits" },
    PlayerBodyBehaviorRule { condition: "the body is not returned to the_ship", outcome: "the crew pays an additional 12% insurance penalty for a total of 20% of total credits" },
    PlayerBodyBehaviorRule { condition: "the ship leaves into orbit", outcome: "the player_body does not persist and the employee respawns upon entering space" },
    PlayerBodyBehaviorRule { condition: "an employee dies at the_company for any reason except by the_company_monster", outcome: "the player_body can be sold to the_company for credits" },
    PlayerBodyBehaviorRule { condition: "an employee dies by forest_keeper, earth_leviathan, masked, or by sinking in quicksand during rainy weather", outcome: "no player_body spawns and all carried items are permanently deleted" },
    PlayerBodyBehaviorRule { condition: "an employee dies to the_company_monster", outcome: "the carried items drop instead of being lost permanently" },
    PlayerBodyBehaviorRule { condition: "an employee dies to a barber", outcome: "the body is cut in half and only the top half can be collected" },
    PlayerBodyBehaviorRule { condition: "an employee dies to a bunker_spider", outcome: "the body can be wrapped in a web and left hanging from the ceiling" },
    PlayerBodyBehaviorRule { condition: "a bunker_spider picks up a player_body", outcome: "the body can be wrapped in a web and only be obtainable by using the teleporter" },
    PlayerBodyBehaviorRule { condition: "an employee dies to a cadaver_bloom, ghost_girl, kidnapper_fox, or the Factory main entrance fan", outcome: "the body is decapitated" },
    PlayerBodyBehaviorRule { condition: "an employee dies to a coil_head", outcome: "the body loses its forearms and its head is replaced by a spring" },
    PlayerBodyBehaviorRule { condition: "an employee dies to a feiopar", outcome: "scratch marks appear on the body" },
    PlayerBodyBehaviorRule { condition: "an employee dies to a giant_sapsucker", outcome: "the body is segmented and none of it can be retrieved, resulting in a 20% fee" },
    PlayerBodyBehaviorRule { condition: "an employee dies to a hygrodere", outcome: "the gloves and boots disappear and only the suit remains" },
    PlayerBodyBehaviorRule { condition: "an employee dies to an old_bird using its flamethrower or to a company_cruiser explosion", outcome: "the body appears charred" },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerBodyVariant {
    Standard,
    CutInHalf,
    WrappedInWeb,
    Decapitated,
    SpringHead,
    ScratchMarked,
    Segmented,
    SuitOnly,
    Charred,
}

impl Default for PlayerBodyVariant {
    fn default() -> Self {
        Self::Standard
    }
}

impl PlayerBodyVariant {
    pub fn discriminant(self) -> u64 {
        match self {
            Self::Standard => 0,
            Self::CutInHalf => 1,
            Self::WrappedInWeb => 2,
            Self::Decapitated => 3,
            Self::SpringHead => 4,
            Self::ScratchMarked => 5,
            Self::Segmented => 6,
            Self::SuitOnly => 7,
            Self::Charred => 8,
        }
    }
}

pub struct PlayerBodyPlugin;

impl Plugin for PlayerBodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPlayerBodyEvent>()
            .add_event::<PlayerBodyReturnedToShipEvent>()
            .add_event::<PlayerBodyShipDepartureEvent>()
            .add_event::<PlayerBodyInsuranceFineEvent>()
            .add_event::<PlayerBodySoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_player_body,
                    player_body_pickup_item_bar_bridge,
                    player_body_return_to_ship,
                    player_body_ship_departure_fine,
                    player_body_sell_bridge,
                    player_body_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerBody {
    pub stable_id: u64,
    pub employee_id: u64,
    pub variant: PlayerBodyVariant,
}

impl Default for PlayerBody {
    fn default() -> Self {
        Self { stable_id: 0, employee_id: 0, variant: PlayerBodyVariant::default() }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerBodyScrap {
    pub sell_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for PlayerBodyScrap {
    fn default() -> Self {
        Self {
            sell_value: PLAYER_BODY_SELL_VALUE,
            weight: PLAYER_BODY_WEIGHT,
            conductive: PLAYER_BODY_CONDUCTIVE,
            two_handed: PLAYER_BODY_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerBodyHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerBodyState {
    pub on_ship: bool,
    pub sold: bool,
}

#[derive(Bundle)]
pub struct PlayerBodyBundle {
    pub name: Name,
    pub body: PlayerBody,
    pub scrap: PlayerBodyScrap,
    pub position: SimPosition,
    pub held_by: PlayerBodyHeldBy,
    pub state: PlayerBodyState,
}

impl PlayerBodyBundle {
    pub fn new(event: SpawnPlayerBodyEvent) -> Self {
        Self {
            name: Name::new(PLAYER_BODY_NAME),
            body: PlayerBody { stable_id: event.stable_id, employee_id: event.employee_id, variant: event.variant },
            scrap: PlayerBodyScrap::default(),
            position: event.position,
            held_by: PlayerBodyHeldBy::default(),
            state: PlayerBodyState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnPlayerBodyEvent {
    pub stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub variant: PlayerBodyVariant,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerBodyReturnedToShipEvent {
    pub body_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerBodyShipDepartureEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerBodyInsuranceFineEvent {
    pub employee_id: u64,
    pub body_stable_id: u64,
    pub fine_basis_points: u32,
    pub body_returned: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerBodySoldEvent {
    pub body_stable_id: u64,
    pub employee_id: u64,
    pub credit_value: I32F32,
}

pub fn player_body_sell_value() -> I32F32 {
    PLAYER_BODY_SELL_VALUE
}

fn spawn_player_body(mut commands: Commands, mut events: EventReader<SpawnPlayerBodyEvent>) {
    for event in events.read() {
        commands.spawn(PlayerBodyBundle::new(*event));
    }
}

fn player_body_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    bodies: Query<(&PlayerBody, &PlayerBodyHeldBy), Changed<PlayerBodyHeldBy>>,
) {
    for (body, held_by) in &bodies {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: PLAYER_BODY_ID,
                two_handed: PLAYER_BODY_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = body.stable_id;
        }
    }
}

fn player_body_return_to_ship(
    mut return_events: EventReader<PlayerBodyReturnedToShipEvent>,
    mut fine_events: EventWriter<PlayerBodyInsuranceFineEvent>,
    mut bodies: Query<(&PlayerBody, &mut PlayerBodyState)>,
) {
    for event in return_events.read() {
        for (body, mut state) in &mut bodies {
            if body.stable_id != event.body_stable_id {
                continue;
            }
            state.on_ship = true;
            fine_events.send(PlayerBodyInsuranceFineEvent {
                employee_id: event.employee_id,
                body_stable_id: body.stable_id,
                fine_basis_points: PLAYER_BODY_FINE_RETURNED_BASIS_POINTS,
                body_returned: true,
            });
        }
    }
}

fn player_body_ship_departure_fine(
    mut departure_events: EventReader<PlayerBodyShipDepartureEvent>,
    mut fine_events: EventWriter<PlayerBodyInsuranceFineEvent>,
    mut commands: Commands,
    bodies: Query<(Entity, &PlayerBody, &PlayerBodyState)>,
) {
    for _ in departure_events.read() {
        for (entity, body, state) in &bodies {
            if !state.on_ship && !state.sold {
                fine_events.send(PlayerBodyInsuranceFineEvent {
                    employee_id: body.employee_id,
                    body_stable_id: body.stable_id,
                    fine_basis_points: PLAYER_BODY_FINE_NOT_RETURNED_BASIS_POINTS,
                    body_returned: false,
                });
            }
            commands.entity(entity).despawn();
        }
    }
}

fn player_body_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<PlayerBodySoldEvent>,
    mut bodies: Query<(&PlayerBody, &mut PlayerBodyState)>,
) {
    for event in sell_events.read() {
        for (body, mut state) in &mut bodies {
            if body.stable_id != event.scrap_entity_id {
                continue;
            }
            state.sold = true;
            sold_events.send(PlayerBodySoldEvent {
                body_stable_id: body.stable_id,
                employee_id: body.employee_id,
                credit_value: PLAYER_BODY_SELL_VALUE,
            });
        }
    }
}

fn player_body_checksum(
    mut checksum: ResMut<SimChecksumState>,
    bodies: Query<(&PlayerBody, &PlayerBodyScrap, &SimPosition, &PlayerBodyHeldBy, &PlayerBodyState)>,
) {
    accumulate_str(&mut checksum, 0x1000, PLAYER_BODY_ID);
    accumulate_str(&mut checksum, 0x1001, PLAYER_BODY_NAME);
    accumulate_str(&mut checksum, 0x1002, PLAYER_BODY_TYPE);
    accumulate_str(&mut checksum, 0x1003, PLAYER_BODY_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, PLAYER_BODY_EFFECTS);
    checksum.accumulate(PLAYER_BODY_SOURCE_REVISION as u64);
    checksum.accumulate(PLAYER_BODY_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(PLAYER_BODY_WEIGHT.to_bits() as u64);
    checksum.accumulate(PLAYER_BODY_SELL_VALUE.to_bits() as u64);
    checksum.accumulate(PLAYER_BODY_CONDUCTIVE as u64);
    checksum.accumulate(PLAYER_BODY_TWO_HANDED as u64);
    checksum.accumulate(PLAYER_BODY_FINE_RETURNED_BASIS_POINTS as u64);
    checksum.accumulate(PLAYER_BODY_FINE_NOT_RETURNED_BASIS_POINTS as u64);
    for dependency in PLAYER_BODY_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }
    for rule in PLAYER_BODY_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }
    for (body, scrap, position, held_by, state) in &bodies {
        checksum.accumulate(body.stable_id);
        checksum.accumulate(body.employee_id);
        checksum.accumulate(body.variant.discriminant());
        checksum.accumulate(scrap.sell_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(state.on_ship as u64);
        checksum.accumulate(state.sold as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}