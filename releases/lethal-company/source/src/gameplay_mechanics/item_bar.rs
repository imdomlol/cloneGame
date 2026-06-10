// Sources: vault/gameplay_mechanics/item_bar.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const ITEM_BAR_ID: &str = "item_bar";
pub const ITEM_BAR_NAME: &str = "Item Bar";
pub const ITEM_BAR_TYPE: &str = "gameplay_mechanics";
pub const ITEM_BAR_SUBTYPE: &str = "ui";
pub const ITEM_BAR_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Item_Bar";
pub const ITEM_BAR_SOURCE_REVISION: u32 = 20881;
pub const ITEM_BAR_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const ITEM_BAR_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const ITEM_BAR_BLUE_SLOT_COUNT: usize = 4;
pub const ITEM_BAR_YELLOW_SLOT_COUNT: usize = 1;
pub const ITEM_BAR_SHOVEL_ID: &str = "shovel";
pub const ITEM_BAR_TOY_ROBOT_ID: &str = "toy_robot";
pub const ITEM_BAR_KEY_ID: &str = "key";

pub const ITEM_BAR_DEPENDS_ON: [&str; 28] = [
    "items",
    "employee",
    "earth_leviathan",
    "forest_keeper",
    "masked",
    "weather",
    "store",
    "shovel",
    "airhorn",
    "toy_robot",
    "fancy_lamp",
    "key",
    "the_company",
    "the_ship",
    "main_entrance",
    "fire_exit",
    "regular_door",
    "company_bell",
    "steam_valve",
    "fusebox",
    "item_cupboard",
    "locked_door",
    "suit_rack",
    "ship_door",
    "ship_lever",
    "ship_monitor",
    "ship_terminal",
    "teleporter",
];

pub const ITEM_BAR_RULES: [&str; 8] = [
    "The item bar shows four blue inventory slots at the bottom of the screen.",
    "The item bar includes one yellow slot in the top-left for most store items or other objects with no value, except the shovel.",
    "Selecting items is done by scrolling up or down.",
    "Interacting with objects is done with E.",
    "Dropping the currently held item is done with G.",
    "If an employee dies, then all carried items are normally dropped.",
    "If an employee is killed by an earth_leviathan, forest_keeper, or masked, then carried items are deleted instead of dropped.",
    "If an employee sinks in quicksand during rainy weather, then carried items are deleted instead of dropped.",
];

pub const ITEM_BAR_MODIFIERS: [&str; 6] = [
    "Functional items such as airhorn can be activated for their effect.",
    "Passive items such as fancy_lamp can emit light.",
    "A toy_robot can emit noise when picked up.",
    "A toy_robot can only be silenced by repeatedly dropping and picking it up again.",
    "A key disappears after it is used appropriately.",
    "Switching away from an item can stop that item's special effect.",
];

pub const ITEM_BAR_STRATEGY: [&str; 3] = [
    "Use the yellow slot for non-shovel utility items that do not need a blue slot.",
    "Keep critical carried items out of death conditions that delete inventory instead of dropping it.",
    "Expect two-handed items to block most interactions until they are dropped.",
];

pub const ITEM_BAR_NOTES: [&str; 17] = [
    "When the employee picks up a two-handed item, the UI shows a HANDS FULL prompt.",
    "While holding a two-handed item, the employee can still enter or exit via the main_entrance or fire_exit.",
    "While holding a two-handed item, the employee can still open or close unlocked regular_door.",
    "While holding a two-handed item, the employee can still ring the company_bell at the_company.",
    "While holding a two-handed item, the employee can still shut off the steam_valve.",
    "While holding a two-handed item, the employee can still toggle the the_ship lights.",
    "While holding a two-handed item, the employee cannot interact with the fusebox.",
    "While holding a two-handed item, the employee cannot open or close the item_cupboard.",
    "While holding a two-handed item, the employee cannot unlock locked doors.",
    "While holding a two-handed item, the employee cannot pick up additional items even when inventory space is available.",
    "While holding a two-handed item, the employee cannot change suits at the suit_rack.",
    "While holding a two-handed item, the employee cannot open or close the the_ship doors.",
    "While holding a two-handed item, the employee cannot pull the the_ship lever.",
    "While holding a two-handed item, the employee cannot enable, disable, or switch the the_ship monitor.",
    "While holding a two-handed item, the employee cannot use the the_ship terminal.",
    "While holding a two-handed item, the employee cannot activate the teleporters if they were purchased.",
    "The Item Bar is the inventory UI for an employee's held items.",
];

