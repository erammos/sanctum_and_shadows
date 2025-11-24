mod board;
mod card_view;
use std::collections::HashMap;

use common::CardId;
use common::Response;
use common::card::CardState;
use common::card::Faction;
use common::{ActionReq, InitReq, InitStateResponse};
use macroquad::miniquad::{ElapsedQuery, window};
use macroquad::prelude::*;
use message_io::events::EventReceiver;
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::StoredNetEvent;
use message_io::node::StoredNodeEvent;
use message_io::node::{self, NodeHandler, NodeListener};

use crate::board::Board;
use crate::board::DropTarget;
use crate::board::MY_DECK;
use crate::board::MY_HAND;
use crate::board::MY_TRASH;
use crate::board::OTHER_DECK;
use crate::board::OTHER_HAND;
use crate::board::OTHER_TRASH;
use crate::card_view::CardView; // <-- Using shared code!
// Helper to store our networking items
struct Net {
    handler: NodeHandler<()>,
    listener: NodeListener<()>,
    server_id: Endpoint,
}
use glam::{Mat4, Vec2, Vec3, Vec4};

fn ndc_to_world(m_inv: &Mat4, ndc_pos: Vec2) -> Vec3 {
    let p_clip_near = Vec4::new(ndc_pos.x, -ndc_pos.y, -1.0, 1.0);
    let p_clip_far = Vec4::new(ndc_pos.x, -ndc_pos.y, 1.0, 1.0);

    let p_near_h = *m_inv * p_clip_near;
    let p_far_h = *m_inv * p_clip_far;

    // Perform the Perspective Divide (Crucial Step for perspective matrices!)
    // P_world = P_homogeneous / W
    // This gives the actual 3D points P_near (Origin) and P_far (Directional point)
    let p_near = Vec3::new(
        p_near_h.x / p_near_h.w,
        p_near_h.y / p_near_h.w,
        p_near_h.z / p_near_h.w,
    );
    let p_far = Vec3::new(
        p_far_h.x / p_far_h.w,
        p_far_h.y / p_far_h.w,
        p_far_h.z / p_far_h.w,
    );

    let a = p_near; // Ray Origin
    let d = p_far - p_near; // Ray Direction Vector

    // Ray-Plane Intersection Formula: t = -A.z / D.z

    if d.y.abs() < 1e-6 {
        return a;
    }

    // Calculate the interpolation factor 't'
    let t = -a.y / d.y;

    // P_world = A + t * D
    let world_coord = a + t * d;

    // The Z component is now guaranteed to be 0 (or mathematically near-zero).
    world_coord
}

fn other_faction(my_faction: Faction) -> Faction {
    match my_faction {
        Faction::Sanctum => Faction::Thief,
        Faction::Thief => Faction::Sanctum,
    }
}

fn get_color(faction: Faction, turn: Faction) -> Color {
    if turn == faction {
        Color::from_rgba(0, 255, 0, 255)
    } else {
        Color::from_rgba(255, 0, 0, 255)
    }
}
fn is_my_turn(my_faction: Faction, state: &InitStateResponse) -> bool {
    my_faction == state.turn
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Window Conf".to_owned(),
        fullscreen: false,
        window_height: 1080,
        window_width: 1920,
        ..Default::default()
    }
}

