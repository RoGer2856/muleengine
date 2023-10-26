macro_rules! renderer_object_mod {
    ( $mod_name:ident, $trait_name:ident, $handler_name:ident, $release_fn:ident, $trait_name_literal:literal ) => {
        pub mod $mod_name {
            use std::{cmp::Ordering, fmt::Debug, sync::Arc};

            use crate::{
                bytifex_utils::result_option_inspect::ResultInspector,
                bytifex_utils::{cast::AsAny, containers::object_pool::ObjectPoolIndex},
                renderer::renderer_system::renderer_decoupler,
            };

            pub trait $trait_name: AsAny + Sync + Send + 'static {}

            #[derive(Clone)]
            pub(crate) struct HandlerDestructor {
                pub(crate) object_pool_index: ObjectPoolIndex,
                renderer_client: renderer_decoupler::Client,
            }

            #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
            pub struct $handler_name(pub(crate) Arc<HandlerDestructor>);

            impl $handler_name {
                pub fn new(
                    object_pool_index: ObjectPoolIndex,
                    renderer_client: renderer_decoupler::Client,
                ) -> Self {
                    Self(Arc::new(HandlerDestructor {
                        object_pool_index,
                        renderer_client,
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
                    let fut = self.renderer_client.$release_fn(self.object_pool_index);
                    tokio::spawn(async move {
                        let _ = fut.await.inspect_err(|e| {
                            log::error!("Release {}, msg = {e:?}", $trait_name_literal)
                        });
                    });
                }
            }
        }
    };
}

renderer_object_mod!(
    renderer_camera,
    RendererCamera,
    RendererCameraHandler,
    release_camera,
    "RendererCamera"
);

renderer_object_mod!(
    renderer_group,
    RendererGroup,
    RendererGroupHandler,
    release_renderer_group,
    "RendererGroup"
);

renderer_object_mod!(
    renderer_layer,
    RendererLayer,
    RendererLayerHandler,
    release_renderer_layer,
    "RendererLayer"
);

renderer_object_mod!(
    renderer_material,
    RendererMaterial,
    RendererMaterialHandler,
    release_material,
    "RendererMaterial"
);

renderer_object_mod!(
    renderer_mesh,
    RendererMesh,
    RendererMeshHandler,
    release_mesh,
    "RendererMesh"
);

renderer_object_mod!(
    renderer_object,
    RendererObject,
    RendererObjectHandler,
    release_renderer_object,
    "RendererObject"
);

renderer_object_mod!(
    renderer_shader,
    RendererShader,
    RendererShaderHandler,
    release_shader,
    "RendererShader"
);

renderer_object_mod!(
    renderer_transform,
    RendererTransform,
    RendererTransformHandler,
    release_transform,
    "RendererTransform"
);
