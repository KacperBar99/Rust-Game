use fyrox::animation;
use fyrox::animation::machine::node;
use fyrox::animation::spritesheet::SpriteSheetAnimation;
use fyrox::core::algebra::Quaternion;
use fyrox::core::color::Color;
use fyrox::core::reflect::GetField;
use fyrox::gui::inspector::Value;
use fyrox::gui::text::Text;
use fyrox::plugin::PluginConstructor;
use fyrox::scene::collider::Collider;
use fyrox::scene::sprite::{self, Sprite};
use fyrox::scene::transform::{self, Transform};
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
use std::convert::TryInto;

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct Points {
    rigidbody: Handle<Node>,
    value: i32,
    values: Vec<SpriteSheetAnimation>,
    text: Handle<Node>,
    text2: Handle<Node>,
    text3: Handle<Node>,
}
impl_component_provider!(Points);

impl TypeUuidProvider for Points {
    fn type_uuid() -> Uuid {
        uuid!("c5671e19-9f1a-4286-8486-add4efaadaec")
    }
}

impl ScriptTrait for Points {
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }

    fn restore_resources(&mut self, resource_manager: ResourceManager) {
        for value in self.values.iter_mut() {
            value.restore_resources(&resource_manager);
        }
    }

    fn on_update(&mut self, context: &mut ScriptContext) {
        let mut pos = 1.0;
        if let Some(body) = context.scene.graph.try_get(self.rigidbody) {
            pos = body.as_rigid_body2d().local_transform().position()[0];
        }

        self.value = (pos).abs().round() as i32;

        if let Some(animation) = self.values.get_mut(0) {
            let mut val: usize = (self.value).try_into().unwrap();
            val = (val / 100) % 10;
            animation.set_current_frame(val);
            if let Some(sprite) = context
                .scene
                .graph
                .try_get_mut(self.text)
                .and_then(|n| n.cast_mut::<Rectangle>())
            {
                sprite.set_texture(animation.texture());
                sprite.set_uv_rect(animation.current_frame_uv_rect().unwrap_or_default());
            }
            /////////////

            let mut val: usize = (self.value).try_into().unwrap();
            val = (val / 10) % 10;
            animation.set_current_frame(val);

            if let Some(sprite) = context
                .scene
                .graph
                .try_get_mut(self.text2)
                .and_then(|n| n.cast_mut::<Rectangle>())
            {
                sprite.set_texture(animation.texture());
                sprite.set_uv_rect(animation.current_frame_uv_rect().unwrap_or_default());
            }
            //////////////////
            let mut val: usize = (self.value).try_into().unwrap();
            val = val % 10;
            animation.set_current_frame(val);
            if let Some(sprite) = context
                .scene
                .graph
                .try_get_mut(self.text3)
                .and_then(|n| n.cast_mut::<Rectangle>())
            {
                sprite.set_texture(animation.texture());
                sprite.set_uv_rect(animation.current_frame_uv_rect().unwrap_or_default());
            }
        }
    }
}
