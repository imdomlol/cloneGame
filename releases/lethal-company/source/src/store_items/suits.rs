// Sources: vault/store_items/suits.md, vault/item_index_pages/items.md

use bevy::prelude::*;

use crate::sim::SimChecksumState;

pub const SUIT_COUNT: usize = 8;
pub const PURCHASABLE_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SuitKind {
    Decoy = 0,
    Brown = 1,
    Green = 2,
    Purple = 3,
    Hazard = 4,
    Bee = 5,
    Bunny = 6,
    Pajama = 7,
}

impl SuitKind {
    pub const ALL: [SuitKind; SUIT_COUNT] = [
        SuitKind::Decoy,
        SuitKind::Brown,
        SuitKind::Green,
        SuitKind::Purple,
        SuitKind::Hazard,
        SuitKind::Bee,
        SuitKind::Bunny,
        SuitKind::Pajama,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            SuitKind::Decoy => "Decoy Suit",
            SuitKind::Brown => "Brown Suit",
            SuitKind::Green => "Green Suit",
            SuitKind::Purple => "Purple Suit",
            SuitKind::Hazard => "Hazard Suit",
            SuitKind::Bee => "Bee Suit",
            SuitKind::Bunny => "Bunny Suit",
            SuitKind::Pajama => "Pajama Suit",
        }
    }

    pub const fn cost(self) -> u32 {
        match self {
            SuitKind::Decoy => 0,
            SuitKind::Brown => 0,
            SuitKind::Green => 60,
            SuitKind::Purple => 70,
            SuitKind::Hazard => 90,
            SuitKind::Bee => 110,
            SuitKind::Bunny => 200,
            SuitKind::Pajama => 900,
        }
    }

    pub const fn is_default(self) -> bool {
        matches!(self, SuitKind::Decoy | SuitKind::Brown)
    }

    pub const fn is_purchasable(self) -> bool {
        !self.is_default()
    }
}

pub const CHALLENGE_MOON_DEFAULT_SUIT: SuitKind = SuitKind::Purple;

#[derive(Event)]
pub struct SuitPurchasedEvent {
    pub suit: SuitKind,
}

#[derive(Event)]
pub struct SuitEquippedEvent {
    pub suit: SuitKind,
}

#[derive(Event)]
pub struct SuitRunEndedEvent;

#[derive(Event)]
pub struct SuitJumpedEvent {
    pub suit: SuitKind,
}

#[derive(Event)]
pub struct BunnySuitHopSoundEvent;

#[derive(Resource)]
pub struct SuitsState {
    pub owned: [bool; SUIT_COUNT],
    pub equipped: SuitKind,
}

impl Default for SuitsState {
    fn default() -> Self {
        let mut owned = [false; SUIT_COUNT];
        owned[SuitKind::Decoy.index()] = true;
        owned[SuitKind::Brown.index()] = true;

        Self {
            owned,
            equipped: SuitKind::Decoy,
        }
    }
}

impl SuitsState {
    pub fn owns(&self, suit: SuitKind) -> bool {
        self.owned[suit.index()]
    }
}

pub struct SuitsPlugin;

impl Plugin for SuitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SuitPurchasedEvent>()
            .add_event::<SuitEquippedEvent>()
            .add_event::<SuitRunEndedEvent>()
            .add_event::<SuitJumpedEvent>()
            .add_event::<BunnySuitHopSoundEvent>()
            .init_resource::<SuitsState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_suit_purchases,
                    handle_suit_equips,
                    handle_suit_run_end,
                    handle_suit_jumps,
                    suits_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_suit_purchases(
    mut events: EventReader<SuitPurchasedEvent>,
    mut state: ResMut<SuitsState>,
) {
    for event in events.read() {
        state.owned[event.suit.index()] = true;
    }
}

fn handle_suit_equips(mut events: EventReader<SuitEquippedEvent>, mut state: ResMut<SuitsState>) {
    for event in events.read() {
        if state.owns(event.suit) {
            state.equipped = event.suit;
        }
    }
}

fn handle_suit_run_end(mut events: EventReader<SuitRunEndedEvent>, mut state: ResMut<SuitsState>) {
    for _ in events.read() {
        *state = SuitsState::default();
    }
}

fn handle_suit_jumps(
    mut events: EventReader<SuitJumpedEvent>,
    state: Res<SuitsState>,
    mut hop_events: EventWriter<BunnySuitHopSoundEvent>,
) {
    for event in events.read() {
        if event.suit == SuitKind::Bunny && state.equipped == SuitKind::Bunny {
            hop_events.send(BunnySuitHopSoundEvent);
        }
    }
}

fn suits_checksum(state: Res<SuitsState>, mut cs: ResMut<SimChecksumState>) {
    for suit in SuitKind::ALL {
        cs.accumulate(state.owned[suit.index()] as u64);
    }

    cs.accumulate(state.equipped as u64);
}