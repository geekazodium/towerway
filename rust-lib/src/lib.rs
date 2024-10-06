use core::f64;
use core::panic;

use godot::builtin::Callable;
use godot::builtin::GString;
use godot::builtin::Rect2i;
use godot::builtin::Variant;
use godot::builtin::Vector2;
use godot::builtin::Vector2i;
use godot::classes::Area2D;
use godot::classes::INode;
use godot::classes::INode2D;
use godot::classes::IPathFollow2D;
use godot::classes::ISprite2D;
use godot::classes::ITileMapLayer;
use godot::classes::Input;
use godot::classes::Node;
use godot::classes::Node2D;
use godot::classes::PackedScene;
use godot::classes::PathFollow2D;
use godot::classes::Sprite2D;
use godot::classes::TileData;
use godot::classes::TileMapLayer;
use godot::classes::Viewport;
use godot::init::gdextension;
use godot::init::ExtensionLibrary;
use godot::obj::Base;
use godot::obj::Gd;
use godot::obj::WithBaseField;
use godot::prelude::godot_api;
use godot::prelude::GodotClass;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base = TileMapLayer,init)]
struct DefenseLayer{
    base: Base<TileMapLayer>,
    #[export]
    update_phys_interval: i32,
    phys_clock: i32,
    #[export]
    rect: Rect2i
}

#[godot_api]
impl ITileMapLayer for DefenseLayer {
    fn physics_process(&mut self, _delta: f64){
        self.phys_clock += 1;
        if self.phys_clock >= self.update_phys_interval{
            self.phys_clock = 0;
            self.update_tiles();
        }
    }
}

#[godot_api]
impl DefenseLayer{
    #[signal]
    fn overpopulate_death(pos: Vector2);
    #[signal]
    fn extra_overpopulate_death(pos: Vector2);
    #[signal]
    fn cell_create(pos: Vector2);
}

impl DefenseLayer{
    fn update_tiles(&mut self){
        let rect = self.rect.clone();
        let pos = rect.position;
        let range = rect.size;

        let mut cells = Vec::new();
        for y in 0..range.y{
            for x in 0..range.x{
                let tile_pos = Vector2i::new(x, y)+pos;
                let tile = self.base().get_cell_tile_data(tile_pos);
                cells.push(CellRules::from_tile(tile));
            }
        }

        //godot_print!("{:#?}",cells);

        let neighbor_coords = vec![
            Vector2i::new(1, 1),
            Vector2i::new(1, 0),
            Vector2i::new(1, -1),
            Vector2i::new(0, -1),
            Vector2i::new(-1, -1),
            Vector2i::new(-1, 0),
            Vector2i::new(-1, 1),
            Vector2i::new(0, 1)
        ];

        for y in 0..range.y{
            for x in 0..range.x{
                let i = Self::map_vec_to_index(rect, Vector2i::new(x, y));
                let mut neighbors = vec![];
                for n in &neighbor_coords{
                    let c = Vector2i::new(x, y) + *n;
                    if c.x < 0 || c.x >= range.x{
                        neighbors.push(CellRules::ForceEmpty);
                    }else if c.y < 0 || c.y >= range.y {
                        neighbors.push(CellRules::ForceEmpty);
                    }else{
                        neighbors.push(cells.get(Self::map_vec_to_index(rect,c)).unwrap().clone());
                    }
                }
                let cell_rules = cells.get(i).unwrap().clone();
                let events = cell_rules.events(&neighbors);
                let tile_pos = Vector2i::new(x, y)+pos;
                for e in events{
                    self.base_mut().emit_signal(e.get_event_name().into(),&[Variant::from((tile_pos.cast_float() + Vector2::new(0.5, 0.5)) * TILE_SIZE)]);
                }
                let t = cell_rules.next_cell(&neighbors);
                if t.can_set(){
                    self.base_mut().set_cell_ex(tile_pos).atlas_coords(t.to_atlas_coords()).source_id(0).done();
                }
            }
        }
    }
    fn map_vec_to_index(rect: Rect2i,vec: Vector2i)-> usize{
        let x = vec.x as usize;
        let cy = (vec.y * rect.size.x) as usize;
        x + cy
    }
}

const TILE_TYPE_DATA_LAYER: &str = "tile_type";
const TILE_SIZE: f32 = 32.;

#[derive(PartialEq, Eq, Clone, Debug)]
enum CellRules{
    Empty,
    ForceEmpty,
    BasicFilled,
    PermaCell,
}

