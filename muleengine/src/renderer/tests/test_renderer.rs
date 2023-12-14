use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::{Duration, Instant},
};

use bytifex_utils::sync::{
    app_loop_state::{AppLoopState, AppLoopStateWatcher},
    types::{arc_rw_lock_new, ArcRwLock},
};
use vek::Transform;

use crate::{
    mesh::{Material, Mesh},
    renderer::{
        renderer_impl::RendererImpl,
        renderer_pipeline_step_impl,
        renderer_system::RendererClient,
        renderer_system::{AsyncRenderer, SyncRenderer},
        RendererCamera, RendererGroup, RendererLayer, RendererMaterial, RendererMesh,
        RendererObject, RendererShader, RendererTransform,
    },
    system_container::System,
    test_utils::sendable_ptr::SendablePtr,
};

#[derive(Clone)]
pub struct TestRendererImpl {
    pub renderer_steps: Vec<renderer_pipeline_step_impl::RendererPipelineStepImpl>,
    pub renderer_groups: ArcRwLock<BTreeMap<SendablePtr<dyn RendererGroup>, TestRendererGroupImpl>>,
    pub renderer_layers: ArcRwLock<BTreeMap<SendablePtr<dyn RendererLayer>, TestRendererLayerImpl>>,
    pub transforms:
        ArcRwLock<BTreeMap<SendablePtr<dyn RendererTransform>, Transform<f32, f32, f32>>>,
    pub materials: ArcRwLock<BTreeMap<SendablePtr<dyn RendererMaterial>, Material>>,
    pub shaders: ArcRwLock<BTreeMap<SendablePtr<dyn RendererShader>, String>>,
    pub meshes: ArcRwLock<BTreeMap<SendablePtr<dyn RendererMesh>, Arc<Mesh>>>,
    pub cameras: ArcRwLock<BTreeSet<SendablePtr<dyn RendererCamera>>>,

    pub renderer_objects: ArcRwLock<BTreeSet<SendablePtr<dyn RendererObject>>>,
}

impl TestRendererImpl {
    pub fn new() -> Self {
        Self {
            renderer_steps: Vec::new(),
            renderer_groups: arc_rw_lock_new(BTreeMap::new()),
            renderer_layers: arc_rw_lock_new(BTreeMap::new()),
            transforms: arc_rw_lock_new(BTreeMap::new()),
            materials: arc_rw_lock_new(BTreeMap::new()),
            shaders: arc_rw_lock_new(BTreeMap::new()),
            meshes: arc_rw_lock_new(BTreeMap::new()),
            cameras: arc_rw_lock_new(BTreeSet::new()),
            renderer_objects: arc_rw_lock_new(BTreeSet::new()),
        }
    }
}

#[derive(Clone)]
pub struct TestRendererLayerImpl {
    pub renderer_groups: ArcRwLock<BTreeSet<SendablePtr<dyn RendererGroup>>>,
}

impl RendererLayer for TestRendererLayerImpl {}

impl TestRendererLayerImpl {
    pub fn new() -> Self {
        Self {
            renderer_groups: arc_rw_lock_new(BTreeSet::new()),
        }
    }

    pub fn add_renderer_group(&mut self, renderer_group: ArcRwLock<dyn RendererGroup>) -> bool {
        self.renderer_groups
            .write()
            .insert(SendablePtr::new(renderer_group.data_ptr()))
    }

    pub fn remove_renderer_group(&mut self, renderer_group: &ArcRwLock<dyn RendererGroup>) -> bool {
        self.renderer_groups
            .write()
            .remove(&SendablePtr::new(renderer_group.data_ptr()))
    }
}

#[derive(Clone)]
pub struct TestRendererGroupImpl {
    pub renderer_objects: ArcRwLock<BTreeSet<SendablePtr<dyn RendererObject>>>,
}

impl RendererGroup for TestRendererGroupImpl {}

impl TestRendererGroupImpl {
    pub fn new() -> Self {
        Self {
            renderer_objects: arc_rw_lock_new(BTreeSet::new()),
        }
    }

    pub fn add_renderer_object(&mut self, renderer_object: ArcRwLock<dyn RendererObject>) -> bool {
        self.renderer_objects
            .write()
            .insert(SendablePtr::new(renderer_object.data_ptr()))
    }

    pub fn remove_renderer_object(
        &mut self,
        renderer_object: &ArcRwLock<dyn RendererObject>,
    ) -> bool {
        self.renderer_objects
            .write()
            .remove(&SendablePtr::new(renderer_object.data_ptr()))
    }
}

pub struct TestRendererTransformImpl;
impl RendererTransform for TestRendererTransformImpl {}

pub struct TestRendererMaterialImpl;
impl RendererMaterial for TestRendererMaterialImpl {}

