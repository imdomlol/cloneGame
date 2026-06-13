// Sources: vault/item_index_pages/items.md, vault/item_index_pages/scrap.md, vault/scrap_items/old_phone.md
use bevy::prelude::*;
use fixed::types::{I32F32, I64F64};
use std::collections::BTreeMap;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{ItemBarDropHeldItemEvent, ItemBarPickupEvent};
use crate::gameplay_mechanics::profit_quota::{FulfillProfitQuotaEvent, ProfitQuotaState};
use crate::sim::{SimChecksumState, SimTick};

pub const SCRAP_ECONOMY_ID: &str = "scrap_economy";
pub const SCRAP_ECONOMY_NAME: &str = "Scrap Economy";
pub const SCRAP_ECONOMY_TYPE: &str = "system";
pub const SCRAP_ECONOMY_SUBTYPE: &str = "economy";

pub const ITEMS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Items";
pub const ITEMS_SOURCE_REVISION: u32 = 21248;
pub const ITEMS_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const SCRAP_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Scrap";
pub const SCRAP_SOURCE_REVISION: u32 = 21237;
pub const SCRAP_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const OLD_PHONE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Old_Phone";
pub const OLD_PHONE_SOURCE_REVISION: u32 = 20358;
pub const OLD_PHONE_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const SPECIAL_CLIPBOARD_VALUE: i64 = 0;
pub const SPECIAL_CLIPBOARD_WEIGHT_LBS: i64 = 0;
pub const SPECIAL_CLIPBOARD_CONDUCTIVE: bool = false;
pub const SPECIAL_CLIPBOARD_TWO_HANDED: bool = false;

pub const SPECIAL_KEY_VALUE: i64 = 3;
pub const SPECIAL_KEY_WEIGHT_LBS: i64 = 0;
pub const SPECIAL_KEY_CONDUCTIVE: bool = true;
pub const SPECIAL_KEY_TWO_HANDED: bool = false;

pub const SPECIAL_SHOTGUN_SHELLS_VALUE: i64 = 0;
pub const SPECIAL_SHOTGUN_SHELLS_WEIGHT_LBS: i64 = 0;
pub const SPECIAL_SHOTGUN_SHELLS_CONDUCTIVE: bool = false;
pub const SPECIAL_SHOTGUN_SHELLS_TWO_HANDED: bool = false;

pub const SPECIAL_STICKY_NOTE_VALUE: i64 = 0;
pub const SPECIAL_STICKY_NOTE_WEIGHT_LBS: i64 = 0;
pub const SPECIAL_STICKY_NOTE_CONDUCTIVE: bool = false;
pub const SPECIAL_STICKY_NOTE_TWO_HANDED: bool = false;

pub const OLD_PHONE_ID: &str = "old_phone";
pub const OLD_PHONE_NAME: &str = "Old phone";
pub const OLD_PHONE_WEIGHT: I32F32 = I32F32::lit("5");
pub const OLD_PHONE_CONDUCTIVE: bool = false;
pub const OLD_PHONE_MIN_VALUE: I64F64 = I64F64::lit("48");
pub const OLD_PHONE_MAX_VALUE: I64F64 = I64F64::lit("63");
pub const OLD_PHONE_TWO_HANDED: bool = false;
pub const OLD_PHONE_WIKI_ID: u32 = 43;

pub const SINGLE_ITEM_DAY_CHANCE_BASIS_POINTS: u16 = 520;

pub const OLD_PHONE_SPAWN_CHANCES: [ScrapEconomySpawnChance; 8] = [
    ScrapEconomySpawnChance {
        moon: "titan",
        chance_basis_points: 237,
    },
    ScrapEconomySpawnChance {
        moon: "artifice",
        chance_basis_points: 185,
    },
    ScrapEconomySpawnChance {
        moon: "rend",
        chance_basis_points: 158,
    },
    ScrapEconomySpawnChance {
        moon: "offense",
        chance_basis_points: 96,
    },
    ScrapEconomySpawnChance {
        moon: "embrion",
        chance_basis_points: 93,
    },
    ScrapEconomySpawnChance {
        moon: "assurance",
        chance_basis_points: 82,
    },
    ScrapEconomySpawnChance {
        moon: "vow",
        chance_basis_points: 72,
    },
    ScrapEconomySpawnChance {
        moon: "adamance",
        chance_basis_points: 42,
    },
];

