//! Game project.
use fyrox::animation::spritesheet::SpriteSheetAnimation;
use fyrox::core::algebra::Quaternion;
use fyrox::plugin::PluginConstructor;
use fyrox::scene::collider::Collider;
use fyrox::scene::transform::Transform;
use fyrox::{
    core::{
        algebra::{Vector2, Vector3,UnitQuaternion},
        futures::executor::block_on,
        pool::Handle,
        reflect::prelude::*,
        uuid::{uuid, Uuid},
        visitor::prelude::*,
    },
    engine::resource_manager::ResourceManager,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    impl_component_provider,
    plugin::{Plugin,PluginContext, PluginRegistrationContext},
    resource::texture::Texture,
    scene::{
        dim2::{rectangle::Rectangle, rigidbody::RigidBody},
        node::{Node, TypeUuidProvider},
        Scene, SceneLoader,
        loader::AsyncSceneLoader,
    },
    script::{ScriptContext, ScriptTrait},
    utils::log::Log,
    event_loop::ControlFlow,
    gui::message::UiMessage,
};

pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn register(self: &GameConstructor, context: PluginRegistrationContext) {
        let script_constructors = &context.serialization_context.script_constructors;
        script_constructors.add::<Player>("Player");
    }

    fn create_instance(
        &self,
        override_scene: Handle<Scene>,
        context: PluginContext,
    ) -> Box<dyn Plugin> {
        Box::new(Game::new(override_scene, context))
    }
}

pub struct Game {
    scene: Handle<Scene>,
    loader: Option<AsyncSceneLoader>,
}

impl Game {
    pub fn new(override_scene: Handle<Scene>, context: PluginContext) -> Self {
        let mut loader = None;
        let scene = if override_scene.is_some() {
            override_scene
        } else {
            loader = Some(AsyncSceneLoader::begin_loading(
                "data/scene.rgs".into(),
                context.serialization_context.clone(),
                context.resource_manager.clone(),
            ));
            Default::default()
        };

        Self { scene, loader }
    }
}

impl Plugin for Game {
    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, context: &mut PluginContext, _control_flow: &mut ControlFlow) {
         if let Some(loader) = self.loader.as_ref() {
            if let Some(result) = loader.fetch_result() {
                match result {
                    Ok(scene) => {
                        self.scene = context.scenes.add(scene);
                    }
                    Err(err) => Log::err(err),
                }
            }
        }
    
        // Add your global update code here.
    }

    fn on_os_event(
        &mut self,
        _event: &Event<()>,
        _context: PluginContext,
        _control_flow: &mut ControlFlow,
    ) {
        // Do something on OS event here.
    }

    fn on_ui_message(
        &mut self,
        _context: &mut PluginContext,
        _message: &UiMessage,
        _control_flow: &mut ControlFlow,
    ) {
        // Handle UI events here.
    }
}
#[derive(Visit, Reflect, Debug, Clone,Default)]
struct Player{
    sprite: Handle<Node>,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    reset: bool,
    animations: Vec<SpriteSheetAnimation>,
    current_animation: u32,
}

impl_component_provider!(Player,);



impl TypeUuidProvider for Player {
    // Returns unique script id for serialization needs.
    fn type_uuid() -> Uuid {
        uuid!("c5671d19-9f1a-4286-8486-add4ebaadaec")
    }
}

impl ScriptTrait for Player {
    // Called once at initialization.
    fn on_init(&mut self, context: &mut ScriptContext) {}
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) { }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput { input, .. } = event {
                if let Some(keycode) = input.virtual_keycode {
                    let is_pressed = input.state == ElementState::Pressed;
        
                    match keycode {
                        VirtualKeyCode::A => self.move_left = is_pressed,
                        VirtualKeyCode::D => self.move_right = is_pressed,
                        VirtualKeyCode::W => self.move_up = is_pressed,
                        VirtualKeyCode::S => self.move_down = is_pressed,
                        VirtualKeyCode::Space =>self.reset = is_pressed,
                        _ => (),
                    }
                }
            }
        }
    }

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        // The script can be assigned to any scene node, but we assert that it will work only with
        // 2d rigid body nodes.
        if let Some(rigid_body) = context.scene.graph[context.handle].cast_mut::<RigidBody>() {
            let x_speed = match (self.move_left, self.move_right) {
                (true, false) => 3.0,
                (false, true) => -3.0,
                _ => 0.0,
            };
            let y_speed:f32 = match (self.move_down, self.move_up){
                (true, false) => -3.0,
                (false, true) => 3.0,
                _ => 0.0,
            };
            rigid_body.set_ang_vel(-x_speed);
            if(self.move_up || self.move_down){
                let rotation = rigid_body.local_transform().rotation().euler_angles();
                rigid_body.set_lin_vel(Vector2::new(rotation.2.sin(),-rotation.2.cos())*y_speed*-1.0);
            }
            else {
                rigid_body.set_lin_vel(Vector2::new(0.0,0.0));
            }

            
            if(self.reset){
                self.reset=false;
                let mut trans=rigid_body.local_transform().clone();
                trans.set_rotation(UnitQuaternion::identity());
                trans.set_position(Vector3::new(0.0, 2.0, 0.0));
                rigid_body.set_local_transform(trans);
            }   
            if let Some(current_animation) = self.animations.get_mut(self.current_animation as usize) {
                current_animation.update(context.dt);
    
                if let Some(sprite) = context
                    .scene
                    .graph
                    .try_get_mut(self.sprite)
                    .and_then(|n| n.cast_mut::<Rectangle>())
                {
                    // Set new frame to the sprite.
                    sprite.set_texture(current_animation.texture());
                    sprite.set_uv_rect(
                        current_animation
                            .current_frame_uv_rect()
                            .unwrap_or_default(),
                    );
                }
            }
        }
    }

    fn restore_resources(&mut self, resource_manager: ResourceManager) {
        for animation in self.animations.iter_mut() {
            animation.restore_resources(&resource_manager);
        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
    
}

