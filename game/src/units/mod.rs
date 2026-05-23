use bevy::prelude::Component;

/// Marker for infected / enemy-faction entities.
/// Declared here so all unit modules share one canonical type.
#[derive(Component)]
pub struct Infected;

pub mod soldier;
