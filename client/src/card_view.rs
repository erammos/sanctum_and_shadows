use common::card::CardState;
use macroquad::prelude::{scene::camera_pos, *};

use crate::board::{DropTarget, TargetType};

pub struct CardView<'texture> {
    pub card_state: CardState,
    pub texture: &'texture Texture2D,
    pub position: Vec3,
    pub size: Vec2,
    pub zoom_in: f32,
    pub is_grabbed: bool,
    pub attached_to_target: Option<usize>
}

impl<'texture> CardView<'texture> {
    pub fn zoom_in(&mut self, scale: f32) {
        self.zoom_in = scale;
    }
    pub fn new(card_state: CardState, texture: &'texture Texture2D) -> Self {
        Self {
            card_state,
            texture,
            position: vec3(0.0, 0.0, 0.0),
            size: vec2((10.0 / 16.0 ) * 0.25, 1.0 * 0.25),
            is_grabbed: false,
            zoom_in: 1.0,
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
    pub fn draw(&self) {
        match self.card_state {
            CardState::Revealed(_, _) => {
                draw_plane(self.position, self.size, Some(&self.texture), WHITE);
                if self.zoom_in > 1.0
                {
                   let new_position = vec3( self.position.x, self.position.y + 1.0,self.position.z - 1.2);
                    draw_plane(new_position, self.size * self.zoom_in, Some(&self.texture), WHITE);
                }
            }
            CardState::Hidden(_) => {
                draw_plane(self.position, self.size * self.zoom_in, None, RED);
            }
        }
    }
}
