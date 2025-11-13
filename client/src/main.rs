mod board;
mod card_view;
use common::Response;
use common::card::CardState;
use common::card::Faction;
use common::{ActionReq, InitReq, InitStateResponse};
use macroquad::miniquad::{ElapsedQuery, window};
use macroquad::prelude::*;
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::{self, NodeHandler, NodeListener};

use crate::board::Board;
use crate::board::DropTarget;
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
#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let texture: Texture2D = load_texture("client/assets/sanc-003.png").await.unwrap();

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
            id: 0,
            anchor: vec3(0.0, 0.0, -2.0),
            size: vec2(10.0, 0.5),
            target_type: board::TargetType::Hand,
            can_drop: false,
        },
        DropTarget {
            id: 1,
            anchor: vec3(0.0, 0.0, 1.5),
            size: vec2(10.0, 0.5),
            target_type: board::TargetType::Hand,
            can_drop: true,
        },
    ]);

    let mut card_set = None;
    //  board.add_card_to_target(card, 1);
    // board.add_card_to_target(card1, 1);
    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            let msg = ActionReq::DrawCard;
            let bytes = bincode::serialize(&msg).unwrap();
            net.handler.network().send(net.server_id, &bytes);
        }

        clear_background(BLACK);

        let mut response = match receiver.try_receive() {
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
        };

        let mut turn: Faction = Faction::Thief;
        match response {
            Some(response_data) => match response_data {
                Response::Initial(init_state_response) => {
                    turn = init_state_response.turn;
                    card_set = Some(init_state_response.card_set);
                    let mystate = init_state_response.my_state.unwrap();
                    let common = mystate.get_common();
                    for c in common.deck {
                        let mut card = CardView::new(c, &texture);
                    }
                    for c in common.hand {
                        let mut card = CardView::new(c, &texture);
                        board.add_card_to_target(card, 1);
                    }
                }
                Response::DrawCard => {}
            },
            None => {}
        }
        // --- 3. DRAWING ---
        //draw_texture(&texture, 0.0, 0.0, WHITE);

        set_camera(&camera);

        let mouse_world_pos = ndc_to_world(&inv_matrix, mouse_position_local());
        board.update(mouse_world_pos);
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
