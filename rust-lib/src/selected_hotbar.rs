use godot::{builtin::Vector2, classes::TextureRect, obj::{Base, WithBaseField}, prelude::{godot_api, GodotClass}};

#[derive(GodotClass)]
#[class(base = TextureRect, init)]
pub struct SelectedHotbar{
    base: Base<TextureRect>,
    #[export]
    option_offset: Vector2,
    #[export]
    min_pos: Vector2
}

#[godot_api]
impl SelectedHotbar{
    #[func]
    pub fn set_selected(&mut self, index: u32){
        let mut new_transform = self.min_pos; //sloppy code, don't need this, PLEASE FIX
        new_transform += index as f32 * self.option_offset;
        self.base_mut().set_position(new_transform);
    }
}