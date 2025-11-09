use crate::card::{CardData, Faction, Mana};
// In your game_logic.rs or model.rs

pub struct Player {
    pub id: Option<String>,
    pub faction: Option<Faction>,
}
