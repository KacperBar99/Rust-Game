use fyrox::animation;
use fyrox::animation::machine::node;
use fyrox::animation::spritesheet::SpriteSheetAnimation;
use fyrox::core::algebra::{ComplexField, Quaternion, distance};
use fyrox::core::color::Color;
use fyrox::core::reflect::GetField;
use fyrox::gui::inspector::Value;
use fyrox::gui::text::Text;
use fyrox::plugin::PluginConstructor;
use fyrox::scene::collider::Collider;
use fyrox::scene::dim2::rigidbody;
use fyrox::scene::sprite::{self, Sprite};
use fyrox::scene::transform::Transform;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector2, Vector3},
        futures::executor::block_on,
        pool::Handle,
        reflect::prelude::*,
        uuid::{uuid, Uuid},
        visitor::prelude::*,
    },
    engine::{executor::Executor, resource_manager::ResourceManager},
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    gui::{
        message::MessageDirection,
        message::UiMessage,
        text::{TextBuilder, TextMessage},
        widget::WidgetBuilder,
        UiNode,
    },
    impl_component_provider,
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    resource::texture::Texture,
    scene::{
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection},
        dim2::rectangle::RectangleBuilder,
        dim2::{rectangle::Rectangle, rigidbody::RigidBody},
        light::{point::PointLightBuilder, spot::SpotLightBuilder, BaseLightBuilder},
        loader::AsyncSceneLoader,
        node::{Node, TypeUuidProvider},
        transform::TransformBuilder,
        Scene, SceneLoader,
    },
    script::{ScriptContext, ScriptTrait},
    utils::log::Log,
};

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct Spikes {
    size: f32,
    check_point: Handle<Node>,
    player: Handle<Node>,
    sprite: Handle<Node>,
    animations: Vec<SpriteSheetAnimation>,
    x:f32,
    y:f32,
}

impl_component_provider!(Spikes);

impl TypeUuidProvider for Spikes {
    fn type_uuid() -> Uuid {
        uuid!("c5671f19-9f2b-4286-8486-add4efaadaec")
    }
}

impl ScriptTrait for Spikes {
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
    // Called once at initialization.
    fn on_init(&mut self, context: &mut ScriptContext) {}

    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) {
        if let Some(rigid_body) = context.scene.graph[context.handle].cast_mut::<Rectangle>() {
            self.x = rigid_body.local_transform().position()[0];
            self.y = rigid_body.local_transform().position()[1];
        }
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {}

    fn on_update(&mut self, context: &mut ScriptContext) {
        let mut checkpoint_x = 0.0;
        let mut checkpoint_y = 0.0;
        if let Some(transform) = context.scene.graph.try_get(self.check_point) {
            checkpoint_x = transform.local_transform().position()[0];
            checkpoint_y = transform.local_transform().position()[1];
        }
        if let Some(rigid_body) = context.scene.graph[context.handle].cast_mut::<RigidBody>() {
            self.x  = rigid_body.local_transform().position()[0];             
            self.y  = rigid_body.local_transform().position()[1];

            if let Some(current_animation) = self.animations.get_mut(0) {
                current_animation.update(context.dt);

                if let Some(sprite) = context
                    .scene
                    .graph
                    .try_get_mut(self.sprite)
                    .and_then(|n| n.cast_mut::<Rectangle>())
                {
                    sprite.set_texture(current_animation.texture());
                    sprite.set_uv_rect(
                        current_animation
                            .current_frame_uv_rect()
                            .unwrap_or_default(),
                    );
                }
            }
        }
        if let Some(player) = context.scene.graph[self.player].cast_mut::<RigidBody>() {
            let x = player.local_transform().position()[0];
            let y = player.local_transform().position()[1];
            let distance = ((self.x-x).abs()+(self.y-y).abs()).sqrt();

            if distance <= self.size {
                let mut trans=player.local_transform().clone();
                trans.set_position(Vector3::new(checkpoint_x, checkpoint_y, 0.0));
                player.set_local_transform(trans);
            }
        }
    }
    
    fn restore_resources(&mut self, resource_manager: ResourceManager) {
        for animation in self.animations.iter_mut() {
            animation.restore_resources(&resource_manager);
        }
    }
}
