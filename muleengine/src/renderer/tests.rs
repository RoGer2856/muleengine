use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use parking_lot::RwLock;
use vek::Transform;

use crate::{
    mesh::{Material, Mesh},
    sendable_ptr::SendablePtr,
    system_container::System,
};

use super::{
    renderer_client::RendererClient, renderer_impl::RendererImpl, renderer_system::Renderer,
    RendererMaterial, RendererMesh, RendererObject, RendererShader,
};

#[derive(Clone)]
struct TestRendererImpl {
    materials: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererMaterial>>>>,
    shaders: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererShader>>>>,
    meshes: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererMesh>>>>,

    renderer_objects: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererObject>>>>,
    renderer_objects_to_draw: Arc<RwLock<BTreeSet<SendablePtr<dyn RendererObject>>>>,
}

impl TestRendererImpl {
    pub fn new() -> Self {
        Self {
            materials: Arc::new(RwLock::new(BTreeSet::new())),
            shaders: Arc::new(RwLock::new(BTreeSet::new())),
            meshes: Arc::new(RwLock::new(BTreeSet::new())),
            renderer_objects: Arc::new(RwLock::new(BTreeSet::new())),
            renderer_objects_to_draw: Arc::new(RwLock::new(BTreeSet::new())),
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
    fn add_renderer_object(
        &mut self,
        renderer_object: &Arc<RwLock<dyn super::RendererObject>>,
        _transform: vek::Transform<f32, f32, f32>,
    ) -> Result<(), String> {
        let renderer_object = *self
            .renderer_objects
            .read()
            .get(&SendablePtr::new(renderer_object.data_ptr()))
            .ok_or_else(|| {
                "Adding renderer object, error = could not found renderer object".to_string()
            })?;
        self.renderer_objects_to_draw
            .write()
            .insert(renderer_object);

        Ok(())
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

    fn release_material(
        &mut self,
        material: Arc<RwLock<dyn RendererMaterial>>,
    ) -> Result<(), String> {
        self.materials
            .write()
            .remove(&SendablePtr::new(material.data_ptr()));
        Ok(())
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

    fn release_shader(&mut self, shader: Arc<RwLock<dyn RendererShader>>) -> Result<(), String> {
        self.shaders
            .write()
            .remove(&SendablePtr::new(shader.data_ptr()));
        Ok(())
    }

    fn create_mesh(
        &mut self,
        _mesh: Arc<crate::mesh::Mesh>,
    ) -> Result<Arc<RwLock<dyn super::RendererMesh>>, String> {
        let mesh = Arc::new(RwLock::new(TestRendererMeshImpl));
        self.meshes
            .write()
            .insert(SendablePtr::new(mesh.data_ptr()));
        Ok(mesh)
    }

    fn release_mesh(&mut self, mesh: Arc<RwLock<dyn RendererMesh>>) -> Result<(), String> {
        self.meshes
            .write()
            .remove(&SendablePtr::new(mesh.data_ptr()));
        Ok(())
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn super::RendererMesh>>,
        shader: &Arc<RwLock<dyn super::RendererShader>>,
        material: &Arc<RwLock<dyn super::RendererMaterial>>,
    ) -> Result<Arc<RwLock<dyn super::RendererObject>>, String> {
        self.shaders
            .read()
            .get(&SendablePtr::new(shader.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, error = could not found shader".to_string()
            })?;

        self.materials
            .read()
            .get(&SendablePtr::new(material.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, error = could not found material".to_string()
            })?;

        self.meshes
            .read()
            .get(&SendablePtr::new(mesh.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, error = could not found mesh".to_string()
            })?;

        let renderer_object = Arc::new(RwLock::new(TestMeshRendererObjectImpl));
        self.renderer_objects
            .write()
            .insert(SendablePtr::new(renderer_object.data_ptr()));
        Ok(renderer_object)
    }

    fn release_renderer_object(
        &mut self,
        renderer_object: Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String> {
        self.renderer_objects
            .write()
            .remove(&SendablePtr::new(renderer_object.data_ptr()));
        self.renderer_objects_to_draw
            .write()
            .remove(&SendablePtr::new(renderer_object.data_ptr()));
        Ok(())
    }

    fn remove_renderer_object(
        &mut self,
        renderer_object: &Arc<RwLock<dyn super::RendererObject>>,
    ) -> Result<(), String> {
        if self
            .renderer_objects_to_draw
            .write()
            .remove(&SendablePtr::new(renderer_object.data_ptr()))
        {
            Ok(())
        } else {
            Err(
                "Removing renderer object from renderer, error = could not found renderer object"
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

    pub fn renderer_system(&self) -> &Renderer {
        &self.renderer_system
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
async fn shader_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test();

    let test_task = {
        let test_client = test_client.clone();
        tokio::spawn(async move {
            let _handler = test_client
                .renderer_client()
                .create_shader("some shader name".to_string())
                .await
                .unwrap();

            assert_eq!(1, test_client.renderer_impl().shaders.read().len());

            test_client.stop_main_loop();
        })
    };

    test_loop.block_on_main_loop().await;

    test_task.await.unwrap();

    assert_eq!(0, test_loop.renderer_system().renderer_shaders.len());
    assert_eq!(0, test_client.renderer_impl().shaders.read().len());
}

#[tokio::test(flavor = "current_thread")]
async fn material_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test();

    let test_task = {
        let test_client = test_client.clone();
        tokio::spawn(async move {
            let _handler = test_client
                .renderer_client()
                .create_material(Material::default())
                .await
                .unwrap();

            assert_eq!(1, test_client.renderer_impl().materials.read().len());

            test_client.stop_main_loop();
        })
    };

    test_loop.block_on_main_loop().await;

    test_task.await.unwrap();

    assert_eq!(0, test_loop.renderer_system().renderer_materials.len());
    assert_eq!(0, test_client.renderer_impl().materials.read().len());
}

#[tokio::test(flavor = "current_thread")]
async fn mesh_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test();

    let test_task = {
        let test_client = test_client.clone();
        tokio::spawn(async move {
            let _handler = test_client
                .renderer_client()
                .create_mesh(Arc::new(Mesh::default()))
                .await
                .unwrap();

            assert_eq!(1, test_client.renderer_impl().meshes.read().len());

            test_client.stop_main_loop();
        })
    };

    test_loop.block_on_main_loop().await;

    test_task.await.unwrap();

    assert_eq!(0, test_loop.renderer_system().renderer_meshes.len());
    assert_eq!(0, test_client.renderer_impl().meshes.read().len());
}

#[tokio::test(flavor = "current_thread")]
async fn renderer_object_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test();

    let test_task = {
        let test_client = test_client.clone();
        tokio::spawn(async move {
            let material_handler = test_client
                .renderer_client()
                .create_material(Material::default())
                .await
                .unwrap();

            let shader_handler = test_client
                .renderer_client()
                .create_shader("some shader name".to_string())
                .await
                .unwrap();

            let mesh_handler = test_client
                .renderer_client()
                .create_mesh(Arc::new(Mesh::default()))
                .await
                .unwrap();

            let renderer_object_handler = test_client
                .renderer_client()
                .create_renderer_object_from_mesh(mesh_handler, shader_handler, material_handler)
                .await
                .unwrap();

            test_client
                .renderer_client()
                .add_renderer_object(renderer_object_handler.clone(), Transform::default())
                .await
                .unwrap();

            assert_eq!(1, test_client.renderer_impl().renderer_objects.read().len());
            assert_eq!(
                1,
                test_client
                    .renderer_impl()
                    .renderer_objects_to_draw
                    .read()
                    .len()
            );

            test_client.stop_main_loop();
        })
    };

    test_loop.block_on_main_loop().await;

    test_task.await.unwrap();

    assert_eq!(0, test_loop.renderer_system().renderer_objects.len());
    assert_eq!(0, test_client.renderer_impl().renderer_objects.read().len());
    assert_eq!(
        0,
        test_client
            .renderer_impl()
            .renderer_objects_to_draw
            .read()
            .len()
    );
}