pub fn receive(receiver: &mut EventReceiver<StoredNodeEvent<()>>) -> Option<Response> {
    match receiver.try_receive() {
        Some(event) => match event {
            node::StoredNodeEvent::Network(net_event) => match net_event {
                node::StoredNetEvent::Message(endpoint, data) => {
                    let state: Response = bincode::deserialize(&data).unwrap();
                    dbg!("{:?}", &state);
                    Some(state)
                }
                _ => None,
            },
            node::StoredNodeEvent::Signal(_) => None,
        },
        None => None,
    }
}
pub fn get_texture_from_card_state<'a>(c:&CardState, textures:&'a HashMap<CardId, Texture2D>) -> &'a Texture2D
{
    match  c{
        CardState::Revealed(_, cardid) => {
            &textures[cardid]
        }
        CardState::Hidden(_) => {
            &textures["back"]
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    prevent_quit();
    let selected_fanction = match args.get(2).expect("Select faction").as_ref() {
        "thief" => Faction::Thief, // The non-streamed version of tcp.
        "sanctum" => Faction::Sanctum,
        _ => panic!("{}", "Select thief or sanctum"),
    };
    let name = args.get(1).expect("Select Name");
    let (handler, listener) = node::split::<()>();

    let (server_id, _) = handler
        .network()
        .connect(Transport::Ws, "127.0.0.1:8080")
        .unwrap();

    let mut net = Net {
        handler,
        listener,
        server_id,
    };

    let back_texture = load_texture("client/assets/cards/back.png").await.unwrap();
    let (task, mut receiver) = net.listener.enqueue();
    let bytes = bincode::serialize(&ActionReq::Init(InitReq {
        name: name.clone(),
        faction: selected_fanction,
    }))
    .unwrap();
    let mut status = net.handler.network().send(net.server_id, &bytes);
    while (status != SendStatus::Sent) {
        status = net.handler.network().send(net.server_id, &bytes);
        next_frame().await;
    }
    let mut response = None;
    while let None = response {
        response = receive(&mut receiver);
    }
    let camera = Camera3D {
        position: vec3(0.0, 5.0, 0.0),
        up: vec3(0., 0., -1.0),
        target: vec3(0., 0., 0.),
        projection: Projection::Perspective,
        ..Default::default()
    };
    let inv_matrix = camera.matrix().inverse();
    let mut board = Board::new(vec![
        DropTarget {
            id: OTHER_HAND,
            anchor: vec3(0.0, 0.0, -1.5),
            size: vec2(10.0, 0.5),
            target_type: board::TargetType::Hand,
            can_drop: false,
        },
        DropTarget {
            id: MY_DECK,
            anchor: vec3(-2.0, 0.0, 1.5),
            size: vec2(1.0, 1.0),
            target_type: board::TargetType::Stack,
            can_drop: false,
        },
        DropTarget {
            id: OTHER_DECK,
            anchor: vec3(2.0, 0.0, -1.5),
            size: vec2(1.0, 1.0),
            target_type: board::TargetType::Stack,
            can_drop: false,
        },
        DropTarget {
            id: MY_HAND,
            anchor: vec3(0.0, 0.0, 1.5),
            size: vec2(10.0, 0.5),
            target_type: board::TargetType::Hand,
            can_drop: true,
        },
    ]);

    let mut textures: HashMap<CardId, Texture2D> = HashMap::new();
    let mut card_set = None;
    let mut turn: Faction = Faction::Thief;
    if let Some(response_data) = response { match response_data {
        Response::Initial(init_state_response) => {
            turn = init_state_response.turn;
            card_set = Some(init_state_response.card_set);
            for (id, card) in card_set.unwrap().iter() {
                let texture = load_texture(&card.image_file).await.unwrap();
                textures.insert(id.to_string(), texture);
            }
            textures.insert("back".to_string(),back_texture);
            let mystate = init_state_response.my_state.unwrap();
            let other_state = init_state_response.other_state.unwrap();
            for c in mystate.get_common().hand.iter() {
                board.add_card_to_target(CardView::new(c.clone(),get_texture_from_card_state(c,&textures)), MY_HAND);
            }
            for c in mystate.get_common().deck.iter() {
                board.add_card_to_target(CardView::new(c.clone(), get_texture_from_card_state(c,&textures)), MY_DECK);
            }
            for c in other_state.get_common().hand.iter() {
                board.add_card_to_target(CardView::new(c.clone(), get_texture_from_card_state(c,&textures)), OTHER_HAND);
            }
            for c in other_state.get_common().deck.iter() {
                board.add_card_to_target(CardView::new(c.clone(), get_texture_from_card_state(c,&textures)), OTHER_DECK);
            }
        }
        Response::DrawCard {  card: ref c } => {
            board.draw_card(c, &textures);
        }
    } }
    //  board.add_card_to_target(card, 1);
    // board.add_card_to_target(card1, 1);
    loop {

        clear_background(BLACK);

        let mut response = receive(&mut receiver);
        if let Some(response_data) = response { match response_data {
            Response::DrawCard { card: ref c } => {
                board.draw_card(c, &textures);
            }
            _=> ()
        } }
        set_camera(&camera);

        let mouse_world_pos = ndc_to_world(&inv_matrix, mouse_position_local());
        board.update(mouse_world_pos, &net.handler, net.server_id);
        board.draw();

        set_default_camera();
        draw_text(
            &format!("{}", selected_fanction),
            10.0,
            screen_height() - 50.0,
            40.0,
            get_color(selected_fanction, turn),
        );
        draw_text(
            &format!("{}", other_faction(selected_fanction)),
            10.0,
            50.0,
            40.0,
            get_color(other_faction(selected_fanction), turn),
        );
        if is_quit_requested() {
            net.handler.stop();
            break;
        }

        next_frame().await;
    }
}
