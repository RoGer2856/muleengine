use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use parking_lot::RwLock;

use crate::{sendable_ptr::SendablePtr, system_container::System};

use super::{
    renderer_client::RendererClient, renderer_impl::RendererImpl, renderer_system::Renderer,
    RendererMaterial, RendererMesh, RendererObject, RendererShader,
};

#[derive(Clone)]
struct TestRendererImpl {
    materials: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererMaterial>>>>,
    shaders: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererShader>>>>,
    meshes: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererMesh>>>>,

    drawable_objects: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererObject>>>>,
    drawable_objects_to_draw: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererObject>>>>,
}

impl TestRendererImpl {
    pub fn new() -> Self {
        Self {
            materials: Arc::new(RwLock::new(BTreeSet::new())),
            shaders: Arc::new(RwLock::new(BTreeSet::new())),
            meshes: Arc::new(RwLock::new(BTreeSet::new())),
            drawable_objects: Arc::new(RwLock::new(BTreeSet::new())),
            drawable_objects_to_draw: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

struct TestRendererMaterialImpl;
impl RendererMaterial for TestRendererMaterialImpl {}

struct TestRendererShaderImpl;
impl RendererShader for TestRendererShaderImpl {}

struct TestRendererMeshImpl;
impl RendererMesh for TestRendererMeshImpl {}

struct TestMeshRendererObjectImpl;
impl RendererObject for TestMeshRendererObjectImpl {}

impl RendererImpl for TestRendererImpl {
    fn add_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn super::RendererObject>>,
        _transform: vek::Transform<f32, f32, f32>,
    ) -> Result<(), String> {
        let drawable_object = *self
            .drawable_objects
            .read()
            .get(&SendablePtr::new(drawable_object.data_ptr()))
            .ok_or_else(|| {
                "Adding drawable object, error = could not found drawable object".to_string()
            })?;
        self.drawable_objects_to_draw
            .write()
            .insert(drawable_object);

        Ok(())
    }

    fn create_drawable_mesh(
        &mut self,
        _mesh: Arc<crate::mesh::Mesh>,
    ) -> Result<Arc<RwLock<dyn super::RendererMesh>>, String> {
        let mesh = Arc::new(RwLock::new(TestRendererMeshImpl));
        self.meshes
            .write()
            .insert(SendablePtr::new(mesh.data_ptr()));
        Ok(mesh)
    }

    fn create_drawable_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn super::RendererMesh>>,
        shader: &Arc<RwLock<dyn super::RendererShader>>,
        material: &Arc<RwLock<dyn super::RendererMaterial>>,
    ) -> Result<Arc<RwLock<dyn super::RendererObject>>, String> {
        self.shaders
            .read()
            .get(&SendablePtr::new(shader.data_ptr()))
            .ok_or_else(|| {
                "Creating drawable object from mesh, error = could not found shader".to_string()
            })?;

        self.materials
            .read()
            .get(&SendablePtr::new(material.data_ptr()))
            .ok_or_else(|| {
                "Creating drawable object from mesh, error = could not found material".to_string()
            })?;

        self.meshes
            .read()
            .get(&SendablePtr::new(mesh.data_ptr()))
            .ok_or_else(|| {
                "Creating drawable object from mesh, error = could not found mesh".to_string()
            })?;

        let drawable_object = Arc::new(RwLock::new(TestMeshRendererObjectImpl));
        self.drawable_objects
            .write()
            .insert(SendablePtr::new(drawable_object.data_ptr()));
        Ok(drawable_object)
    }

    fn create_material(
        &mut self,
        _material: crate::mesh::Material,
    ) -> Result<Arc<RwLock<dyn super::RendererMaterial>>, String> {
        let material = Arc::new(RwLock::new(TestRendererMaterialImpl));
        self.materials
            .write()
            .insert(SendablePtr::new(material.data_ptr()));
        Ok(material)
    }

    fn create_shader(
        &mut self,
        _shader_name: String,
    ) -> Result<Arc<RwLock<dyn super::RendererShader>>, String> {
        let shader = Arc::new(RwLock::new(TestRendererShaderImpl));
        self.shaders
            .write()
            .insert(SendablePtr::new(shader.data_ptr()));
        Ok(shader)
    }

    fn remove_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn super::RendererObject>>,
    ) -> Result<(), String> {
        if self
            .drawable_objects_to_draw
            .write()
            .remove(&SendablePtr::new(drawable_object.data_ptr()))
        {
            Ok(())
        } else {
            Err(
                "Removing drawable object from renderer, error = could not found drawable object"
                    .to_string(),
            )
        }
    }

    fn render(&mut self) {}

    fn set_camera(&mut self, _camera: crate::camera::Camera) {}

    fn set_window_dimensions(&mut self, _dimensions: vek::Vec2<usize>) {}
}

struct TestLoop {
    should_run: Arc<AtomicBool>,
    renderer_system: Renderer,
}

#[derive(Clone)]
struct TestLoopClient {
    should_run: Arc<AtomicBool>,
    renderer_impl: TestRendererImpl,
    renderer_client: RendererClient,
}

fn init_test() -> (TestLoop, TestLoopClient) {
    let renderer_impl = TestRendererImpl::new();
    let should_run = Arc::new(AtomicBool::new(true));
    let renderer_system = Renderer::new(renderer_impl.clone());
    let renderer_client = renderer_system.client();

    (
        TestLoop {
            should_run: should_run.clone(),
            renderer_system,
        },
        TestLoopClient {
            should_run,
            renderer_impl,
            renderer_client,
        },
    )
}

impl TestLoop {
    pub async fn block_on_main_loop(&mut self) {
        while self.should_run.fetch_and(true, Ordering::SeqCst) {
            self.renderer_system.tick(1.0 / 30.0);

            tokio::task::yield_now().await;
        }

        self.renderer_system.tick(1.0 / 30.0);
    }
}

impl TestLoopClient {
    pub fn renderer_client(&self) -> &RendererClient {
        &self.renderer_client
    }

    pub fn stop_main_loop(&self) {
        self.should_run.fetch_and(false, Ordering::SeqCst);
    }

    pub fn renderer_impl(&self) -> &TestRendererImpl {
        &self.renderer_impl
    }
}

#[tokio::test(flavor = "current_thread")]
async fn shader_is_released_when_ids_are_dropped() {
    {
        let (mut test_loop, test_client) = init_test();

        let test_task = {
            let test_client = test_client.clone();
            tokio::spawn(async move {
                let _shader_id = test_client
                    .renderer_client()
                    .create_shader("some shader name".to_string())
                    .await
                    .unwrap();

                test_client.stop_main_loop();
            })
        };

        test_loop.block_on_main_loop().await;

        test_task.await.unwrap();

        assert_eq!(0, test_client.renderer_impl().shaders.read().len());
    }
}