pub const SCRAP_ECONOMY_RULES: [ScrapEconomyRule; 15] = [
    ScrapEconomyRule {
        condition: "an item is classified as scrap",
        outcome: "it can be found on moons and collecting it is the main objective for employees",
    },
    ScrapEconomyRule {
        condition: "the entire crew dies during a day",
        outcome: "all scrap is lost",
    },
    ScrapEconomyRule {
        condition: "the ship leaves at midnight and no crew members are onboard",
        outcome: "all scrap is lost",
    },
    ScrapEconomyRule {
        condition: "the crew fails to meet profit_quota",
        outcome: "the_company ejects the crew into space and the run ends in a game over",
    },
    ScrapEconomyRule {
        condition: "an item is neither scrap nor store",
        outcome: "it is treated as a special item and is not lost when the entire crew dies",
    },
    ScrapEconomyRule {
        condition: "a scrap item is regular scrap",
        outcome: "its spawn chance on a moon equals its rarity divided by the sum of all rarities in that moon's scrap pool",
    },
    ScrapEconomyRule {
        condition: "a day is a single_item_day",
        outcome: "only one scrap type can spawn on that moon, and the event has a 5.2% chance to occur on a given day",
    },
    ScrapEconomyRule {
        condition: "a scrap item is special scrap",
        outcome: "it spawns only when its item-specific conditions are met instead of using the moon's scrap pool",
    },
    ScrapEconomyRule {
        condition: "a scrap item is conductive",
        outcome: "it can be struck by lightning during stormy weather",
    },
    ScrapEconomyRule {
        condition: "the carried scrap weight increases",
        outcome: "movement slows and stamina depletes faster while stamina regeneration slows",
    },
    ScrapEconomyRule {
        condition: "a scrap item is two-handed",
        outcome: "it prevents use of other inventory slots until dropped",
    },
    ScrapEconomyRule {
        condition: "the whole crew dies or goes missing on a moon",
        outcome: "all scrap is lost, including items with alternative uses such as stop_sign and double_barrel",
    },
    ScrapEconomyRule {
        condition: "old_phone is picked up or equipped",
        outcome: "it plays a woman's scream followed by a long audible disconnect tone",
    },
    ScrapEconomyRule {
        condition: "old_phone is picked up or equipped",
        outcome: "it has no known functional effect",
    },
    ScrapEconomyRule {
        condition: "old_phone is collected",
        outcome: "it can be sold for credits",
    },
];

pub struct ScrapEconomyPlugin;

