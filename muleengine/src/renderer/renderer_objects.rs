macro_rules! renderer_object_mod {
    ( $mod_name:ident, $trait_name:ident, $handler_name:ident, $command:ident, $trait_name_literal:literal ) => {
        pub mod $mod_name {
            use std::{cmp::Ordering, fmt::Debug, sync::Arc};

            use crate::{
                containers::object_pool::ObjectPoolIndex,
                prelude::{AsAny, ResultInspector},
                renderer::renderer_command::Command,
                sync::command_channel::CommandSender,
            };

            pub trait $trait_name: AsAny + Sync + Send + 'static {}

            #[derive(Clone)]
            pub(crate) struct HandlerDestructor {
                pub(crate) object_pool_index: ObjectPoolIndex,
                command_sender: CommandSender<Command>,
            }

            #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
            pub struct $handler_name(pub(crate) Arc<HandlerDestructor>);

            impl $handler_name {
                pub fn new(
                    object_pool_index: ObjectPoolIndex,
                    command_sender: CommandSender<Command>,
                ) -> Self {
                    Self(Arc::new(HandlerDestructor {
                        object_pool_index,
                        command_sender,
                    }))
                }
            }

            impl Debug for HandlerDestructor {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct("HandlerDestructor")
                        .field("object_pool_index", &self.object_pool_index)
                        .finish()
                }
            }

            impl Eq for HandlerDestructor {}

            impl PartialEq for HandlerDestructor {
                fn eq(&self, other: &Self) -> bool {
                    self.object_pool_index == other.object_pool_index
                }
            }

            impl Ord for HandlerDestructor {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.object_pool_index.cmp(&other.object_pool_index)
                }
            }

            impl PartialOrd for HandlerDestructor {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl Drop for HandlerDestructor {
                fn drop(&mut self) {
                    let _ = self
                        .command_sender
                        .send(Command::$command {
                            object_pool_index: self.object_pool_index,
                        })
                        .inspect_err(|e| {
                            log::error!("Release {}, msg = {e:?}", $trait_name_literal)
                        });
                }
            }
        }
    };
}

renderer_object_mod!(
    renderer_camera,
    RendererCamera,
    CameraHandler,
    ReleaseCamera,
    "RendererCamera"
);

renderer_object_mod!(
    renderer_group,
    RendererGroup,
    RendererGroupHandler,
    ReleaseRendererGroup,
    "RendererGroup"
);

renderer_object_mod!(
    renderer_layer,
    RendererLayer,
    RendererLayerHandler,
    ReleaseRendererLayer,
    "RendererLayer"
);

renderer_object_mod!(
    renderer_material,
    RendererMaterial,
    MaterialHandler,
    ReleaseMaterial,
    "RendererMaterial"
);

renderer_object_mod!(
    renderer_mesh,
    RendererMesh,
    MeshHandler,
    ReleaseMesh,
    "RendererMesh"
);

renderer_object_mod!(
    renderer_object,
    RendererObject,
    RendererObjectHandler,
    ReleaseRendererObject,
    "RendererObject"
);

renderer_object_mod!(
    renderer_shader,
    RendererShader,
    ShaderHandler,
    ReleaseShader,
    "RendererShader"
);

renderer_object_mod!(
    renderer_transform,
    RendererTransform,
    TransformHandler,
    ReleaseTransform,
    "RendererTransform"
);