pub struct TestRendererShaderImpl;
impl RendererShader for TestRendererShaderImpl {}

pub struct TestRendererMeshImpl;
impl RendererMesh for TestRendererMeshImpl {}

pub struct TestRendererCameraImpl;
impl RendererCamera for TestRendererCameraImpl {}

pub struct TestRendererObjectImpl;
impl RendererObject for TestRendererObjectImpl {}

impl RendererImpl for TestRendererImpl {
    fn set_renderer_pipeline(
        &mut self,
        steps: Vec<renderer_pipeline_step_impl::RendererPipelineStepImpl>,
    ) -> Result<(), String> {
        self.renderer_steps = steps;
        Ok(())
    }

    fn create_renderer_layer(
        &mut self,
        camera: ArcRwLock<dyn RendererCamera>,
    ) -> Result<ArcRwLock<dyn RendererLayer>, String> {
        self.cameras
            .read()
            .get(&SendablePtr::new(camera.data_ptr()))
            .ok_or_else(|| "Creating renderer layer, msg = could not find camera".to_string())?;

        let renderer_layer = arc_rw_lock_new(TestRendererLayerImpl::new());
        self.renderer_layers.write().insert(
            SendablePtr::new(renderer_layer.data_ptr()),
            renderer_layer.read().clone(),
        );
        Ok(renderer_layer)
    }

    fn release_renderer_layer(
        &mut self,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        self.renderer_layers
            .write()
            .remove(&SendablePtr::new(renderer_layer.data_ptr()))
            .ok_or_else(|| "Releasing renderer layer, msg = could not find RendererLayer")?;
        Ok(())
    }

    fn add_renderer_group_to_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        self.renderer_groups
            .read()
            .get(&SendablePtr::new(renderer_group.data_ptr()))
            .ok_or_else(|| {
                "Adding renderer group to layer, msg = could not find renderer group".to_string()
            })?;

        self.renderer_layers
            .write()
            .get_mut(&SendablePtr::new(renderer_layer.data_ptr()))
            .ok_or_else(|| {
                "Adding renderer group to layer, msg = could not find renderer layer".to_string()
            })?
            .add_renderer_group(renderer_group);

