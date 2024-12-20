use std::fmt::Display;

use godot::{builtin::Callable, classes::{BaseButton, INode}, global::{godot_print, godot_warn}, obj::{Base, Gd, WithBaseField}, prelude::{godot_api, GodotClass}};
use godot::classes::Node;



#[derive(GodotClass)]
#[class(base = Node)]
pub struct IngameStateTracker{
    base: Base<Node>,
    state: GameplayState,
    #[export]
    drawing_done_button: Option<Gd<BaseButton>>
}

#[godot_api]
impl INode for IngameStateTracker {
    fn init(base: Base<Node>)-> Self{
        Self{
            base,
            state: GameplayState::DEFENDING,
            drawing_done_button: None
        }
    }
    fn ready(&mut self){
        self.end_wave();
        self.get_drawing_done_button().unwrap().connect("pressed".into(), Callable::from_object_method(&self.base_mut(), "end_drawing"));
    }
}

#[godot_api]
impl IngameStateTracker{

    #[signal]
    fn on_start_wave();
    #[signal]
    fn on_start_draw();
    #[signal]
    fn on_death();
    #[signal]
    fn on_win();

    #[func]
    pub fn end_wave(&mut self){
        if self.get_state() == GameplayState::DEFENDING{
            self.state = GameplayState::DRAWING;
            self.get_drawing_done_button().unwrap().set_visible(true);
            self.base_mut().emit_signal(START_DRAW_SIGNAL.into(), &[]);
            godot_print!("end_wave");
        }else{
            self.warn_state_change_invalid(GameplayState::DRAWING);
        }
    }
    #[func]
    pub fn end_drawing(&mut self){
        if self.get_state() == GameplayState::DRAWING{
            self.state = GameplayState::DEFENDING;
            self.get_drawing_done_button().unwrap().set_visible(false);
            self.base_mut().emit_signal(START_WAVE_SIGNAL.into(), &[]);
            godot_print!("end_drawing");
        }else{
            self.warn_state_change_invalid(GameplayState::DEFENDING);
        }
    }
    #[func]
    pub fn die(&mut self){
        if self.get_state() == GameplayState::DEFENDING || self.get_state() == GameplayState::DRAWING{
            self.state = GameplayState::DEAD;
            self.get_drawing_done_button().unwrap().set_visible(false);
            self.base_mut().emit_signal(DEATH_SIGNAL.into(), &[]);
            godot_print!("dead");
        }else {
            //self.warn_state_change_invalid(GameplayState::DEAD);
        }
    }
    #[func]
    pub fn win(&mut self){
        if self.get_state() == GameplayState::DEFENDING{
            self.state = GameplayState::SUCCESS;
            self.base_mut().emit_signal(WIN_SIGNAL.into(), &[]);
            godot_print!("you won");
        }else {
            self.warn_state_change_invalid(GameplayState::SUCCESS);
        }
    }
}

pub const WIN_SIGNAL: &str = "on_win";
pub const DEATH_SIGNAL: &str = "on_death";
pub const START_WAVE_SIGNAL: &str = "on_start_wave";
pub const START_DRAW_SIGNAL: &str = "on_start_draw";

impl IngameStateTracker{
    fn warn_state_change_invalid(&self, next: GameplayState){
        godot_warn!("can't switch between state {} to {}", self.get_state(), next);
    }
    pub fn get_state(&self)->GameplayState{
        self.state
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GameplayState {
    DRAWING,
    DEFENDING,
    DEAD,
    SUCCESS
}

impl GameplayState {
    
}

impl Display for GameplayState{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self{
            Self::DRAWING=> "DRAWING",
            Self::DEAD=> "DEAD",
            Self::DEFENDING=> "DEFENDING",
            Self::SUCCESS=> "SUCCESS"
        })
    }
}