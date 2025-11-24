use std::collections::HashMap;
use common::{CardId, InstanceId, Response};
use macroquad::prelude::*;
use message_io::network::Endpoint;
use message_io::node::NodeHandler;
use common::ActionReq::DrawCard;
use common::card::CardState;
use crate::card_view::CardView;
use crate::Net;

struct Timer {
    current: f64,
}
impl Timer {
    fn now() -> Timer {
        Timer {
            current: get_time(),
        }
    }
    fn elapsed(&self) -> u64 {
        ((get_time() - self.current) * 1000.0) as u64
    }
}
pub struct DragInfo {
    selected_card: usize,
    from_position: Vec3,
    from_target_id: Option<usize>,
    drag_offset: Vec3,
}
pub struct FocusInfo {
    selected_card: usize,
    previous_scale: f32,
}
pub struct Board<'texture> {
    pub targets: Vec<DropTarget>,
    cards: Vec<CardView<'texture>>,
    current_drag: Option<DragInfo>,
    current_focus: Option<FocusInfo>,
    focus_timer: Timer,
}

pub enum TargetType {
    Event,
    BoardV,
    BoardH,
    Trash,
    Hand,
    Stack,
}

pub const MY_HAND: usize = 0;
pub const MY_DECK: usize = 1;
pub const MY_TRASH: usize = 2;
pub const OTHER_HAND: usize = 3;
pub const OTHER_DECK: usize = 4;
pub const OTHER_TRASH: usize = 5;

pub struct DropTarget {
    pub id: usize,
    pub anchor: Vec3,
    pub size: Vec2,
    pub target_type: TargetType,
    pub can_drop: bool,
}

impl DropTarget {
    pub fn draw(&self) {
        draw_plane(self.anchor, self.size, None, PURPLE);
    }
}
impl<'texture> Board<'texture> {
    pub fn new(targets: Vec<DropTarget>) -> Self {
        Self {
            cards: vec![],
            targets,
            current_drag: None,
            current_focus: None,
            focus_timer: Timer::now(),
        }
    }
    pub fn draw_card(&mut self, card_state: &CardState, textures: &'texture std::collections::HashMap<std::string::String, macroquad::texture::Texture2D>) {

        let card_view  = self.cards.iter_mut().find(|t| t.card_state.get_instance_id() == card_state.get_instance_id()).unwrap();
        match card_state {
            CardState::Revealed(instance_id, card_id) => {
                card_view.card_state = card_state.clone();
                card_view.texture = &textures[card_id];
            }
            CardState::Hidden(instance_id) => {
                card_view.card_state = card_state.clone();
                card_view.texture = &textures["back"];
                card_view.attached_to_target = Some(OTHER_HAND);
                self.update_layout(OTHER_HAND);
            }
        }
    }
    pub fn update_layout(&mut self, target_id: usize) {
        let target = &self.targets.iter().find(|t| t.id == target_id).unwrap();
        let distance = self.cards[0].size.x * 2.0 + 0.02;
        let mut cards_per_target: Vec<_> = self
            .cards
            .iter_mut()
            .filter(|c| c.attached_to_target == Some(target_id))
            .collect();
        match target.target_type {
            TargetType::BoardH => {}
            TargetType::Hand => {
                let mut next_pos = target.anchor;
                let offset = ((cards_per_target.len() - 1) as f32 * distance) / 2.0;
                for card in cards_per_target.iter_mut().rev() {
                    card.position = vec3(next_pos.x - offset, next_pos.y, next_pos.z);
                    next_pos = vec3(next_pos.x + distance, next_pos.y, next_pos.z);
                }
            }
            TargetType::Event => {}
            TargetType::BoardV => {}
            TargetType::Trash => {}
            TargetType::Stack => {
                let mut next_pos = target.anchor;
                let offset = 0.02;
                for card in cards_per_target {
                    card.position = vec3(next_pos.x, next_pos.y, next_pos.z);
                    next_pos = vec3(next_pos.x , next_pos.y + offset, next_pos.z);
                }
            }
        };
        self.cards.sort_by(|a,b| b.position.y.total_cmp(&a.position.y));

    }
    pub fn add_card_to_target(&mut self, mut card: CardView<'texture>, target_id: usize) {
        card.attached_to_target = Some(target_id);
        self.cards.push(card);
        self.update_layout(target_id);
    }

    pub fn check_intersection(
        &mut self,
        mouse_world: Vec3,
    ) -> Option<(usize, &mut CardView<'texture>)> {
        for (i, card) in self.cards.iter_mut().enumerate() {
            if card.intersects(mouse_world) {
                return Some((i, card));
            }
        }
        None
    }
    pub fn zoom_out_all_cards(&mut self) {
        for card in self.cards.iter_mut() {
            self.current_focus = None;
            card.zoom_in(1.0)
        }
    }
    pub fn update(&mut self, mouse_world: Vec3, handler: &NodeHandler<()>, endpoint: Endpoint) {
        if self.current_drag.is_none() {
            let is_focus = self.current_focus.is_some();
            if let Some((index, card)) = self.check_intersection(mouse_world) {
                if !is_focus {
                    card.zoom_in(3.0);
                    self.current_focus = Some(FocusInfo {
                        selected_card: index,
                        previous_scale: 1.0,
                    })
                }
            } else {
                self.zoom_out_all_cards();
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) && (self.current_drag.is_none()) {
            self.zoom_out_all_cards();
            if let Some((index, card)) = self.check_intersection(mouse_world) {
                card.is_grabbed = true;
                match card.card_state {
                    common::card::CardState::Revealed(instance_id, _) => {
                        self.current_drag = Some(DragInfo {
                            selected_card: index,
                            from_position: card.position,
                            drag_offset: mouse_world - card.position,
                            from_target_id: card.attached_to_target,
                        });
                    }
                    common::card::CardState::Hidden(_) => {
                        self.current_drag = Some(DragInfo {
                            selected_card: index,
                            from_position: card.position,
                            drag_offset: mouse_world - card.position,
                            from_target_id: card.attached_to_target,
                        });;
                    }
                };
            }
        } else if is_mouse_button_down(MouseButton::Left) {
            if let Some(drag) = &self.current_drag {
                let card = &mut self.cards[drag.selected_card as usize];
                card.position = mouse_world - drag.drag_offset;
            }
        } else if is_mouse_button_released(MouseButton::Left) {
            if let Some(drag) = &self.current_drag {
                let card = &mut self.cards[drag.selected_card as usize];
                let mut selected_target: Option<usize> = drag.from_target_id;
                for (target) in &self.targets {
                    if (target.can_drop || target.id == drag.from_target_id.unwrap()) && card.intersects_area(&target) {
                        selected_target = Some(target.id);
                        break;
                    }
                }
                if let Some(target_id) = selected_target {
                    #[allow(clippy::single_match)] match drag.from_target_id.unwrap()
                    {
                        MY_DECK=> if target_id == MY_HAND {
                           let req =  DrawCard(card.card_state.get_instance_id());
                            let output_data = bincode::serialize(&req).unwrap();
                            handler.network().send(endpoint,&output_data);
                       },
                        _ => {}
                    }
                    card.attached_to_target = Some(target_id);
                    self.update_layout(target_id);
                }
                self.current_drag = None;
            }
        }
    }
    pub fn draw(&self) {
        for target in &self.targets {
            target.draw();
        }
        for card in &self.cards {
            card.draw();
        }
    }
}
