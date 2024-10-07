use godot::{builtin::Callable, classes::{BaseButton, INode, Node, PackedScene}, obj::{Base, Gd, WithBaseField}, prelude::{godot_api, GodotClass}};

#[derive(GodotClass)]
#[class(base = Node, init)]
struct PauseStateManager{
    base: Base<Node>,
    #[export]
    pause_scene: Option<Gd<PackedScene>>,
    pause_instance: Option<Gd<Node>>,
    #[export]
    pause_button: Option<Gd<BaseButton>>
}

#[godot_api]
impl INode for PauseStateManager {
    fn ready(&mut self){
        self.get_pause_button().expect("no pause button reference set!")
            .connect("pressed".into(),Callable::from_object_method(&self.to_gd(), "pause"));
    }
}

#[godot_api]
impl PauseStateManager {
    #[func]
    fn pause(&mut self){
        if self.pause_instance.is_none(){
            self.base().get_tree().unwrap().set_pause(true);
            let s = self.get_pause_scene().unwrap().instantiate();
            let mut node = s.clone().expect("invalid node, failed to create pause menu");
            node.connect("hidden".into(),Callable::from_object_method(&self.to_gd(), "unpause"));
            self.pause_instance = Some(node);
            
            self.base_mut().add_child(s.unwrap());
        }
    }
    #[func]
    fn unpause(&mut self){
        if self.pause_instance.is_some(){
            let to_delete = self.pause_instance.take();
            to_delete.unwrap().queue_free();
            self.base().get_tree().unwrap().set_pause(false);
        }
    }
}