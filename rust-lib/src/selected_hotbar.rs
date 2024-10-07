use godot::{builtin::{Transform2D, Vector2}, classes::{ITextureRect, TextureRect}, obj::{Base, WithBaseField}, prelude::{godot_api, GodotClass}};

#[derive(GodotClass)]
#[class(base = TextureRect, init)]
struct SelectedHotbar{
    base: Base<TextureRect>,
    #[export]
    option_offset: Vector2,
    min_transform: Transform2D
}

#[godot_api]
impl ITextureRect for SelectedHotbar{
    fn ready(&mut self){
        self.min_transform = self.base().get_transform().clone();
    }
}

#[godot_api]
impl SelectedHotbar{
    #[func]
    pub fn set_selected(&mut self, index: u32){
        let mut new_transform = self.min_transform.clone();
        new_transform.origin += index as f32 * self.option_offset;
    }
}