pub const ITEM_BAR_BEHAVIORAL_MECHANICS: [ItemBarBehaviorRule; 29] = [
    ItemBarBehaviorRule {
        condition: "the player scrolls up",
        outcome: "selection moves left through the inventory bar",
    },
    ItemBarBehaviorRule {
        condition: "the player scrolls down",
        outcome: "selection moves right through the inventory bar",
    },
    ItemBarBehaviorRule {
        condition: "the player presses E",
        outcome: "they pick up or interact with an object",
    },
    ItemBarBehaviorRule {
        condition: "the player presses G",
        outcome: "they drop the currently held item",
    },
    ItemBarBehaviorRule {
        condition: "an employee dies",
        outcome: "carried items normally drop",
    },
    ItemBarBehaviorRule {
        condition: "an employee is killed by an earth_leviathan, forest_keeper, or masked",
        outcome: "carried items are deleted instead of dropped",
    },
    ItemBarBehaviorRule {
        condition: "an employee sinks in quicksand during rainy weather",
        outcome: "carried items are deleted instead of dropped",
    },
    ItemBarBehaviorRule {
        condition: "an item is functional",
        outcome: "it can be activated for its effect, such as airhorn noise",
    },
    ItemBarBehaviorRule {
        condition: "an item is passive",
        outcome: "it can continue producing effects such as fancy_lamp light",
    },
    ItemBarBehaviorRule {
        condition: "a toy_robot is picked up",
        outcome: "it can emit noise",
    },
    ItemBarBehaviorRule {
        condition: "a toy_robot is to be silenced",
        outcome: "it must be dropped and picked up repeatedly",
    },
    ItemBarBehaviorRule {
        condition: "a key is used appropriately",
        outcome: "it disappears",
    },
    ItemBarBehaviorRule {
        condition: "the player switches away from an item",
        outcome: "that item's special effect may stop",
    },
    ItemBarBehaviorRule {
        condition: "an employee picks up a two-handed item",
        outcome: "a HANDS FULL prompt appears",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the main_entrance or fire_exit",
        outcome: "they can still enter or exit",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses unlocked regular doors",
        outcome: "they can still open or close them",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and rings the company_bell at the_company",
        outcome: "the action still works",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the steam_valve",
        outcome: "they can still shut it off",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the_ship",
        outcome: "they can still toggle the ship's lights",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the fusebox",
        outcome: "the interaction is blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the item_cupboard",
        outcome: "opening or closing it is blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and tries to unlock locked doors",
        outcome: "the unlock action is blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and tries to pick up more items",
        outcome: "pickup is blocked even when inventory space exists",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the suit_rack",
        outcome: "suit changes are blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the_ship doors",
        outcome: "opening or closing them is blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the_ship lever",
        outcome: "launching or landing the ship is blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the_ship monitor",
        outcome: "enabling, disabling, or switching monitors is blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the_ship terminal",
        outcome: "terminal use is blocked",
    },
    ItemBarBehaviorRule {
        condition: "an employee is holding a two-handed item and uses the teleporters",
        outcome: "activation is blocked if they were purchased",
    },
];

pub const ITEM_BAR_TWO_HANDED_ALLOWED_INTERACTIONS: [&str; 6] = [
    "main_entrance",
    "fire_exit",
    "regular_door",
    "company_bell",
    "steam_valve",
    "the_ship_lights",
];

pub const ITEM_BAR_TWO_HANDED_BLOCKED_INTERACTIONS: [&str; 10] = [
    "fusebox",
    "item_cupboard",
    "locked_door",
    "items",
    "suit_rack",
    "ship_door",
    "ship_lever",
    "ship_monitor",
    "ship_terminal",
    "teleporter",
];

