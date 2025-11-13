use common::InstanceId;
use macroquad::prelude::*;

use crate::card_view::CardView;

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
}

pub struct DropTarget {
    pub id: u32,
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
    pub fn update_layout(&mut self, target_id: usize) {
        let distance = self.cards[0].size.x * 2.0 + 0.02;

        let target = &self.targets[target_id as usize];
        match target.target_type {
            TargetType::BoardH => {}
            TargetType::Hand => {
                let mut next_pos = target.anchor;
                let offset = ((self.cards.len() - 1) as f32 * distance) / 2.0;
                for (i, card) in &mut self
                    .cards
                    .iter_mut()
                    .filter(|c| c.attached_to_target == Some(target_id))
                    .enumerate()
                {
                    card.position = vec3(next_pos.x - offset, next_pos.y, next_pos.z);
                    next_pos = vec3(next_pos.x + distance, next_pos.y, next_pos.z);
                }
            }
            TargetType::Event => {}
            TargetType::BoardV => {}
            TargetType::Trash => {}
        };
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
    pub fn update(&mut self, mouse_world: Vec3) {
        if (self.current_drag.is_none()) {
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
                        self.current_drag = None;
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
                for (i, target) in self.targets.iter().enumerate() {
                    if target.can_drop && card.intersects_area(&target) {
                        selected_target = Some(i);
                        break;
                    }
                }
                if let Some(target_id) = selected_target {
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
