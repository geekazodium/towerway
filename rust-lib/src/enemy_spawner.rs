use godot::{classes::{INode, Node, PackedScene, Path2D}, obj::{Base, Gd, WithBaseField}, prelude::{godot_api, GodotClass}};

use crate::{ingame_state_tracker::{GameplayState, IngameStateTracker}, player_health::PlayerHealth};

#[derive(GodotClass)]
#[class(base = Node, init)]
pub struct EnemySpawner{
    base: Base<Node>,
    #[export]
    interval: f64,
    #[export]
    timer: f64,
    #[export]
    enemies: Option<Gd<PackedScene>>,
    #[export]
    enabled: bool,
    #[export]
    game_state: Option<Gd<IngameStateTracker>>
}

#[godot_api]
impl INode for EnemySpawner{
    fn ready(&mut self){
        if self.game_state.is_none(){
            self.game_state = Some(self.base().get_parent().unwrap().get_parent().unwrap().find_child("IngameStateTracker".into()).unwrap().cast());
        }
    }
    fn physics_process(&mut self, delta: f64){
        if self.get_game_state().unwrap().bind().get_state() != GameplayState::DEFENDING{
            return;
        }
        if !self.get_enabled(){
            return;
        }
        self.timer += delta;
        if self.timer > self.interval{
            self.timer = 0.;
            let instance = self.enemies.clone().expect("enemy scene not set").instantiate().unwrap();
            let mut parent = self.base().get_parent().unwrap();
            parent.add_child(instance);
        }
    }
}

#[derive(GodotClass)]
#[class(base = Path2D, init)]
pub struct EnemyPath{
    base: Base<Path2D>,
    #[export]
    player_health: Option<Gd<PlayerHealth>>
}

impl EnemyPath{
    pub fn hit_player(&self){
        self.get_player_health().unwrap().bind_mut().take_damage(1);
    }
}