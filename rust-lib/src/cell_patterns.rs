use core::f64;
use std::ops::Add;

use godot::builtin::Array;
use godot::builtin::Rect2i;
use godot::builtin::Vector2;
use godot::builtin::Vector2i;
use godot::classes::Control;
use godot::classes::INode;
use godot::classes::ITileMapLayer;
use godot::classes::Input;
use godot::classes::Node;
use godot::classes::TextureProgressBar;
use godot::classes::TileMapLayer;
use godot::classes::Viewport;
use godot::obj::Base;
use godot::obj::Gd;
use godot::obj::WithBaseField;
use godot::prelude::godot_api;
use godot::prelude::GodotClass;

use crate::defense_layer::TILE_SIZE;
use crate::ingame_state_tracker::GameplayState;
use crate::ingame_state_tracker::IngameStateTracker;
use crate::CellRules;

#[derive(GodotClass)]
#[class(base = Node,init)]
struct PlayerEnergy {
    base: Base<Node>,
    #[export]
    energy: i32,
    #[export]
    max_energy: i32,
    energy_timer: f64,
    #[export]
    energy_interval: f64,
    #[export]
    energy_per_interval: i32,
    #[export]
    display: Option<Gd<TextureProgressBar>>
}

#[godot_api]
impl INode for PlayerEnergy{
    fn physics_process(&mut self, delta: f64){
        self.energy_timer += delta;
        if self.energy_timer >= self.energy_interval{
            if self.energy<self.max_energy{
                self.gain_energy(self.energy_per_interval);
            }
            self.energy_timer = 0.;
        }
    }
}

#[godot_api]
impl PlayerEnergy {
    #[func]
    fn can_use(&self, cost: i32) -> bool {
        cost <= self.energy
    }
    #[func]
    fn try_use(&mut self, cost: i32) -> bool {
        if cost <= self.energy {
            self.energy -= cost;
            self.display.clone().unwrap().set_value(self.energy as f64);
            return true;
        }
        return false;
    }
    #[func]
    fn gain_energy(&mut self, amount: i32){
        self.energy += amount;
        self.energy = self.energy.min(self.max_energy);
        self.display.clone().unwrap().set_value(self.energy as f64);
    }
}

#[derive(GodotClass)]
#[class(base = Node, init)]
struct CellPatternToolbox {
    base: Base<Node>,
    #[export]
    patterns: Array<Gd<CellPattern>>,
    #[export]
    selected_pattern: u8,
    #[export]
    brush_tiles: Array<u16>,
    selected_tile: u8,
    #[export]
    gamestate: Option<Gd<IngameStateTracker>>,
    #[export]
    transparency_pane: Option<Gd<Control>>
}

#[godot_api]
impl INode for CellPatternToolbox {
    fn ready(&mut self){
        for _ in 0..self.patterns.len(){
            self.switch_next();
        }
    }
    fn process(&mut self, _delta: f64) {
        //detecting if actions were pressed and switch to the next/prev pattern
        if Input::singleton().is_action_just_pressed("next_pattern".into()) {
            self.switch_next();
        } else if Input::singleton().is_action_just_pressed("prev_pattern".into()) {
            self.switch_prev();
        }
        if Input::singleton().is_action_just_pressed("switch_draw_tile".into()).into(){
            self.switch_brush();
        }
        let is_drawing = self.get_game_state().bind().get_state() == GameplayState::DRAWING;
        self.get_transparency_pane().unwrap().set_visible(is_drawing);
    }
}

#[godot_api]
impl CellPatternToolbox {
    #[func]
    fn switch_to(&mut self, index: u8) {
        if index >= self.patterns.len() as u8 {
            return;
        }
        self.patterns
            .get(self.selected_pattern as usize)
            .unwrap()
            .bind_mut()
            .set_enabled(false);
        self.selected_pattern = index;
        self.patterns
            .get(self.selected_pattern as usize)
            .unwrap()
            .bind_mut()
            .set_enabled(true);
    }

    #[func]
    pub fn switch_next(&mut self) {
        self.switch_to(self.selected_pattern.add(1).rem_euclid(self.patterns.len() as u8));
    }
    #[func]
    pub fn switch_prev(&mut self) {
        self.switch_to(
            self.selected_pattern
                .add(self.patterns.len() as u8 - 1)
                .rem_euclid(self.patterns.len() as u8),
        );
    }
    #[func]
    pub fn switch_brush(&mut self){
        self.selected_tile += 1;
        self.selected_tile = self.selected_tile.rem_euclid(self.brush_tiles.len() as u8);
    }

    pub fn get_selected_brush_tile(&self) -> u16{
        self.get_brush_tiles().get(self.selected_tile as usize).expect("invalid tile input")
    }

    pub fn get_game_state(&self)-> Gd<IngameStateTracker>{
        self.gamestate.clone().unwrap()
    }
}

