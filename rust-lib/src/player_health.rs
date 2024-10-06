use godot::classes::INode;
use godot::classes::Node;
use godot::classes::PackedScene;
use godot::classes::TextureProgressBar;
use godot::obj::Base;
use godot::obj::Gd;
use godot::obj::WithBaseField;
use godot::prelude::godot_api;
use godot::prelude::GodotClass;

use crate::ingame_state_tracker::IngameStateTracker;

#[derive(GodotClass)]
#[class(base = Node, init)]
pub struct PlayerHealth{
    base: Base<Node>,
    #[export]
    max_health: i32,
    #[export]
    health: i32,
    #[export]
    game_over_scene: Option<Gd<PackedScene>>,
    #[export]
    game_state: Option<Gd<IngameStateTracker>>,
    #[export]
    health_bar: Option<Gd<TextureProgressBar>>
}

#[godot_api]
impl INode for PlayerHealth {
    fn ready(&mut self){
        self.health = self.max_health;
        let mut health_bar = self.get_health_bar().expect("no healthbar found");
        health_bar.set_value(self.health as f64);
        health_bar.set_max(self.max_health as f64);
    }
}

#[godot_api]
impl PlayerHealth{
    //function for player to take damage, amount is how much health the player looses
    #[func]
    pub fn take_damage(&mut self, amount: i32){
        self.health -= amount;
        let mut health_bar = self.get_health_bar().expect("no healthbar found");
        health_bar.set_value(self.health as f64);
        if self.health <= 0 {
            let inst = self.get_game_over_scene().unwrap().instantiate().unwrap();
            self.get_game_state().unwrap().bind_mut().die();
            self.base_mut().add_child(inst);
        }
    }
}