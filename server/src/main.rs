use std::collections::HashMap;
use std::fs;

use common::card::{CardData, CardState, Faction, load_cards_from_json};
use common::player::Player;
use common::{
    ActionReq, BasicStats, CardId, InitStateResponse, InstanceId, PlayerStateResponse, PlayerType,
    SanctumStateResponse, ThieStateResponse,
};
use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self, NodeListener}; // <-- Using shared code!

type IsHidden = bool;
enum InstantiateLocation {
    BOARD(IsHidden),
    HAND,
    DECK,
    TRASH,
}
struct InstantiatedCard {
    id: CardId,
    location: InstantiateLocation,
}
struct Instances {
    counter: u32,
    pub data: HashMap<InstanceId, InstantiatedCard>,
}
impl Instances {
    fn new() -> Self {
        Self {
            counter: 0,
            data: HashMap::new(),
        }
    }
    fn create_instance(&mut self, card_id: &CardId, location: InstantiateLocation) -> u32 {
        self.counter += 1;
        self.data.insert(
            self.counter,
            InstantiatedCard {
                id: card_id.clone(),
                location,
            },
        );
        self.counter
    }
    fn get_instantiated_card(&self, instance_id: InstanceId) -> &InstantiatedCard {
        self.data.get(&instance_id).unwrap()
    }
    fn get_mut_instantiated_card(&mut self, instance_id: InstanceId) -> &mut InstantiatedCard {
        self.data.get_mut(&instance_id).unwrap()
    }
    fn create_card_state(&self, instance_id: InstanceId, visible: bool) -> CardState {
        if visible {
            CardState::Revealed(
                instance_id,
                self.get_instantiated_card(instance_id).id.clone(),
            )
        } else {
            CardState::Hidden(instance_id)
        }
    }
    fn create_card_states(&self, instances: &Vec<InstanceId>, visible: bool) -> Vec<CardState> {
        let cards: Vec<CardState> = instances
            .iter()
            .map(|&instance_id| self.create_card_state(instance_id, visible))
            .collect();
        cards
    }
}
fn instantiate_deck(
    faction: Faction,
    cards: &HashMap<String, CardData>,
    instances: &mut Instances,
) -> Vec<InstanceId> {
    let mut deck: Vec<InstanceId> = vec![];
    for i in 0..4 {
        for data in cards.iter().filter(|data| data.1.faction == faction) {
            deck.push(instances.create_instance(&data.1.id, InstantiateLocation::DECK));
        }
    }
    deck
}
fn create_hand(deck: &mut Vec<InstanceId>, instances: &mut Instances) -> Vec<InstanceId> {
    let mut hand = vec![];
    for i in (0..5) {
        let ins = deck.pop().unwrap();
        let card = instances.get_mut_instantiated_card(ins);
        card.location = InstantiateLocation::HAND;
        hand.push(ins);
    }

    hand
}
fn create_sanctum_state_response(
    visible: bool,
    instances: &Instances,
    deck: &Vec<InstanceId>,
    hand: &Vec<InstanceId>,
) -> PlayerStateResponse {
    PlayerStateResponse::Sanctum(SanctumStateResponse {
        stats: BasicStats {
            mana_pool: 5,
            stamina: 5,
            score: 0,
        },
        deck: instances.create_card_states(&deck, visible),
        hand: instances.create_card_states(&hand, visible),
        discard: vec![],
        score_area: vec![],
        hand_lair: None,
        deck_lair: None,
        discard_lair: None,
        remotes: None,
    })
}
fn create_thief_state_response(
    visible: bool,
    instances: &Instances,
    deck: &Vec<InstanceId>,
    hand: &Vec<InstanceId>,
) -> PlayerStateResponse {
    PlayerStateResponse::Thief(ThieStateResponse {
        stats: BasicStats {
            mana_pool: 5,
            stamina: 5,
            score: 0,
        },
        deck: instances.create_card_states(&deck, visible),
        hand: instances.create_card_states(&hand, visible),
        discard: vec![],
        score_area: vec![],
        spell_slots: None,
        gear_slots: None,
        ally_slots: None,
    })
}
fn main() {
    let cards = load_cards_from_json("cards.json").unwrap();
    let mut instances = Instances::new();
    let mut sanc_deck = instantiate_deck(Faction::Sanctum, &cards, &mut instances);
    let sanc_hand = create_hand(&mut sanc_deck, &mut instances);
    let mut thief_deck = instantiate_deck(Faction::Thief, &cards, &mut instances);
    let thief_hand = create_hand(&mut thief_deck, &mut instances);

    let mut clients: HashMap<Endpoint, Player> = HashMap::new();
    let (node, listener) = node::split::<()>();

    // Start listening for WebSocket connections
    node.network()
        .listen(Transport::Ws, "0.0.0.0:8080")
        .unwrap();
    println!("Server running on ws://0.0.0.0:8080");

    // A simple way to store all your game states

    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => (), // Only generated at connect() calls.
        NetEvent::Accepted(endpoint, _listener_id) => {
            // Only connection oriented protocols will generate this event
            clients.insert(
                endpoint,
                Player {
                    id: None,
                    faction: None,
                },
            );
            println!("Client ({}) connected", endpoint.addr());
        }
        NetEvent::Message(endpoint, input_data) => {
            let message: ActionReq = bincode::deserialize(&input_data).unwrap();

            match message {
                ActionReq::DrawCard => {
                    //                   let output_data = bincode::serialize(&state).unwrap();
                    //                  for client in clients.iter() {
                    //                     node.network().send(*client.0, &output_data);
                }
                ActionReq::Init(init_req) => {
                    let d = clients.get_mut(&endpoint).unwrap();
                    d.faction = Some(init_req.faction);
                    d.id = Some(init_req.name);

                    let response = match (init_req.faction) {
                        Faction::Sanctum => InitStateResponse {
                            my_state: Some(create_sanctum_state_response(
                                true, &instances, &sanc_deck, &sanc_hand,
                            )),
                            other_state: Some(create_thief_state_response(
                                false,
                                &instances,
                                &thief_deck,
                                &thief_hand,
                            )),
                            card_set: cards.clone(),
                            turn: Faction::Sanctum,
                        },
                        Faction::Thief => InitStateResponse {
                            my_state: Some(create_thief_state_response(
                                true,
                                &instances,
                                &thief_deck,
                                &thief_hand,
                            )),
                            other_state: Some(create_sanctum_state_response(
                                false, &instances, &sanc_deck, &sanc_hand,
                            )),
                            card_set: cards.clone(),
                            turn: Faction::Sanctum,
                        },
                    };
                    let bytes = bincode::serialize(&response).unwrap();
                    node.network().send(endpoint, &bytes);
                    dbg!("Player init with {} {}", d.faction, &d.id);
                    dbg!("Send to Player {:?}", &response);
                }
            }
        }
        NetEvent::Disconnected(endpoint) => {
            // Only connection oriented protocols will generate this event
            clients.remove(&endpoint);
            println!("Client ({}) disconnected", endpoint.addr());
        }
    });
}