        Ok(())
    }

    fn remove_renderer_group_from_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        self.renderer_groups
            .read()
            .get(&SendablePtr::new(renderer_group.data_ptr()))
            .ok_or_else(|| {
                "Removing renderer group from layer, msg = could not find renderer group"
                    .to_string()
            })?;

        self.renderer_layers
            .write()
            .get_mut(&SendablePtr::new(renderer_layer.data_ptr()))
            .ok_or_else(|| {
                "Removing renderer group from layer, msg = could not find renderer layer"
                    .to_string()
            })?
            .remove_renderer_group(&renderer_group);

        Ok(())
    }

    fn create_renderer_group(&mut self) -> Result<ArcRwLock<dyn RendererGroup>, String> {
        let renderer_group = arc_rw_lock_new(TestRendererGroupImpl::new());
        self.renderer_groups.write().insert(
            SendablePtr::new(renderer_group.data_ptr()),
            renderer_group.read().clone(),
        );
        Ok(renderer_group)
    }

    fn release_renderer_group(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String> {
        self.renderer_groups
            .write()
            .remove(&SendablePtr::new(renderer_group.data_ptr()))
            .ok_or_else(|| "Releasing renderer group, msg = could not find RendererGroup")?;
        Ok(())
    }

    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<ArcRwLock<dyn RendererTransform>, String> {
        let renderer_transform = arc_rw_lock_new(TestRendererTransformImpl);
        self.transforms
            .write()
            .insert(SendablePtr::new(renderer_transform.data_ptr()), transform);
        Ok(renderer_transform)
    }

    fn update_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), String> {
        self.transforms
            .write()
            .get_mut(&SendablePtr::new(transform.data_ptr()))
            .and_then(|transform| {
                *transform = new_transform;
                Some(())
            })
            .ok_or_else(|| "Updating transform, msg = could not find transform".to_string())
    }

    fn release_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<(), String> {
        self.transforms
            .write()
            .remove(&SendablePtr::new(transform.data_ptr()))
            .ok_or_else(|| "Releasing transform, msg = could not find RendererTransform")?;
        Ok(())
    }

    fn create_material(
        &mut self,
        material: crate::mesh::Material,
    ) -> Result<ArcRwLock<dyn RendererMaterial>, String> {
        let renderer_material = arc_rw_lock_new(TestRendererMaterialImpl);
        self.materials
            .write()
            .insert(SendablePtr::new(renderer_material.data_ptr()), material);
        Ok(renderer_material)
    }

    fn update_material(
        &mut self,
        material: ArcRwLock<dyn RendererMaterial>,
        new_material: Material,
    ) -> Result<(), String> {
        self.materials
            .write()
            .get_mut(&SendablePtr::new(material.data_ptr()))
            .and_then(|material| {
                *material = new_material;
                Some(())
            })
            .ok_or_else(|| "Updating material, msg = could not find material".to_string())
    }

    fn release_material(
        &mut self,
        material: ArcRwLock<dyn RendererMaterial>,
    ) -> Result<(), String> {
        self.materials
            .write()
            .remove(&SendablePtr::new(material.data_ptr()))
            .ok_or_else(|| "Releasing material, msg = could not find RendererMaterial")?;
        Ok(())
    }

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<ArcRwLock<dyn RendererShader>, String> {
        let renderer_shader = arc_rw_lock_new(TestRendererShaderImpl);
        self.shaders
            .write()
            .insert(SendablePtr::new(renderer_shader.data_ptr()), shader_name);
        Ok(renderer_shader)
    }

    fn update_shader(
        &mut self,
        shader: ArcRwLock<dyn RendererShader>,
        new_shader_name: String,
    ) -> Result<(), String> {
        self.shaders
            .write()
            .get_mut(&SendablePtr::new(shader.data_ptr()))
            .and_then(|shader| {
                *shader = new_shader_name;
                Some(())
            })
            .ok_or_else(|| "Updating shader, msg = could not find shader".to_string())
    }

    fn release_shader(&mut self, shader: ArcRwLock<dyn RendererShader>) -> Result<(), String> {
        self.shaders
            .write()
            .remove(&SendablePtr::new(shader.data_ptr()))
            .ok_or_else(|| "Releasing shader, msg = could not find RendererShader")?;
        Ok(())
    }

    fn create_mesh(
        &mut self,
        mesh: Arc<crate::mesh::Mesh>,
    ) -> Result<ArcRwLock<dyn RendererMesh>, String> {
        let renderer_mesh = arc_rw_lock_new(TestRendererMeshImpl);
        self.meshes
            .write()
            .insert(SendablePtr::new(renderer_mesh.data_ptr()), mesh);
        Ok(renderer_mesh)
    }

    fn update_mesh(
        &mut self,
        mesh: ArcRwLock<dyn RendererMesh>,
        new_mesh: Arc<Mesh>,
    ) -> Result<(), String> {
        self.meshes
            .write()
            .get_mut(&SendablePtr::new(mesh.data_ptr()))
            .and_then(|mesh| {
                *mesh = new_mesh;
                Some(())
            })
            .ok_or_else(|| "Updating mesh, msg = could not find mesh".to_string())
    }

    fn release_mesh(&mut self, mesh: ArcRwLock<dyn RendererMesh>) -> Result<(), String> {
        self.meshes
            .write()
            .remove(&SendablePtr::new(mesh.data_ptr()))
            .ok_or_else(|| "Releasing mesh, msg = could not find RendererMesh")?;
        Ok(())
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: ArcRwLock<dyn RendererMesh>,
        shader: ArcRwLock<dyn RendererShader>,
        material: ArcRwLock<dyn RendererMaterial>,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<ArcRwLock<dyn RendererObject>, String> {
        self.shaders
            .read()
            .get(&SendablePtr::new(shader.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find shader".to_string()
            })?;

        self.materials
            .read()
            .get(&SendablePtr::new(material.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find material".to_string()
            })?;

        self.meshes
            .read()
            .get(&SendablePtr::new(mesh.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find mesh".to_string()
            })?;

        self.transforms
            .read()
            .get(&SendablePtr::new(transform.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find transform".to_string()
            })?;

        let renderer_object = arc_rw_lock_new(TestRendererObjectImpl);
        self.renderer_objects
            .write()
            .insert(SendablePtr::new(renderer_object.data_ptr()));
        Ok(renderer_object)
    }

    fn release_renderer_object(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
    ) -> Result<(), String> {
        self.renderer_objects
            .write()
            .remove(&SendablePtr::new(renderer_object.data_ptr()))
            .then(|| ())
            .ok_or_else(|| "Releasing renderer object, msg = could not find RendererObject")?;

        for (_, renderer_group) in self.renderer_groups.write().iter_mut() {
            renderer_group.remove_renderer_object(&renderer_object);
        }

        Ok(())
    }

    fn add_renderer_object_to_group(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String> {
        self.renderer_objects
            .read()
            .contains(&SendablePtr::new(renderer_object.data_ptr()))
            .then(|| ())
            .ok_or_else(|| {
                "Adding renderer object to group, msg = could not find renderer object".to_string()
            })?;

        self.renderer_groups
            .write()
            .get_mut(&SendablePtr::new(renderer_group.data_ptr()))
            .ok_or_else(|| {
                "Adding renderer object to group, msg = could not find renderer group".to_string()
            })?
            .add_renderer_object(renderer_object);

        Ok(())
    }

    fn remove_renderer_object_from_group(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String> {
        self.renderer_groups
            .write()
            .get_mut(&SendablePtr::new(renderer_group.data_ptr()))
            .ok_or_else(|| {
                "Removing renderer object from group, msg = could not find renderer group"
                    .to_string()
            })?
            .remove_renderer_object(&renderer_object)
            .then(|| ())
            .ok_or_else(|| {
                "Removing renderer object from group, msg = could not find renderer object in group"
                    .to_string()
            })
    }

    fn create_camera(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<ArcRwLock<dyn RendererCamera>, String> {
        self.transforms
            .read()
            .get(&SendablePtr::new(transform.data_ptr()))
            .ok_or_else(|| "Creating camera, msg = could not find transform".to_string())?;

        let camera = arc_rw_lock_new(TestRendererCameraImpl);
        self.cameras
            .write()
            .insert(SendablePtr::new(camera.data_ptr()));
        Ok(camera)
    }

    fn release_camera(&mut self, camera: ArcRwLock<dyn RendererCamera>) -> Result<(), String> {
        self.cameras
            .write()
            .remove(&SendablePtr::new(camera.data_ptr()))
            .then_some(())
            .ok_or_else(|| "Releasing camera, msg = could not find RendererCamera")?;
        Ok(())
    }

    fn render(&mut self) {}
}

pub struct TestLoopSync {
    app_loop_state_watcher: AppLoopStateWatcher,
    renderer_system: SyncRenderer,
}

pub struct TestLoopAsync {
    app_loop_state_watcher: AppLoopStateWatcher,
    renderer_system: AsyncRenderer,
}

#[derive(Clone)]
pub struct TestLoopClient {
    app_loop_state: AppLoopState,
    renderer_impl: TestRendererImpl,
    renderer_client: RendererClient,
}

pub fn init_test_sync() -> (TestLoopSync, TestLoopClient) {
    let renderer_impl = TestRendererImpl::new();
    let app_loop_state = AppLoopState::new();
    let app_loop_state_watcher = app_loop_state.watcher();
    let renderer_system = SyncRenderer::new(renderer_impl.clone());
    let renderer_client = renderer_system.client();

    (
        TestLoopSync {
            app_loop_state_watcher,
            renderer_system,
        },
        TestLoopClient {
            app_loop_state,
            renderer_impl,
            renderer_client,
        },
    )
}

pub fn init_test_async() -> (TestLoopAsync, TestLoopClient) {
    let renderer_impl = TestRendererImpl::new();
    let app_loop_state = AppLoopState::new();
    let app_loop_state_watcher = app_loop_state.watcher();
    let renderer_system = AsyncRenderer::new(4, renderer_impl.clone()).unwrap();
    let renderer_client = renderer_system.client();

    (
        TestLoopAsync {
            app_loop_state_watcher,
            renderer_system,
        },
        TestLoopClient {
            app_loop_state,
            renderer_impl,
            renderer_client,
        },
    )
}

impl TestLoopSync {
    pub async fn block_on_main_loop(&mut self, timeout: Duration) {
        let start = Instant::now();
        while self.app_loop_state_watcher.should_run() {
            let loop_start = Instant::now();

            self.renderer_system.tick(&loop_start, 1.0 / 30.0);

            tokio::task::yield_now().await;

            let now = Instant::now();
            if now - start >= timeout {
                break;
            }
        }

        let loop_start = Instant::now();
        self.renderer_system.tick(&loop_start, 1.0 / 30.0);
    }

    pub fn renderer_system(&self) -> &SyncRenderer {
        &self.renderer_system
    }
}

impl TestLoopAsync {
    pub async fn block_on_main_loop(&mut self, timeout: Duration) {
        let start = Instant::now();
        while self.app_loop_state_watcher.should_run() {
            self.renderer_system.tick(1.0 / 30.0);

            tokio::task::yield_now().await;

            let now = Instant::now();
            if now - start >= timeout {
                break;
            }
        }

        self.renderer_system.tick(1.0 / 30.0);
    }

    pub fn renderer_system(&self) -> &AsyncRenderer {
        &self.renderer_system
    }
}

impl TestLoopClient {
    pub fn renderer_client(&self) -> &RendererClient {
        &self.renderer_client
    }

    pub fn stop_main_loop(&self) {
        self.app_loop_state.stop_loop();
    }

    pub fn renderer_impl(&self) -> &TestRendererImpl {
        &self.renderer_impl
    }
}
