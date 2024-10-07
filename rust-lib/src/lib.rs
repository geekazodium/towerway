use core::f64;
use core::panic;

use defense_layer::TILE_TYPE_DATA_LAYER;
use enemy_spawner::EnemyPath;
use godot::builtin::Callable;
use godot::builtin::GString;
use godot::builtin::Vector2;
use godot::builtin::Vector2i;
use godot::classes::Area2D;
use godot::classes::Camera2D;
use godot::classes::INode;
use godot::classes::INode2D;
use godot::classes::IPathFollow2D;
use godot::classes::ISprite2D;
use godot::classes::Node;
use godot::classes::Node2D;
use godot::classes::PackedScene;
use godot::classes::PathFollow2D;
use godot::classes::Sprite2D;
use godot::classes::TileData;
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
            Self::Empty => 0,
            Self::BasicFilled => 6,
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
    #[allow(unused)]
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
    fn get_event_index(&self) -> usize{
        match self {
            Self::CellCreate => 2,
            Self::ExtraOverpopulateDeath => 1,
            Self::OverpopulateDeath => 0
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
        let mut p = self.base().get_progress();
        let last_progress = self.base().get_progress_ratio();
        p += self.speed * delta as f32;
        self.base_mut().set_progress(p);
        if self.base().get_progress_ratio() < last_progress{
            let spawner:Gd<EnemyPath> = self.base().get_parent().unwrap().cast();
            spawner.bind().hit_player();
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
struct CameraScaler{
    base: Base<Node>,
    #[export]
    base_height: f32,
    #[export]
    camera: Option<Gd<Camera2D>>
}

#[godot_api]
impl INode for CameraScaler{
    fn process(&mut self, _delta: f64){
        let mut camera = self.get_camera().unwrap();
        let zoom = camera.get_viewport().unwrap().get_visible_rect().size.y / self.base_height;
        camera.set_zoom(Vector2::new(zoom, zoom));
    }
}

pub mod enemy_spawner;
pub mod cell_patterns;
pub mod player_health;
pub mod ingame_state_tracker;
pub mod defense_layer;
pub mod selected_hotbar;
pub mod pause_state_manager;