use common::InstanceId;
use macroquad::prelude::*;

use crate::card_view::CardView;

pub struct DragInfo {
    selected_card: u32,
    from_position: Vec3,
    from_target_id: Option<u32>,
    drag_offset: Vec3,
}
pub struct Board<'board> {
    pub targets: Vec<DropTarget>,
    cards: Vec<CardView<'board>>,
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
impl<'board> Board<'board> {
    pub fn new(targets: Vec<DropTarget>) -> Self {
        Self {
            cards: vec![],
            targets,
            current_drag: None,
        }
    }
    pub fn update_layout(&mut self,target_id:u32)
    {
        let distance = self.cards[0].size.x * 2.0 + 0.02;

        let target = &self.targets[target_id as usize];
        match target.target_type {
            TargetType::BoardH => {

            },
            TargetType::Hand => {
                let mut next_pos = target.anchor;
                let offset = ((self.cards.len() - 1) as f32 * distance) / 2.0;
                for (i,card) in &mut self.cards.iter_mut().filter(|c| c.attached_to_target == Some(target_id)).enumerate()
                {
                    card.position = vec3(next_pos.x - offset,next_pos.y,next_pos.z);
                    next_pos = vec3(next_pos.x + distance,next_pos.y,next_pos.z);
                }

            },
            TargetType::Event => {}
            TargetType::BoardV => {}
            TargetType::Trash => {}
        };
    }
    pub fn add_card_to_target(&mut self, mut card: CardView<'board>, target_id: u32) {
        card.attached_to_target = Some(target_id);
        self.cards.push(card);
        self.update_layout(target_id);
    }
    pub fn update(&mut self, mouse_world: Vec3) {
        if is_mouse_button_pressed(MouseButton::Left) && (self.current_drag.is_none()) {
            for (i, card) in self.cards.iter_mut().enumerate() {
                if card.intersects(mouse_world) {
                    card.is_grabbed = true;
                    match card.card_state {
                        common::card::CardState::Revealed(instance_id, _) => {
                            self.current_drag = Some(DragInfo {
                                selected_card: i as u32,
                                from_position: card.position,
                                drag_offset: mouse_world - card.position,
                                from_target_id: card.attached_to_target,
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
        if is_mouse_button_down(MouseButton::Left) {
            if let Some(drag) = &self.current_drag {
                let card = &mut self.cards[drag.selected_card as usize];
                card.position = mouse_world - drag.drag_offset;
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(drag) = &self.current_drag {
                let card = &mut self.cards[drag.selected_card as usize];
                let mut selected_target:Option<u32> = drag.from_target_id;
                for (i,target)in self.targets.iter().enumerate() {
                    if card.intersects_area(&target) {
                        selected_target = Some(i as u32);
                        break;
                    }
                }
                if let Some(target_id)  = selected_target{
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
