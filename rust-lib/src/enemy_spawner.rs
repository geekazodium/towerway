use godot::{builtin::{Array, Callable}, classes::{Area2D, IArea2D, INode, Node, PackedScene, Path2D, TextureProgressBar}, obj::{Base, Gd, WithBaseField}, prelude::{godot_api, GodotClass}};

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
    enabled: bool,
    #[export]
    wait_for: Option<Gd<EnemySpawner>>,
    #[export]
    spawns_left: u32,
    #[export]
    game_state: Option<Gd<IngameStateTracker>>
}

#[godot_api]
impl INode for EnemySpawner{
    fn ready(&mut self){
        if self.game_state.is_none(){
            self.game_state = Some(self.base().get_parent().unwrap().get_parent().unwrap().find_child("IngameStateTracker".into()).unwrap().cast());
        }
        let waitfor = self.get_wait_for();
        if waitfor.is_none(){
            self.enabled = true;
        }else{
            self.enabled = false;
            let mut c = waitfor.unwrap();
            c.connect("spawning_end".into(),Callable::from_object_method(&self.to_gd(),"on_spawning_end"));
        }
    }
    fn physics_process(&mut self, delta: f64){
        if self.get_game_state().unwrap().bind().get_state() != GameplayState::DEFENDING{
            return;
        }
        if !self.enabled{
            return;
        }
        if self.is_done(){
            return;
        }
        self.timer += delta;
        if self.timer > self.interval{
            self.timer = 0.;
            let instance = self.enemies.clone().expect("enemy scene not set").instantiate().unwrap();
            let mut parent = self.base().get_parent().unwrap();
            parent.add_child(instance);
            self.spawns_left -= 1;
            if self.is_done(){
                self.base_mut().emit_signal("spawning_end".into(), &[]);
            }
        }
    }
}

#[godot_api]
impl EnemySpawner{
    #[signal]
    fn spawning_end();
    #[func]
    fn on_spawning_end(&mut self){
        self.enabled = true;
    }
}

impl EnemySpawner{
    fn is_done(&self) -> bool{
        self.spawns_left <= 0
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

#[derive(GodotClass)]
#[class(base = Area2D, init)]
struct EnemySpawnerProgressTracker{
    #[export]
    progress_bar: Option<Gd<TextureProgressBar>>,
    #[export]
    spawners: Array<Gd<EnemySpawner>>,
    base: Base<Area2D>,
    #[export]
    gamestate: Option<Gd<IngameStateTracker>>
}

#[godot_api]
impl IArea2D for EnemySpawnerProgressTracker{
    fn ready(&mut self){
        self.get_progress_bar().unwrap().set_max(self.spawners.len() as f64);
    }
    fn physics_process(&mut self, _delta: f64) {
        
        let mut active_spawners_count = self.spawners.len();
        for spawner in self.spawners.iter_shared() {
            if spawner.bind().is_done(){
                active_spawners_count -= 1;
            }
        }

        self.get_progress_bar().unwrap().set_value(active_spawners_count as f64);

        if active_spawners_count == 0 && self.base().get_overlapping_areas().len() == 0{
            self.get_gamestate().unwrap().bind_mut().win();
        }
    }
}