pub const ITEM_BAR_DELETE_DEATH_CAUSES: [&str; 3] = ["earth_leviathan", "forest_keeper", "masked"];

pub struct ItemBarPlugin;

impl Plugin for ItemBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemBarState>()
            .add_event::<ItemBarScrollEvent>()
            .add_event::<ItemBarPickupEvent>()
            .add_event::<ItemBarInteractEvent>()
            .add_event::<ItemBarDropHeldItemEvent>()
            .add_event::<ItemBarEmployeeDeathInventoryEvent>()
            .add_event::<ItemBarInventoryResolvedEvent>()
            .add_event::<ItemBarItemEffectEvent>()
            .add_event::<ItemBarTwoHandedPromptEvent>()
            .add_systems(
                FixedUpdate,
                (
                    item_bar_apply_scroll,
                    item_bar_apply_pickups,
                    item_bar_apply_interactions,
                    item_bar_apply_drop_requests,
                    item_bar_apply_death_inventory,
                    item_bar_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemBarBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemBarScrollDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemBarSlotKind {
    Blue,
    Yellow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemBarInventoryOutcome {
    Dropped,
    Deleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemBarInteractionResult {
    Allowed,
    BlockedByTwoHandedItem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemBarItemEffect {
    FunctionalActivated,
    PassiveContinues,
    ToyRobotNoise,
    ToyRobotSilenceAttempt,
    KeyConsumed,
    SwitchedAwayStopped,
    HandsFullPrompt,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ItemBarState {
    pub selected_blue_slot: usize,
    pub blue_slots: [Option<&'static str>; ITEM_BAR_BLUE_SLOT_COUNT],
    pub yellow_slot: Option<&'static str>,
    pub held_two_handed_item: bool,
    pub scrolls_applied: u64,
    pub pickups_applied: u64,
    pub interactions_attempted: u64,
    pub interactions_blocked: u64,
    pub drops_requested: u64,
    pub death_inventory_resolutions: u64,
    pub carried_items_dropped: u64,
    pub carried_items_deleted: u64,
    pub item_effects_emitted: u64,
    pub hands_full_prompts: u64,
    pub last_employee_id: u64,
    pub last_item_id: &'static str,
    pub last_interaction_id: &'static str,
    pub last_death_cause_id: &'static str,
    pub last_inventory_outcome: Option<ItemBarInventoryOutcome>,
}

impl Default for ItemBarState {
    fn default() -> Self {
        Self {
            selected_blue_slot: 0,
            blue_slots: [None; ITEM_BAR_BLUE_SLOT_COUNT],
            yellow_slot: None,
            held_two_handed_item: false,
            scrolls_applied: 0,
            pickups_applied: 0,
            interactions_attempted: 0,
            interactions_blocked: 0,
            drops_requested: 0,
            death_inventory_resolutions: 0,
            carried_items_dropped: 0,
            carried_items_deleted: 0,
            item_effects_emitted: 0,
            hands_full_prompts: 0,
            last_employee_id: 0,
            last_item_id: "",
            last_interaction_id: "",
            last_death_cause_id: "",
            last_inventory_outcome: None,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarScrollEvent {
    pub employee_id: u64,
    pub direction: ItemBarScrollDirection,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarPickupEvent {
    pub employee_id: u64,
    pub item_id: &'static str,
    pub from_store_or_valueless: bool,
    pub two_handed: bool,
    pub functional: bool,
    pub passive: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarInteractEvent {
    pub employee_id: u64,
    pub interaction_id: &'static str,
    pub held_item_id: Option<&'static str>,
    pub use_key: bool,
    pub toy_robot_silence_attempt: bool,
    pub switching_away_from_item: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarDropHeldItemEvent {
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarEmployeeDeathInventoryEvent {
    pub employee_id: u64,
    pub cause_id: &'static str,
    pub rainy_quicksand: bool,
    pub carried_item_count: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarInventoryResolvedEvent {
    pub employee_id: u64,
    pub outcome: ItemBarInventoryOutcome,
    pub item_count: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarItemEffectEvent {
    pub employee_id: u64,
    pub item_id: &'static str,
    pub effect: ItemBarItemEffect,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBarTwoHandedPromptEvent {
    pub employee_id: u64,
    pub item_id: &'static str,
}

pub fn item_bar_blue_slot_count() -> usize {
    ITEM_BAR_BLUE_SLOT_COUNT
}

pub fn item_bar_yellow_slot_count() -> usize {
    ITEM_BAR_YELLOW_SLOT_COUNT
}

pub fn item_bar_uses_yellow_slot(item_id: &str, from_store_or_valueless: bool) -> bool {
    from_store_or_valueless && item_id != ITEM_BAR_SHOVEL_ID
}

pub fn item_bar_death_deletes_inventory(cause_id: &str, rainy_quicksand: bool) -> bool {
    rainy_quicksand || ITEM_BAR_DELETE_DEATH_CAUSES.contains(&cause_id)
}

pub fn item_bar_two_handed_interaction_result(interaction_id: &str) -> ItemBarInteractionResult {
    if ITEM_BAR_TWO_HANDED_ALLOWED_INTERACTIONS.contains(&interaction_id) {
        ItemBarInteractionResult::Allowed
    } else if ITEM_BAR_TWO_HANDED_BLOCKED_INTERACTIONS.contains(&interaction_id) {
        ItemBarInteractionResult::BlockedByTwoHandedItem
    } else {
        ItemBarInteractionResult::Allowed
    }
}

fn item_bar_apply_scroll(
    mut scroll_events: EventReader<ItemBarScrollEvent>,
    mut state: ResMut<ItemBarState>,
) {
    for event in scroll_events.read() {
        state.scrolls_applied = state.scrolls_applied.wrapping_add(1);
        state.last_employee_id = event.employee_id;

        state.selected_blue_slot = match event.direction {
            ItemBarScrollDirection::Up => {
                if state.selected_blue_slot == 0 {
                    ITEM_BAR_BLUE_SLOT_COUNT - 1
                } else {
                    state.selected_blue_slot - 1
                }
            }
            ItemBarScrollDirection::Down => (state.selected_blue_slot + 1) % ITEM_BAR_BLUE_SLOT_COUNT,
        };
    }
}

fn item_bar_apply_pickups(
    mut pickup_events: EventReader<ItemBarPickupEvent>,
    mut effect_events: EventWriter<ItemBarItemEffectEvent>,
    mut prompt_events: EventWriter<ItemBarTwoHandedPromptEvent>,
    mut state: ResMut<ItemBarState>,
) {
    for event in pickup_events.read() {
        state.pickups_applied = state.pickups_applied.wrapping_add(1);
        state.last_employee_id = event.employee_id;
        state.last_item_id = event.item_id;

        if item_bar_uses_yellow_slot(event.item_id, event.from_store_or_valueless) {
            state.yellow_slot = Some(event.item_id);
        } else if let Some(slot) = state.blue_slots.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(event.item_id);
        }

        if event.two_handed {
            state.held_two_handed_item = true;
            state.hands_full_prompts = state.hands_full_prompts.wrapping_add(1);
            state.item_effects_emitted = state.item_effects_emitted.wrapping_add(1);
            prompt_events.send(ItemBarTwoHandedPromptEvent {
                employee_id: event.employee_id,
                item_id: event.item_id,
            });
            effect_events.send(ItemBarItemEffectEvent {
                employee_id: event.employee_id,
                item_id: event.item_id,
                effect: ItemBarItemEffect::HandsFullPrompt,
            });
        }

        if event.functional {
            state.item_effects_emitted = state.item_effects_emitted.wrapping_add(1);
            effect_events.send(ItemBarItemEffectEvent {
                employee_id: event.employee_id,
                item_id: event.item_id,
                effect: ItemBarItemEffect::FunctionalActivated,
            });
        }

        if event.passive {
            state.item_effects_emitted = state.item_effects_emitted.wrapping_add(1);
            effect_events.send(ItemBarItemEffectEvent {
                employee_id: event.employee_id,
                item_id: event.item_id,
                effect: ItemBarItemEffect::PassiveContinues,
            });
        }

        if event.item_id == ITEM_BAR_TOY_ROBOT_ID {
            state.item_effects_emitted = state.item_effects_emitted.wrapping_add(1);
            effect_events.send(ItemBarItemEffectEvent {
                employee_id: event.employee_id,
                item_id: event.item_id,
                effect: ItemBarItemEffect::ToyRobotNoise,
            });
        }
    }
}

fn item_bar_apply_interactions(
    mut interact_events: EventReader<ItemBarInteractEvent>,
    mut effect_events: EventWriter<ItemBarItemEffectEvent>,
    mut state: ResMut<ItemBarState>,
) {
    for event in interact_events.read() {
        state.interactions_attempted = state.interactions_attempted.wrapping_add(1);
        state.last_employee_id = event.employee_id;
        state.last_interaction_id = event.interaction_id;

        if state.held_two_handed_item
            && item_bar_two_handed_interaction_result(event.interaction_id)
                == ItemBarInteractionResult::BlockedByTwoHandedItem
        {
            state.interactions_blocked = state.interactions_blocked.wrapping_add(1);
            continue;
        }

        if event.use_key {
            consume_item(&mut state, ITEM_BAR_KEY_ID);
            state.item_effects_emitted = state.item_effects_emitted.wrapping_add(1);
            effect_events.send(ItemBarItemEffectEvent {
                employee_id: event.employee_id,
                item_id: ITEM_BAR_KEY_ID,
                effect: ItemBarItemEffect::KeyConsumed,
            });
        }

        if event.toy_robot_silence_attempt {
            state.item_effects_emitted = state.item_effects_emitted.wrapping_add(1);
            effect_events.send(ItemBarItemEffectEvent {
                employee_id: event.employee_id,
                item_id: ITEM_BAR_TOY_ROBOT_ID,
                effect: ItemBarItemEffect::ToyRobotSilenceAttempt,
            });
        }

        if event.switching_away_from_item {
            if let Some(item_id) = event.held_item_id {
                state.item_effects_emitted = state.item_effects_emitted.wrapping_add(1);
                effect_events.send(ItemBarItemEffectEvent {
                    employee_id: event.employee_id,
                    item_id,
                    effect: ItemBarItemEffect::SwitchedAwayStopped,
                });
            }
        }
    }
}

fn item_bar_apply_drop_requests(
    mut drop_events: EventReader<ItemBarDropHeldItemEvent>,
    mut state: ResMut<ItemBarState>,
) {
    for event in drop_events.read() {
        state.drops_requested = state.drops_requested.wrapping_add(1);
        state.last_employee_id = event.employee_id;

        let selected = state.selected_blue_slot;
        if let Some(item_id) = state.blue_slots[selected] {
            state.last_item_id = item_id;
            state.blue_slots[selected] = None;
        } else if let Some(item_id) = state.yellow_slot {
            state.last_item_id = item_id;
            state.yellow_slot = None;
        }

        state.held_two_handed_item = false;
    }
}

fn item_bar_apply_death_inventory(
    mut death_events: EventReader<ItemBarEmployeeDeathInventoryEvent>,
    mut resolved_events: EventWriter<ItemBarInventoryResolvedEvent>,
    mut state: ResMut<ItemBarState>,
) {
    for event in death_events.read() {
        let outcome = if item_bar_death_deletes_inventory(event.cause_id, event.rainy_quicksand) {
            ItemBarInventoryOutcome::Deleted
        } else {
            ItemBarInventoryOutcome::Dropped
        };

        state.death_inventory_resolutions = state.death_inventory_resolutions.wrapping_add(1);
        state.last_employee_id = event.employee_id;
        state.last_death_cause_id = event.cause_id;
        state.last_inventory_outcome = Some(outcome);

        match outcome {
            ItemBarInventoryOutcome::Dropped => {
                state.carried_items_dropped = state
                    .carried_items_dropped
                    .wrapping_add(event.carried_item_count as u64);
            }
            ItemBarInventoryOutcome::Deleted => {
                state.carried_items_deleted = state
                    .carried_items_deleted
                    .wrapping_add(event.carried_item_count as u64);
            }
        }

        state.blue_slots = [None; ITEM_BAR_BLUE_SLOT_COUNT];
        state.yellow_slot = None;
        state.held_two_handed_item = false;

        resolved_events.send(ItemBarInventoryResolvedEvent {
            employee_id: event.employee_id,
            outcome,
            item_count: event.carried_item_count,
        });
    }
}

fn consume_item(state: &mut ItemBarState, item_id: &'static str) {
    for slot in &mut state.blue_slots {
        if *slot == Some(item_id) {
            *slot = None;
            return;
        }
    }

    if state.yellow_slot == Some(item_id) {
        state.yellow_slot = None;
    }
}

fn item_bar_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<ItemBarState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(ITEM_BAR_SOURCE_REVISION as u64);
    checksum.accumulate(ITEM_BAR_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(ITEM_BAR_BLUE_SLOT_COUNT as u64);
    checksum.accumulate(ITEM_BAR_YELLOW_SLOT_COUNT as u64);

    for dependency in ITEM_BAR_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in ITEM_BAR_RULES {
        accumulate_str(&mut checksum, 0x1100, rule);
    }

    for modifier in ITEM_BAR_MODIFIERS {
        accumulate_str(&mut checksum, 0x1200, modifier);
    }

    for strategy in ITEM_BAR_STRATEGY {
        accumulate_str(&mut checksum, 0x1300, strategy);
    }

    for note in ITEM_BAR_NOTES {
        accumulate_str(&mut checksum, 0x1400, note);
    }

    for rule in ITEM_BAR_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for interaction_id in ITEM_BAR_TWO_HANDED_ALLOWED_INTERACTIONS {
        accumulate_str(&mut checksum, 0x2100, interaction_id);
    }

    for interaction_id in ITEM_BAR_TWO_HANDED_BLOCKED_INTERACTIONS {
        accumulate_str(&mut checksum, 0x2200, interaction_id);
    }

    for cause_id in ITEM_BAR_DELETE_DEATH_CAUSES {
        accumulate_str(&mut checksum, 0x2300, cause_id);
    }

    checksum.accumulate(state.selected_blue_slot as u64);
    checksum.accumulate(state.held_two_handed_item as u64);
    checksum.accumulate(state.scrolls_applied);
    checksum.accumulate(state.pickups_applied);
    checksum.accumulate(state.interactions_attempted);
    checksum.accumulate(state.interactions_blocked);
    checksum.accumulate(state.drops_requested);
    checksum.accumulate(state.death_inventory_resolutions);
    checksum.accumulate(state.carried_items_dropped);
    checksum.accumulate(state.carried_items_deleted);
    checksum.accumulate(state.item_effects_emitted);
    checksum.accumulate(state.hands_full_prompts);
    checksum.accumulate(state.last_employee_id);

    for (index, slot) in state.blue_slots.iter().enumerate() {
        checksum.accumulate(0x3000 ^ index as u64);
        if let Some(item_id) = slot {
            accumulate_str(&mut checksum, 0x3001 ^ index as u64, item_id);
        }
    }

    if let Some(item_id) = state.yellow_slot {
        accumulate_str(&mut checksum, 0x3100, item_id);
    }

    accumulate_str(&mut checksum, 0x3200, state.last_item_id);
    accumulate_str(&mut checksum, 0x3201, state.last_interaction_id);
    accumulate_str(&mut checksum, 0x3202, state.last_death_cause_id);

    let outcome_bits = match state.last_inventory_outcome {
        Some(ItemBarInventoryOutcome::Dropped) => 1,
        Some(ItemBarInventoryOutcome::Deleted) => 2,
        None => 0,
    };
    checksum.accumulate(outcome_bits);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}