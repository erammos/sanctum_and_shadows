pub mod card;
pub mod player;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type InstanceId = u32;
pub type CardId = String;

use crate::card::{CardData, CardState, Faction};

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerType {
    Sanctum,
    Thief,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BasicStats {
    pub mana_pool: u32,
    pub stamina: u32,
    pub score: u32,
}
#[derive(Debug, Clone)]
pub struct Remote {
    pub wards: Vec<InstanceId>,
    pub contents: Option<InstanceId>,
}

#[derive(Debug, Clone)]
pub struct ThiefStateInternal {
    pub stats: BasicStats,

    pub deck: Vec<InstanceId>,
    pub hand: Vec<InstanceId>,
    pub discard: Vec<InstanceId>,
    pub score_area: Vec<InstanceId>,

    pub spell_slots: Vec<InstanceId>,
    pub gear_slots: Vec<InstanceId>,
    pub ally_slots: Vec<InstanceId>,
}

#[derive(Debug, Clone)]
pub struct SanctumStateInternal {
    pub stats: BasicStats,

    pub deck: Vec<InstanceId>,
    pub hand: Vec<InstanceId>,
    pub discard: Vec<InstanceId>,
    pub score_area: Vec<InstanceId>,

    pub hand_lair: Vec<InstanceId>,    // Protects HQ
    pub deck_lair: Vec<InstanceId>,    // Protects R&D
    pub discard_lair: Vec<InstanceId>, // Protects Archives
    pub remotes: Vec<Remote>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteRes {
    pub wards: Vec<CardState>,
    pub contents: Option<CardState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommonState {
    pub stats: BasicStats,
    pub deck: Vec<CardState>,
    pub hand: Vec<CardState>,
    pub discard: Vec<CardState>,
    pub score_area: Vec<CardState>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThiefState {
    pub spell_slots: Option<Vec<CardState>>,
    pub gear_slots: Option<Vec<CardState>>,
    pub ally_slots: Option<Vec<CardState>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SanctumState {
    pub hand_lair: Option<Vec<CardState>>,
    pub deck_lair: Option<Vec<CardState>>,
    pub discard_lair: Option<Vec<CardState>>,
    pub remotes: Option<Vec<RemoteRes>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Initial(InitStateResponse),
    DrawCard,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerStateResponse {
    Thief {
        common: CommonState,
        specific: ThiefState,
    },
    Sanctum {
        common: CommonState,
        specific: SanctumState,
    },
}
impl PlayerStateResponse {
    pub fn get_common(self) -> CommonState {
        match (self) {
            PlayerStateResponse::Thief { common, .. }
            | PlayerStateResponse::Sanctum { common, .. } => common,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitStateResponse {
    pub my_state: Option<PlayerStateResponse>,
    pub other_state: Option<PlayerStateResponse>,
    pub card_set: HashMap<CardId, CardData>,
    pub turn: Faction,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InitReq {
    pub name: String,
    pub faction: Faction,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum ActionReq {
    DrawCard,
    Init(InitReq),
}
