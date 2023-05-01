use fyrox::animation;
use fyrox::animation::machine::node;
use fyrox::animation::spritesheet::SpriteSheetAnimation;
use fyrox::core::algebra::{Quaternion, ComplexField};
use fyrox::core::color::Color;
use fyrox::core::reflect::GetField;
use fyrox::gui::inspector::Value;
use fyrox::gui::text::Text;
use fyrox::plugin::PluginConstructor;
use fyrox::scene::collider::Collider;
use fyrox::scene::sprite::{Sprite, self};
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
    engine::{
        executor::Executor, resource_manager::ResourceManager, 
    },
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    impl_component_provider,
    plugin::{Plugin,PluginContext, PluginRegistrationContext},
    resource::texture::Texture,
    scene::{
        dim2::{rectangle::Rectangle, rigidbody::RigidBody},
        node::{Node, TypeUuidProvider},
        Scene, SceneLoader,
        loader::AsyncSceneLoader,
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection},
        dim2::rectangle::RectangleBuilder,
        light::{point::PointLightBuilder, spot::SpotLightBuilder, BaseLightBuilder},
        transform::TransformBuilder,
    },
    script::{ScriptContext, ScriptTrait},
    utils::log::Log,
    event_loop::ControlFlow,
    gui::{
        message::UiMessage,
        message::MessageDirection,
        text::{TextBuilder, TextMessage},
        widget::WidgetBuilder,
        UiNode,
    },
};

pub mod Points;

#[derive(Visit, Reflect, Debug, Clone,Default)]
pub struct Player{
    sprite: Handle<Node>,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    reset: bool,
    animations: Vec<SpriteSheetAnimation>,
    current_animation: u32,
    freemove: bool,
    death_line: f32,
}

impl_component_provider!(Player);



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
    fn on_start(&mut self, context: &mut ScriptContext) { 
    }

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


    fn on_update(&mut self, context: &mut ScriptContext) {
        
        if let Some(rigid_body) = context.scene.graph[context.handle].cast_mut::<RigidBody>() {

            if rigid_body.local_transform().position()[1] <= self.death_line {
                reset(&mut self.freemove,&mut self.current_animation,rigid_body,&mut self.reset);
            }
            
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

            if (self.freemove){

                rigid_body.set_ang_vel(-x_speed);
                if(self.move_up || self.move_down){
                    let rotation = rigid_body.local_transform().rotation().euler_angles();
                    rigid_body.set_lin_vel(Vector2::new(rotation.2.sin(),-rotation.2.cos())*y_speed*-1.0);
                 }
                else {
                    rigid_body.set_lin_vel(Vector2::new(0.0,0.0));
                }
            }
            else {
                if self.move_up && rigid_body.lin_vel().y.abs() <0.01 {
                    rigid_body.set_lin_vel(Vector2::new(x_speed,5.0));
                }
                else{
                    rigid_body.set_lin_vel(Vector2::new(x_speed,rigid_body.lin_vel().y));
                } 

                if x_speed != 0.0 {
                    self.current_animation = 1;
                } else {
                    self.current_animation = 0;
                }

                if rigid_body.lin_vel()[1].abs()>= 0.01 {
                    self.current_animation = 2;
                }
            }

            if(self.reset){
                reset(&mut self.freemove,&mut self.current_animation,rigid_body,&mut self.reset);
            }

            if let Some(sprite) = context.scene.graph.try_get_mut(self.sprite) {
            
                if x_speed != 0.0 && !self.freemove {
                    let local_transform = sprite.local_transform_mut();

                    let current_scale = **local_transform.scale();

                    local_transform.set_scale(Vector3::new(
                        // Just change X scaling to mirror player's sprite.
                        current_scale.x.copysign(-x_speed),
                        current_scale.y,
                        current_scale.z,
                    ));
                }
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

fn reset(freemove: &mut bool,current_animation: &mut u32,rigid_body: &mut RigidBody,reset: &mut bool){
    *freemove = !*freemove;
                if (*freemove){
                    *current_animation = 3;
                    rigid_body.set_gravity_scale(0.0);
                } else {
                    rigid_body.set_ang_vel(0.0);
                    rigid_body.set_gravity_scale(1.0);
                }

                *reset = false;
                let mut trans=rigid_body.local_transform().clone();
                trans.set_rotation(UnitQuaternion::identity());
                trans.set_position(Vector3::new(0.0, 2.0, 0.0));
                rigid_body.set_local_transform(trans);
}