#[derive(GodotClass)]
#[class(base = TileMapLayer, init)]
struct CellPattern {
    base: Base<TileMapLayer>,
    #[export]
    bounds: Rect2i,
    #[export]
    target: Option<Gd<TileMapLayer>>,
    #[export]
    preview: Option<Gd<TileMapLayer>>,
    #[export]
    energy_source: Option<Gd<PlayerEnergy>>,
    last_mouse_pos: Vector2i,
    enabled: bool
}

#[godot_api]
impl ITileMapLayer for CellPattern {
    fn process(&mut self, _delta: f64) {
        let parent: Gd<CellPatternToolbox> = self.base().get_parent().expect("no parent???").try_cast().expect("object is not a child of CellPatternToolbox"); 
        let state = parent.bind().get_game_state().bind().get_state();
        if state == GameplayState::DRAWING{
            self.drawing_process();
        }else if state == GameplayState::DEFENDING{
            self.defending_process();
        }
    }
    fn ready(&mut self){
        let enabled = self.enabled;
        self.base_mut().set_visible(enabled);
    }
}

impl CellPattern{
    //process method when player is defending
    fn defending_process(&mut self){
        
        self.base_mut().set_visible(false);
        if !self.enabled {
            return;
        }

        let cost = self.get_cost();
        if !self.get_energy_source().unwrap().bind().can_use(cost){
            return;
        }
        let mouse_tile =
            Self::get_mouse_tile(self.base().get_viewport().expect("no valid viewport"));
        
        self.update_hover(self.preview.clone().unwrap(), mouse_tile);

        if Input::singleton().is_action_just_pressed("place_pattern".into()) {
            if self.get_energy_source().unwrap().bind_mut().try_use(self.get_cost()){
                self.place(self.target.clone().unwrap(), mouse_tile, true);
            }
            self.get_preview().unwrap().clear();
        }
    }
    //process method when player is drawing new towers
    fn drawing_process(&mut self){
        if !self.enabled {
            self.base_mut().set_visible(false);
            return;
        }
        self.base_mut().set_visible(true);
        
        let mouse_tile =
            Self::get_mouse_tile(self.base().get_viewport().expect("no valid viewport"));

        if Input::singleton().is_action_just_pressed("place_cell".into())
            || (Input::singleton().is_action_pressed("place_cell".into())
                && self.last_mouse_pos != mouse_tile)
        {
            let parent: Gd<CellPatternToolbox> = self.base().get_parent().expect("no parent???").try_cast().expect("object is not a child of CellPatternToolbox"); 
            let r = CellRules::from_id(parent.bind().get_selected_brush_tile());
            if r == CellRules::ForceEmpty{
                self.base_mut().set_cell(mouse_tile);
            }else{
                self.base_mut()
                    .set_cell_ex(mouse_tile)
                    .source_id(0)
                    .atlas_coords(r.to_atlas_coords())
                    .done();
            }
            self.last_mouse_pos = mouse_tile;
        }
    }
}

#[godot_api]
impl CellPattern {
    fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled{
            self.enabled = enabled;
            self.get_preview().unwrap().clear();
        }
    }
    fn get_mouse_tile(viewport: Gd<Viewport>) -> Vector2i {
        let pos = viewport
            .get_camera_2d()
            .expect("no valid camera2d")
            .get_global_mouse_position();
        (pos / TILE_SIZE).floor().cast_int()
    }
    #[func]
    pub fn get_cost(&self) -> i32 {
        let mut cost = 0;
        let cells = self.base().get_used_cells();
        for cell_pos in cells.iter_shared() {
            let tile = self.base().get_cell_tile_data(cell_pos);
            let cell_rules = CellRules::from_tile(tile);
            cost += cell_rules.to_cost();
        }
        cost
    }
    #[func]
    pub fn get_center(&self) -> Vector2 {
        let cells = self.base().get_used_cells();
        let mut average = Vector2::new(0., 0.);
        let mut count = 0;
        for cell_pos in cells.iter_shared() {
            count += 1;
            average = average.lerp(cell_pos.cast_float(), 1. / count as f32);
        }
        average
    }
    #[func]
    pub fn place(&self, mut target: Gd<TileMapLayer>, center: Vector2i, check_valid: bool) {
        let cells_center = self.get_center();
        let cells = self.base().get_used_cells();
        for cell_pos in cells.iter_shared() {
            let pos = cell_pos + center - cells_center.cast_int();
            let cell_data = self.base().get_cell_tile_data(cell_pos);
            let cell_rules = CellRules::from_tile(cell_data);

            let target_tile = CellRules::from_tile(target.get_cell_tile_data(pos));
            if target_tile.can_set() || !check_valid {
                target
                    .set_cell_ex(pos)
                    .source_id(0)
                    .atlas_coords(cell_rules.to_atlas_coords())
                    .done();
            }
        }
    }
    #[func]
    pub fn update_hover(&self, mut preview: Gd<TileMapLayer>, center: Vector2i) {
        preview.clear();
        self.place(preview, center, false);
    }
}
