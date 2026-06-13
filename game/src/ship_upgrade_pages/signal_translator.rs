// Sources: vault/ship_upgrade_pages/signal_translator.md

use bevy::prelude::*;
use crate::sim::SimChecksumState;

pub const SIGNAL_TRANSLATOR_BUY_COST: u32 = 255;
pub const SIGNAL_TRANSLATOR_MAX_MESSAGE_LEN: usize = 9;
/// Luck modifier scaled by 1000: represents -0.012
pub const SIGNAL_TRANSLATOR_LUCK_MODIFIER_SCALED: i32 = -12;

#[derive(Component, Default)]
pub struct SignalTranslator {
    pub transmitting: bool,
    pub message: [u8; SIGNAL_TRANSLATOR_MAX_MESSAGE_LEN],
    pub message_len: u8,
}

#[derive(Bundle, Default)]
pub struct SignalTranslatorBundle {
    pub signal_translator: SignalTranslator,
}

pub struct SignalTranslatorPlugin;

impl Plugin for SignalTranslatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, signal_translator_checksum_system);
    }
}

fn signal_translator_checksum_system(
    query: Query<&SignalTranslator>,
    mut checksum: ResMut<SimChecksumState>,
) {
    for st in query.iter() {
        checksum.accumulate(st.transmitting as u64);
        checksum.accumulate(st.message_len as u64);
        for byte in st.message.iter() {
            checksum.accumulate(*byte as u64);
        }
    }
}