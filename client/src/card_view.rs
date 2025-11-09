use common::card::CardState;
use macroquad::prelude::{scene::camera_pos, *};

use crate::board::{DropTarget, TargetType};

pub struct CardView<'texture> {
    pub card_state: CardState,
    pub texture: &'texture Texture2D,
    pub position: Vec3,
    pub size: Vec2,
    pub is_grabbed: bool,
    pub attached_to_target: Option<u32>,
}

impl<'texture> CardView<'texture> {
    pub fn new(card_state: CardState, texture: &'texture Texture2D) -> Self {
        Self {
            card_state,
            texture,
            position: vec3(0.0, 0.0, 0.0),
            size: vec2(10.0 / 16.0, 16.0 / 16.0),
            is_grabbed: false,
            attached_to_target: None,
        }
    }
    pub fn intersects(&self, point: Vec3) -> bool {
        let p1 = self.position + vec3(-self.size.x, 0.0, -self.size.y);
        let p3 = self.position + vec3(self.size.x, 0.0, self.size.y);
        point.x >= p1.x && point.x < p3.x && point.z < p3.z && point.z >= p1.z
    }
    pub fn intersects_area(&self, target: &DropTarget) -> bool {
        self.position.x - self.size.x <= target.anchor.x + target.size.x
            && self.position.x + self.size.x >= target.anchor.x - target.size.x
            && self.position.z - self.size.y <= target.anchor.z + target.size.y
            && self.position.z + self.size.y >= target.anchor.z - target.size.y
    }
    pub fn update(&mut self, mouse_world: Vec3) {
        if is_mouse_button_pressed(MouseButton::Left) {
            if (self.intersects(mouse_world)) {
                self.is_grabbed = true
            }
        }
        if is_mouse_button_down(MouseButton::Left) {
            if self.is_grabbed {
                self.position = vec3(mouse_world.x, 0.0, mouse_world.z);
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            self.is_grabbed = false;
        }
    }
    pub fn draw(&self) {
        match self.card_state {
            CardState::Revealed(_, _) => {
                draw_plane(self.position, self.size, Some(&self.texture), WHITE);
            }
            CardState::Hidden(_) => {
                draw_plane(self.position, self.size, None, RED);
            }
        }
    }
}
