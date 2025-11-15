use std::{collections::HashMap, fmt, fs};

use serde::{Deserialize, Serialize};

use crate::{CardId, InstanceId};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Mana(u32);

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum Faction {
    Sanctum,
    Thief,
}
impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Faction::Sanctum => write!(f, "Sanctum"),
            Faction::Thief => write!(f, "Thief"),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum WardSubType {
    Glyph,    // (Barrier)
    Rune,     // (Code Gate)
    Guardian, // (Sentry)
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum AssetSubType {
    Ambush,
    Ritual,
}
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum CounterSpellSubType {
    Fracter, // (Breaks Glyphs)
    Decoder, // (Breaks Runes)
    Killer,  // (Breaks Guardians)
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum CardType {
    AncientArtifact {
        vp: u32,
        attunement: u32,
    },
    Ward {
        subtype: WardSubType,
        cost: Mana,
        strength: u32,
    },
    Asset {
        subtype: AssetSubType,
        cost: Mana,
    },
    Operation {
        // Subtype is optional here, so Option is correct
        subtype: Option<AssetSubType>, // e.g., Ritual
        cost: Mana,
    },
    CounterSpell {
        subtype: CounterSpellSubType,
        cost: Mana,
        strength: u32,
        focus_cost: u32, // Replaces "SpellSlot"
    },
    Event {
        cost: Mana,
    },
    MagicalGear, // No stats, just text
    Ally,        // No stats, just text
}

// --- 3. The final "Card" struct is clean and simple ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CardState {
    Revealed(InstanceId, CardId),
    Hidden(InstanceId),
}
impl CardState {
    pub fn get_card_id(&self) -> Option<CardId> {
        match self {
            CardState::Revealed(instance_id, card_id) => Some(card_id.clone()),
            CardState::Hidden(_) => None,
        }
    }
    pub fn get_instance_id(&self) -> InstanceId {
        match self {
            CardState::Revealed(instance_id, _) => *instance_id,
            CardState::Hidden(instance_id) => *instance_id,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardData {
    pub id: CardId,    // e.g., "sanc-004"
    pub title: String, // e.g., "Glyph of Repellance"
    pub faction: Faction,
    pub text: String, // e.g., "↳ End the Infiltration."
    pub image_file: String,
    // The enum holds all the unique data
    pub data: CardType,
}

pub fn load_cards_from_json(
    file_path: &str,
) -> Result<HashMap<String, CardData>, Box<dyn std::error::Error>> {
    let json_string = fs::read_to_string(file_path)?;
    let cards: HashMap<String, CardData> = serde_json::from_str(&json_string)?;

    Ok(cards)
}
