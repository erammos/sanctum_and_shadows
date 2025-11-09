use common::InstanceId;
use macroquad::prelude::*;

use crate::card_view::CardView;

pub struct DragInfo {
    selected_card: u32,
    from_position: Vec3,
    from_target_id: Option<u32>,
    drag_offset: Vec3,
}
pub struct Board<'texture> {
    pub targets: Vec<DropTarget>,
    cards: Vec<CardView<'texture>>,
    current_drag: Option<DragInfo>,
}
pub enum TargetType {
    Event,
    BoardV,
    BoardH,
    Trash,
    Hand,
}

pub struct DropTarget {
    pub id: u32,
    pub anchor: Vec3,
    pub size: Vec2,
    pub target_type: TargetType,
}
impl DropTarget {
    pub fn draw(&self) {
        draw_plane(self.anchor, self.size, None, GREEN);
    }
}
impl<'texture> Board<'texture> {
    pub fn new(targets: Vec<DropTarget>) -> Self {
        Self {
            cards: vec![],
            targets,
            current_drag: None,
        }
    }
    pub fn add_card_to_target(&mut self, mut card: CardView<'texture>, target_id: u32) {
        card.attached_to_target = Some(target_id);
        self.cards.push(card);
    }
    pub fn update(&mut self, mouse_world: Vec3) {
        if is_mouse_button_pressed(MouseButton::Left) {
            if (self.current_drag.is_none()) {
                for (i, card) in self.cards.iter_mut().enumerate() {
                    if card.intersects(mouse_world) {
                        card.is_grabbed = true;
                        match card.card_state {
                            common::card::CardState::Revealed(instance_id, _) => {
                                self.current_drag = Some(DragInfo {
                                    selected_card: i as u32,
                                    from_position: card.position,
                                    drag_offset: mouse_world - card.position,
                                    from_target_id: None,
                                });
                            }
                            common::card::CardState::Hidden(_) => {
                                self.current_drag = None;
                            }
                        };
                        break;
                    }
                }
            }
        }
        if is_mouse_button_down(MouseButton::Left) {
            if let Some(drag) = &self.current_drag {
                let card = &mut self.cards[drag.selected_card as usize];
                card.position = mouse_world - drag.drag_offset;
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(drag) = &self.current_drag {
                let card = &mut self.cards[drag.selected_card as usize];
                let mut found_target = false;
                for target in &self.targets {
                    if card.intersects_area(&target) {
                        match target.target_type {
                            _ => card.position = target.anchor,
                            TargetType::BoardH => {}
                            TargetType::Hand => {}
                        }
                        found_target = true;
                        break;
                    }
                }
                if (!found_target) {
                    card.position = drag.from_position;
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