impl CellRules{
    fn from_tile(tile: Option<Gd<TileData>>) -> Self{
        let layer_name: GString = TILE_TYPE_DATA_LAYER.into();
        if tile.is_none(){
            return CellRules::ForceEmpty;
        }
        let id:u16 = tile.unwrap().get_custom_data(layer_name.clone()).to();
        CellRules::from_id(id)
    }
    #[allow(unused)]
    fn to_id(&self)->u16{
        match self{
            Self::ForceEmpty=>0,
            Self::Empty=>1,
            Self::BasicFilled=>2,
            Self::PermaCell=>3,
        }
    }
    fn to_cost(&self)->i32{
        match self{
            Self::Empty => 1,
            Self::BasicFilled => 2,
            Self::PermaCell => panic!("user probably shouldn't be able to place these, too op"),
            Self::ForceEmpty => 0
        }
    }
    fn can_set(&self)-> bool{
        match self {
            Self::ForceEmpty=> false,
            _default=>true
        }
    }
    fn to_atlas_coords(&self) -> Vector2i{
        match self{
            Self::ForceEmpty=>panic!("can't set forced empty cell"),
            Self::Empty=>Vector2i::new(0, 0),
            Self::BasicFilled=>Vector2i::new(1, 0),
            Self::PermaCell=>Vector2i::new(2, 0)
        }
    }
    fn from_id(id: u16) -> Self{
        match id {
            0=>Self::ForceEmpty,
            1=>Self::Empty,
            2=>Self::BasicFilled,
            3=>Self::PermaCell,
            _default=> panic!("invalid id")
        }
    }
    fn next_cell(&self,neighbors: &Vec<CellRules>) -> Self{
        match self{
            Self::Empty=>{
                if Self::count_non_empty(neighbors)==3{
                    return Self::BasicFilled;
                }
                Self::Empty
            }
            Self::BasicFilled=>{
                let c = Self::count_non_empty(neighbors);
                if c<=3 && c>=2{
                    return Self::BasicFilled;
                }
                Self::Empty
            }
            Self::ForceEmpty=>Self::ForceEmpty,
            Self::PermaCell=>Self::PermaCell
        }
    }
    fn user_replaceable(&self)-> bool{
        match self {
            Self::Empty=>true,
            Self::BasicFilled=>true,
            _default=>false
        }
    }
    fn events(&self, neighbors: &Vec<CellRules>)->Vec<CellEvents>{
        match self{
            Self::Empty=>{
                let c = Self::count_non_empty(neighbors);
                if c<=3 && c>=2{
                    return vec![CellEvents::CellCreate];
                }
                vec![]
            },
            Self::PermaCell=>vec![],
            Self::ForceEmpty=>vec![],
            Self::BasicFilled=>{
                if Self::count_non_empty(neighbors) > 4{
                    return vec![CellEvents::ExtraOverpopulateDeath];
                }
                if Self::count_non_empty(neighbors) > 3{
                    return vec![CellEvents::OverpopulateDeath];
                }
                vec![]
            }
        }
    }
    fn count_non_empty(neighbors: &Vec<CellRules>) -> u8{
        let mut c:u8 = 8;
        for n in neighbors.iter(){
            if *n == Self::Empty || *n == Self::ForceEmpty{
                c -=1;
            }
        }
        c
    }
}

enum CellEvents{
    OverpopulateDeath,
    ExtraOverpopulateDeath,
    CellCreate,
}