impl Plugin for ScrapEconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScrapEconomyState>()
            .add_event::<ScrapEconomyCollectItemEvent>()
            .add_event::<ScrapEconomyDropHeldItemEvent>()
            .add_event::<ScrapEconomySellHeldScrapEvent>()
            .add_event::<ScrapEconomyCrewLostEvent>()
            .add_event::<ScrapEconomyMidnightShipDepartureEvent>()
            .add_event::<ScrapEconomyInventoryChangedEvent>()
            .add_event::<ScrapEconomyScrapLostEvent>()
            .add_event::<ScrapEconomyQuotaProgressChangedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    scrap_economy_collect_items,
                    scrap_economy_drop_held_items,
                    scrap_economy_sell_held_scrap,
                    scrap_economy_apply_crew_loss,
                    scrap_economy_apply_midnight_departure,
                    scrap_economy_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrapEconomyRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrapEconomySpawnChance {
    pub moon: &'static str,
    pub chance_basis_points: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrapEconomyItemClass {
    Scrap,
    Store,
    Weapon,
    Special,
}

impl ScrapEconomyItemClass {
    pub const fn is_lost_on_crew_wipe(self) -> bool {
        matches!(self, Self::Scrap)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrapEconomyItemLocation {
    World,
    CarriedByEmployee(u64),
    Ship,
    Sold,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrapEconomyLossReason {
    EntireCrewDied,
    MidnightDepartureNoCrewOnboard,
    CrewMissingOnMoon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrapEconomyItemRecord {
    pub stable_id: u64,
    pub item_id: &'static str,
    pub class: ScrapEconomyItemClass,
    pub value: I64F64,
    pub weight_lbs: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
    pub location: ScrapEconomyItemLocation,
}

impl ScrapEconomyItemRecord {
    pub const fn old_phone(stable_id: u64, value: I64F64) -> Self {
        Self {
            stable_id,
            item_id: OLD_PHONE_ID,
            class: ScrapEconomyItemClass::Scrap,
            value,
            weight_lbs: OLD_PHONE_WEIGHT,
            conductive: OLD_PHONE_CONDUCTIVE,
            two_handed: OLD_PHONE_TWO_HANDED,
            location: ScrapEconomyItemLocation::World,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ScrapEconomyState {
    pub items: BTreeMap<u64, ScrapEconomyItemRecord>,
    pub collected_scrap_count: u64,
    pub held_scrap_count: u64,
    pub ship_scrap_count: u64,
    pub sold_scrap_count: u64,
    pub lost_scrap_count: u64,
    pub special_item_count: u64,
    pub collected_scrap_value: I64F64,
    pub carried_scrap_weight_lbs: I32F32,
    pub ship_scrap_value: I64F64,
    pub sold_scrap_value_this_cycle: I64F64,
    pub quota_progress_value: I64F64,
    pub last_employee_id: u64,
    pub last_item_stable_id: u64,
    pub last_item_id: &'static str,
    pub last_loss_reason: Option<ScrapEconomyLossReason>,
}

impl Default for ScrapEconomyState {
    fn default() -> Self {
        Self {
            items: BTreeMap::new(),
            collected_scrap_count: 0,
            held_scrap_count: 0,
            ship_scrap_count: 0,
            sold_scrap_count: 0,
            lost_scrap_count: 0,
            special_item_count: 0,
            collected_scrap_value: I64F64::ZERO,
            carried_scrap_weight_lbs: I32F32::ZERO,
            ship_scrap_value: I64F64::ZERO,
            sold_scrap_value_this_cycle: I64F64::ZERO,
            quota_progress_value: I64F64::ZERO,
            last_employee_id: 0,
            last_item_stable_id: 0,
            last_item_id: "",
            last_loss_reason: None,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomyCollectItemEvent {
    pub employee_id: u64,
    pub stable_id: u64,
    pub item_id: &'static str,
    pub class: ScrapEconomyItemClass,
    pub value: I64F64,
    pub weight_lbs: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
    pub from_store_or_valueless: bool,
    pub functional: bool,
    pub passive: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomyDropHeldItemEvent {
    pub employee_id: u64,
    pub stable_id: u64,
    pub dropped_inside_ship: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomySellHeldScrapEvent {
    pub employee_id: u64,
    pub stable_id: u64,
    pub days_remaining_before_quota_deadline: u8,
    pub can_land_at_company: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomyCrewLostEvent {
    pub reason: ScrapEconomyLossReason,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomyMidnightShipDepartureEvent {
    pub crew_members_onboard: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomyInventoryChangedEvent {
    pub employee_id: u64,
    pub stable_id: u64,
    pub item_id: &'static str,
    pub location: ScrapEconomyItemLocation,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomyScrapLostEvent {
    pub stable_id: u64,
    pub item_id: &'static str,
    pub value: I64F64,
    pub reason: ScrapEconomyLossReason,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapEconomyQuotaProgressChangedEvent {
    pub quota_progress_value: I64F64,
    pub current_quota: I64F64,
}

pub fn scrap_economy_old_phone_collect_event(
    employee_id: u64,
    stable_id: u64,
    value: I64F64,
) -> ScrapEconomyCollectItemEvent {
    ScrapEconomyCollectItemEvent {
        employee_id,
        stable_id,
        item_id: OLD_PHONE_ID,
        class: ScrapEconomyItemClass::Scrap,
        value,
        weight_lbs: OLD_PHONE_WEIGHT,
        conductive: OLD_PHONE_CONDUCTIVE,
        two_handed: OLD_PHONE_TWO_HANDED,
        from_store_or_valueless: false,
        functional: false,
        passive: true,
    }
}

pub fn scrap_economy_old_phone_value_in_range(value: I64F64) -> bool {
    value >= OLD_PHONE_MIN_VALUE && value <= OLD_PHONE_MAX_VALUE
}

pub fn scrap_economy_spawn_chance_basis_points(item_id: &str, moon: &str) -> Option<u16> {
    if item_id != OLD_PHONE_ID {
        return None;
    }

    OLD_PHONE_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance_basis_points)
}

fn scrap_economy_collect_items(
    mut collect_events: EventReader<ScrapEconomyCollectItemEvent>,
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    mut changed_events: EventWriter<ScrapEconomyInventoryChangedEvent>,
    mut state: ResMut<ScrapEconomyState>,
) {
    for event in collect_events.read() {
        let record = ScrapEconomyItemRecord {
            stable_id: event.stable_id,
            item_id: event.item_id,
            class: event.class,
            value: event.value,
            weight_lbs: event.weight_lbs,
            conductive: event.conductive,
            two_handed: event.two_handed,
            location: ScrapEconomyItemLocation::CarriedByEmployee(event.employee_id),
        };

        let previous = state.items.insert(event.stable_id, record);
        if let Some(previous_record) = previous {
            remove_record_totals(&mut state, previous_record);
        }

        apply_record_totals(&mut state, record);
        state.collected_scrap_count = state.collected_scrap_count.wrapping_add(1);
        state.collected_scrap_value += event.value;
        state.last_employee_id = event.employee_id;
        state.last_item_stable_id = event.stable_id;
        state.last_item_id = event.item_id;

        pickup_events.send(ItemBarPickupEvent {
            employee_id: event.employee_id,
            item_id: event.item_id,
            from_store_or_valueless: event.from_store_or_valueless,
            two_handed: event.two_handed,
            functional: event.functional,
            passive: event.passive,
        });

        changed_events.send(ScrapEconomyInventoryChangedEvent {
            employee_id: event.employee_id,
            stable_id: event.stable_id,
            item_id: event.item_id,
            location: ScrapEconomyItemLocation::CarriedByEmployee(event.employee_id),
        });
    }
}

fn scrap_economy_drop_held_items(
    mut drop_events: EventReader<ScrapEconomyDropHeldItemEvent>,
    mut item_bar_drop_events: EventWriter<ItemBarDropHeldItemEvent>,
    mut changed_events: EventWriter<ScrapEconomyInventoryChangedEvent>,
    mut state: ResMut<ScrapEconomyState>,
) {
    for event in drop_events.read() {
        let Some(mut record) = state.items.get(&event.stable_id).copied() else {
            continue;
        };

        remove_record_totals(&mut state, record);
        record.location = if event.dropped_inside_ship {
            ScrapEconomyItemLocation::Ship
        } else {
            ScrapEconomyItemLocation::World
        };
        state.items.insert(event.stable_id, record);
        apply_record_totals(&mut state, record);

        state.last_employee_id = event.employee_id;
        state.last_item_stable_id = event.stable_id;
        state.last_item_id = record.item_id;

        item_bar_drop_events.send(ItemBarDropHeldItemEvent {
            employee_id: event.employee_id,
        });

        changed_events.send(ScrapEconomyInventoryChangedEvent {
            employee_id: event.employee_id,
            stable_id: event.stable_id,
            item_id: record.item_id,
            location: record.location,
        });
    }
}

fn scrap_economy_sell_held_scrap(
    mut sell_requests: EventReader<ScrapEconomySellHeldScrapEvent>,
    mut sell_events: EventWriter<SellScrapForCreditsEvent>,
    mut quota_events: EventWriter<FulfillProfitQuotaEvent>,
    mut progress_events: EventWriter<ScrapEconomyQuotaProgressChangedEvent>,
    quota_state: Res<ProfitQuotaState>,
    mut state: ResMut<ScrapEconomyState>,
) {
    for event in sell_requests.read() {
        let Some(mut record) = state.items.get(&event.stable_id).copied() else {
            continue;
        };

        if record.class != ScrapEconomyItemClass::Scrap {
            continue;
        }

        if matches!(record.location, ScrapEconomyItemLocation::Sold | ScrapEconomyItemLocation::Lost)
        {
            continue;
        }

        remove_record_totals(&mut state, record);
        record.location = ScrapEconomyItemLocation::Sold;
        state.items.insert(event.stable_id, record);
        apply_record_totals(&mut state, record);

        state.sold_scrap_value_this_cycle += record.value;
        state.quota_progress_value = state.sold_scrap_value_this_cycle;
        state.last_employee_id = event.employee_id;
        state.last_item_stable_id = event.stable_id;
        state.last_item_id = record.item_id;

        sell_events.send(SellScrapForCreditsEvent {
            scrap_entity_id: record.stable_id,
            base_credit_value: record.value,
            days_remaining_before_quota_deadline: event.days_remaining_before_quota_deadline,
        });

        quota_events.send(FulfillProfitQuotaEvent {
            quota_fulfilled: state.quota_progress_value,
            can_land_at_company: event.can_land_at_company,
        });

        progress_events.send(ScrapEconomyQuotaProgressChangedEvent {
            quota_progress_value: state.quota_progress_value,
            current_quota: quota_state.current_quota,
        });
    }
}

fn scrap_economy_apply_crew_loss(
    mut loss_events: EventReader<ScrapEconomyCrewLostEvent>,
    mut lost_events: EventWriter<ScrapEconomyScrapLostEvent>,
    mut state: ResMut<ScrapEconomyState>,
) {
    for event in loss_events.read() {
        lose_scrap_items(event.reason, &mut lost_events, &mut state);
    }
}

fn scrap_economy_apply_midnight_departure(
    mut departure_events: EventReader<ScrapEconomyMidnightShipDepartureEvent>,
    mut lost_events: EventWriter<ScrapEconomyScrapLostEvent>,
    mut state: ResMut<ScrapEconomyState>,
) {
    for event in departure_events.read() {
        if event.crew_members_onboard != 0 {
            continue;
        }

        lose_scrap_items(
            ScrapEconomyLossReason::MidnightDepartureNoCrewOnboard,
            &mut lost_events,
            &mut state,
        );
    }
}

fn lose_scrap_items(
    reason: ScrapEconomyLossReason,
    lost_events: &mut EventWriter<ScrapEconomyScrapLostEvent>,
    state: &mut ScrapEconomyState,
) {
    let stable_ids: Vec<u64> = state
        .items
        .iter()
        .filter_map(|(stable_id, record)| {
            if record.class.is_lost_on_crew_wipe()
                && !matches!(
                    record.location,
                    ScrapEconomyItemLocation::Sold | ScrapEconomyItemLocation::Lost
                )
            {
                Some(*stable_id)
            } else {
                None
            }
        })
        .collect();

    for stable_id in stable_ids {
        let Some(mut record) = state.items.get(&stable_id).copied() else {
            continue;
        };

        remove_record_totals(state, record);
        record.location = ScrapEconomyItemLocation::Lost;
        state.items.insert(stable_id, record);
        apply_record_totals(state, record);

        state.last_item_stable_id = stable_id;
        state.last_item_id = record.item_id;
        state.last_loss_reason = Some(reason);

        lost_events.send(ScrapEconomyScrapLostEvent {
            stable_id,
            item_id: record.item_id,
            value: record.value,
            reason,
        });
    }
}

fn apply_record_totals(state: &mut ScrapEconomyState, record: ScrapEconomyItemRecord) {
    match record.location {
        ScrapEconomyItemLocation::CarriedByEmployee(_) => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.held_scrap_count = state.held_scrap_count.wrapping_add(1);
                state.carried_scrap_weight_lbs += record.weight_lbs;
            }
        }
        ScrapEconomyItemLocation::Ship => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.ship_scrap_count = state.ship_scrap_count.wrapping_add(1);
                state.ship_scrap_value += record.value;
            }
        }
        ScrapEconomyItemLocation::Sold => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.sold_scrap_count = state.sold_scrap_count.wrapping_add(1);
            }
        }
        ScrapEconomyItemLocation::Lost => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.lost_scrap_count = state.lost_scrap_count.wrapping_add(1);
            }
        }
        ScrapEconomyItemLocation::World => {}
    }

    if record.class == ScrapEconomyItemClass::Special {
        state.special_item_count = state.special_item_count.wrapping_add(1);
    }
}

fn remove_record_totals(state: &mut ScrapEconomyState, record: ScrapEconomyItemRecord) {
    match record.location {
        ScrapEconomyItemLocation::CarriedByEmployee(_) => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.held_scrap_count = state.held_scrap_count.saturating_sub(1);
                state.carried_scrap_weight_lbs -= record.weight_lbs;
            }
        }
        ScrapEconomyItemLocation::Ship => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.ship_scrap_count = state.ship_scrap_count.saturating_sub(1);
                state.ship_scrap_value -= record.value;
            }
        }
        ScrapEconomyItemLocation::Sold => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.sold_scrap_count = state.sold_scrap_count.saturating_sub(1);
            }
        }
        ScrapEconomyItemLocation::Lost => {
            if record.class == ScrapEconomyItemClass::Scrap {
                state.lost_scrap_count = state.lost_scrap_count.saturating_sub(1);
            }
        }
        ScrapEconomyItemLocation::World => {}
    }

    if record.class == ScrapEconomyItemClass::Special {
        state.special_item_count = state.special_item_count.saturating_sub(1);
    }
}

fn scrap_economy_checksum(
    tick: Res<SimTick>,
    state: Res<ScrapEconomyState>,
    mut checksum: ResMut<SimChecksumState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(state.collected_scrap_count);
    checksum.accumulate(state.held_scrap_count);
    checksum.accumulate(state.ship_scrap_count);
    checksum.accumulate(state.sold_scrap_count);
    checksum.accumulate(state.lost_scrap_count);
    checksum.accumulate(state.special_item_count);
    checksum.accumulate(state.collected_scrap_value.to_bits() as u64);
    checksum.accumulate(state.carried_scrap_weight_lbs.to_bits() as u64);
    checksum.accumulate(state.ship_scrap_value.to_bits() as u64);
    checksum.accumulate(state.sold_scrap_value_this_cycle.to_bits() as u64);
    checksum.accumulate(state.quota_progress_value.to_bits() as u64);
    checksum.accumulate(state.last_employee_id);
    checksum.accumulate(state.last_item_stable_id);

    for (stable_id, record) in &state.items {
        checksum.accumulate(*stable_id);
        checksum.accumulate(record.value.to_bits() as u64);
        checksum.accumulate(record.weight_lbs.to_bits() as u64);
        checksum.accumulate(record.conductive as u64);
        checksum.accumulate(record.two_handed as u64);
        checksum.accumulate(match record.class {
            ScrapEconomyItemClass::Scrap => 1,
            ScrapEconomyItemClass::Store => 2,
            ScrapEconomyItemClass::Weapon => 3,
            ScrapEconomyItemClass::Special => 4,
        });
        checksum.accumulate(match record.location {
            ScrapEconomyItemLocation::World => 1,
            ScrapEconomyItemLocation::CarriedByEmployee(employee_id) => 0x1000_0000 ^ employee_id,
            ScrapEconomyItemLocation::Ship => 2,
            ScrapEconomyItemLocation::Sold => 3,
            ScrapEconomyItemLocation::Lost => 4,
        });
    }
}