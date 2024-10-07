use core::f64;

use godot::builtin::Rect2i;
use godot::builtin::Variant;
use godot::builtin::Vector2;
use godot::builtin::Vector2i;
use godot::classes::ITileMapLayer;
use godot::classes::TileMapLayer;
use godot::obj::Base;
use godot::obj::WithBaseField;
use godot::prelude::godot_api;
use godot::prelude::GodotClass;

use crate::CellRules;

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

pub const TILE_TYPE_DATA_LAYER: &str = "tile_type";
pub const TILE_SIZE: f32 = 64.;