impl CellEvents{
    fn get_event_name(&self) -> &str{
        match self{
            Self::OverpopulateDeath=>"overpopulate_death",
            Self::ExtraOverpopulateDeath=>"extra_overpopulate_death",
            Self::CellCreate=>"cell_create"
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node,init)]
struct EventFire{
    base: Base<Node>,
    #[export]
    projectile: Option<Gd<PackedScene>>,
    #[export]
    event_name: GString
}

#[godot_api]
impl INode for EventFire{
    fn ready(&mut self){
        let mut parent = self.base().get_parent().unwrap();
        parent.connect(self.event_name.to_string().into(),Callable::from_object_method(&self.base_mut(), "unleash_heck"));
    }
}

#[godot_api]
impl EventFire {
    #[func]
    fn unleash_heck(&mut self, pos: Vector2){
        let mut instance:Gd<Node2D> = self.get_projectile().unwrap().instantiate().unwrap().cast();
        self.base_mut().add_child(instance.clone());
        instance.global_translate(pos);
    }
}

#[derive(GodotClass)]
#[class(base = Sprite2D, init)]
struct SmallProjectile{
    base: Base<Sprite2D>,
    #[export]
    speed: f32,
    #[export]
    direction: f32,
    #[export]
    power: i32,
    #[export]
    hitbox: Option<Gd<Area2D>>,
    disabled: bool
}

#[godot_api]
impl ISprite2D for SmallProjectile {
    fn ready(&mut self){
        self.disabled = false;
        self.hitbox.clone().expect("missing hitbox!").connect("area_entered".into(), Callable::from_object_method(&self.base_mut(), "on_area_entered"));
    }
    fn physics_process(&mut self, delta: f64){
        let direction = self.direction * std::f32::consts::PI * 2.;
        let speed = self.speed;
        self.base_mut().move_local_x(direction.cos() * speed * delta as f32);
        self.base_mut().move_local_y(direction.sin() * speed * delta as f32);
    }
}

#[godot_api]
impl SmallProjectile{
    #[func]
    fn on_area_entered(&mut self, area: Gd<Area2D>){
        if self.disabled{
            return;
        }
        let parent = area.get_parent().unwrap();
        let mut damageable = Damageable::find(parent);
        damageable.bind_mut().take_damage(self.power);
        self.disabled = true;
        self.base_mut().queue_free();
    }
}

#[derive(GodotClass)]
#[class(base = Node,init)]
struct Damageable{
    base: Base<Node>,
    #[export]
    max_health: i32,
    #[export]
    current_health: i32
}

impl Damageable {
    pub fn take_damage(&mut self, amount: i32){
        if amount >= self.current_health{
            self.base_mut().get_parent().unwrap().queue_free();
        }
        self.current_health -= amount;
    }
    pub fn find(parent: Gd<Node>)->Gd<Damageable>{
        parent.find_child("Damageable".into()).unwrap().cast()
    }
}

#[derive(GodotClass)]
#[class(base = PathFollow2D, init)]
struct BasicEnemy{
    base: Base<PathFollow2D>,
    #[export]
    speed: f32,
}

#[godot_api]
impl IPathFollow2D for BasicEnemy{
    fn physics_process(&mut self, delta: f64){
        let mut p = self.base_mut().get_progress();
        let last_progress = p;
        p += self.speed * delta as f32;
        self.base_mut().set_progress(p);
        if self.base().get_progress() < last_progress{
            self.base_mut().queue_free();
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node2D, init)]
struct DeleteAfter{
    base: Base<Node2D>,
    #[export]
    delay: f32
}

#[godot_api]
impl INode2D for DeleteAfter{
    fn physics_process(&mut self, delta: f64){
        self.delay -= delta as f32;
        if self.delay < 0.{
            self.base_mut().queue_free();
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node, init)]
struct EnemySpawner{
    base: Base<Node>,
    #[export]
    interval: f64,
    #[export]
    timer: f64,
    #[export]
    enemies: Option<Gd<PackedScene>>,
    #[export]
    enabled: bool
}

#[godot_api]
impl INode for EnemySpawner{
    fn physics_process(&mut self, delta: f64){
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
#[class(base = TileMapLayer, init)]
struct CellPattern{
    base: Base<TileMapLayer>,
    #[export]
    bounds: Rect2i,
    last_mouse_pos: Vector2i,
    #[export]
    target: Option<Gd<TileMapLayer>>,
    #[export]
    preview: Option<Gd<TileMapLayer>>
}

#[godot_api]
impl ITileMapLayer for CellPattern{
    fn process(&mut self, _delta: f64){
        let mouse_tile = Self::get_mouse_tile(self.base().get_viewport().expect("no valid viewport"));
        if Input::singleton().is_action_just_pressed("place_cell".into()) || 
            (Input::singleton().is_action_pressed("place_cell".into()) && self.last_mouse_pos != mouse_tile){
            
            let rules = CellRules::from_tile(self.base().get_cell_tile_data(mouse_tile));
            if rules.user_replaceable(){
                self.base_mut().set_cell_ex(mouse_tile).source_id(0).atlas_coords(CellRules::BasicFilled.to_atlas_coords()).done();
            }
            self.last_mouse_pos = mouse_tile;
        }

        if Input::singleton().is_action_just_pressed("place_pattern".into()){
            self.place(self.target.clone().unwrap(), mouse_tile, true);
        }
        self.update_hover(self.preview.clone().unwrap(), mouse_tile);
    }
}


#[godot_api]
impl CellPattern{
    fn get_mouse_tile(viewport: Gd<Viewport>)-> Vector2i{
        let pos = viewport.get_camera_2d().expect("no valid camera2d").get_global_mouse_position();
        (pos / TILE_SIZE).floor().cast_int()
    }
    #[func]
    pub fn get_cost(&self) -> i32{
        let mut cost = 0;
        let cells = self.base().get_used_cells();
        for cell_pos in cells.iter_shared(){
            let tile = self.base().get_cell_tile_data(cell_pos);
            let cell_rules = CellRules::from_tile(tile);
            cost += cell_rules.to_cost();
        }
        cost
    }
    #[func]
    pub fn get_center(&self)-> Vector2{
        let cells = self.base().get_used_cells();
        let mut average = Vector2::new(0.,0.);
        let mut count = 0;
        for cell_pos in cells.iter_shared(){
            count += 1;
            average = average.lerp(cell_pos.cast_float(), 1./count as f32);
        }
        average
    }
    #[func]
    pub fn place(&self, mut target: Gd<TileMapLayer>, center: Vector2i, check_valid: bool){
        let cells_center = self.get_center();
        let cells = self.base().get_used_cells();
        for cell_pos in cells.iter_shared(){
            let pos = cell_pos + center - cells_center.cast_int();
            let cell_data = self.base().get_cell_tile_data(cell_pos);
            let cell_rules = CellRules::from_tile(cell_data);

            let target_tile = CellRules::from_tile(target.get_cell_tile_data(pos));
            if target_tile.can_set() || !check_valid{
                target.set_cell_ex(pos).source_id(0).atlas_coords(cell_rules.to_atlas_coords()).done();
            }
        }
    }
    #[func]
    pub fn update_hover(&self, mut preview: Gd<TileMapLayer>,center: Vector2i){
        preview.clear();
        self.place(preview, center, false);
    }
}

#[derive(GodotClass)]
#[class(base = Node2D,init)]
struct TickingGroup{
    base: Base<Node2D>
}