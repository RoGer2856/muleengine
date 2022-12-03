use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use parking_lot::RwLock;
use vek::{Transform, Vec3};

use crate::{
    mesh::{Material, Mesh},
    prelude::ArcRwLock,
    sendable_ptr::SendablePtr,
    system_container::System,
};

use super::{
    renderer_client::RendererClient,
    renderer_impl::RendererImpl,
    renderer_system::{AsyncRenderer, SyncRenderer},
    RendererMaterial, RendererMesh, RendererObject, RendererShader, RendererTransform,
};

#[derive(Clone)]
struct TestRendererImpl {
    transforms: ArcRwLock<BTreeMap<SendablePtr<dyn RendererTransform>, Transform<f32, f32, f32>>>,
    materials: ArcRwLock<BTreeMap<SendablePtr<dyn RendererMaterial>, Material>>,
    shaders: ArcRwLock<BTreeMap<SendablePtr<dyn RendererShader>, String>>,
    meshes: ArcRwLock<BTreeMap<SendablePtr<dyn RendererMesh>, Arc<Mesh>>>,

    renderer_objects: ArcRwLock<BTreeSet<SendablePtr<dyn RendererObject>>>,
    renderer_objects_to_draw: ArcRwLock<BTreeSet<SendablePtr<dyn RendererObject>>>,
}

impl TestRendererImpl {
    pub fn new() -> Self {
        Self {
            transforms: Arc::new(RwLock::new(BTreeMap::new())),
            materials: Arc::new(RwLock::new(BTreeMap::new())),
            shaders: Arc::new(RwLock::new(BTreeMap::new())),
            meshes: Arc::new(RwLock::new(BTreeMap::new())),
            renderer_objects: Arc::new(RwLock::new(BTreeSet::new())),
            renderer_objects_to_draw: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

struct TestRendererTransformImpl;
impl RendererTransform for TestRendererTransformImpl {}

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
        renderer_object: ArcRwLock<dyn super::RendererObject>,
    ) -> Result<(), String> {
        let renderer_object = *self
            .renderer_objects
            .read()
            .get(&SendablePtr::new(renderer_object.data_ptr()))
            .ok_or_else(|| {
                "Adding renderer object, error = could not find renderer object".to_string()
            })?;
        self.renderer_objects_to_draw
            .write()
            .insert(renderer_object);

        Ok(())
    }

    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<ArcRwLock<dyn RendererTransform>, String> {
        let renderer_transform = Arc::new(RwLock::new(TestRendererTransformImpl));
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
            .ok_or_else(|| "Updating transform, error = could not find transform".to_string())
    }

    fn release_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<(), String> {
        self.transforms
            .write()
            .remove(&SendablePtr::new(transform.data_ptr()));
        Ok(())
    }

    fn create_material(
        &mut self,
        material: crate::mesh::Material,
    ) -> Result<ArcRwLock<dyn super::RendererMaterial>, String> {
        let renderer_material = Arc::new(RwLock::new(TestRendererMaterialImpl));
        self.materials
            .write()
            .insert(SendablePtr::new(renderer_material.data_ptr()), material);
        Ok(renderer_material)
    }

    fn release_material(
        &mut self,
        material: ArcRwLock<dyn RendererMaterial>,
    ) -> Result<(), String> {
        self.materials
            .write()
            .remove(&SendablePtr::new(material.data_ptr()));
        Ok(())
    }

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<ArcRwLock<dyn super::RendererShader>, String> {
        let renderer_shader = Arc::new(RwLock::new(TestRendererShaderImpl));
        self.shaders
            .write()
            .insert(SendablePtr::new(renderer_shader.data_ptr()), shader_name);
        Ok(renderer_shader)
    }

    fn release_shader(&mut self, shader: ArcRwLock<dyn RendererShader>) -> Result<(), String> {
        self.shaders
            .write()
            .remove(&SendablePtr::new(shader.data_ptr()));
        Ok(())
    }

    fn create_mesh(
        &mut self,
        mesh: Arc<crate::mesh::Mesh>,
    ) -> Result<ArcRwLock<dyn super::RendererMesh>, String> {
        let renderer_mesh = Arc::new(RwLock::new(TestRendererMeshImpl));
        self.meshes
            .write()
            .insert(SendablePtr::new(renderer_mesh.data_ptr()), mesh);
        Ok(renderer_mesh)
    }

    fn release_mesh(&mut self, mesh: ArcRwLock<dyn RendererMesh>) -> Result<(), String> {
        self.meshes
            .write()
            .remove(&SendablePtr::new(mesh.data_ptr()));
        Ok(())
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: ArcRwLock<dyn super::RendererMesh>,
        shader: ArcRwLock<dyn super::RendererShader>,
        material: ArcRwLock<dyn super::RendererMaterial>,
        transform: ArcRwLock<dyn super::RendererTransform>,
    ) -> Result<ArcRwLock<dyn super::RendererObject>, String> {
        self.shaders
            .read()
            .get(&SendablePtr::new(shader.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, error = could not find shader".to_string()
            })?;

        self.materials
            .read()
            .get(&SendablePtr::new(material.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, error = could not find material".to_string()
            })?;

        self.meshes
            .read()
            .get(&SendablePtr::new(mesh.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, error = could not find mesh".to_string()
            })?;

        self.transforms
            .read()
            .get(&SendablePtr::new(transform.data_ptr()))
            .ok_or_else(|| {
                "Creating renderer object from mesh, error = could not find transform".to_string()
            })?;

        let renderer_object = Arc::new(RwLock::new(TestMeshRendererObjectImpl));
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
            .remove(&SendablePtr::new(renderer_object.data_ptr()));
        self.renderer_objects_to_draw
            .write()
            .remove(&SendablePtr::new(renderer_object.data_ptr()));
        Ok(())
    }

    fn remove_renderer_object(
        &mut self,
        renderer_object: ArcRwLock<dyn super::RendererObject>,
    ) -> Result<(), String> {
        if self
            .renderer_objects_to_draw
            .write()
            .remove(&SendablePtr::new(renderer_object.data_ptr()))
        {
            Ok(())
        } else {
            Err(
                "Removing renderer object from renderer, error = could not find renderer object"
                    .to_string(),
            )
        }
    }

    fn render(&mut self) {}

    fn set_camera(&mut self, _camera: crate::camera::Camera) {}

    fn set_window_dimensions(&mut self, _dimensions: vek::Vec2<usize>) {}
}

struct TestLoopSync {
    should_run: Arc<AtomicBool>,
    renderer_system: SyncRenderer,
}

struct TestLoopAsync {
    should_run: Arc<AtomicBool>,
    renderer_system: AsyncRenderer,
}

#[derive(Clone)]
struct TestLoopClient {
    should_run: Arc<AtomicBool>,
    renderer_impl: TestRendererImpl,
    renderer_client: RendererClient,
}

fn init_test_sync() -> (TestLoopSync, TestLoopClient) {
    let renderer_impl = TestRendererImpl::new();
    let should_run = Arc::new(AtomicBool::new(true));
    let renderer_system = SyncRenderer::new(renderer_impl.clone());
    let renderer_client = renderer_system.client();

    (
        TestLoopSync {
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

fn init_test_async() -> (TestLoopAsync, TestLoopClient) {
    let renderer_impl = TestRendererImpl::new();
    let should_run = Arc::new(AtomicBool::new(true));
    let renderer_system = AsyncRenderer::new(4, renderer_impl.clone());
    let renderer_client = renderer_system.client();

    (
        TestLoopAsync {
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

impl TestLoopSync {
    pub async fn block_on_main_loop(&mut self) {
        while self.should_run.fetch_and(true, Ordering::SeqCst) {
            self.renderer_system.tick(1.0 / 30.0);

            tokio::task::yield_now().await;
        }

        self.renderer_system.tick(1.0 / 30.0);
    }

    pub fn renderer_system(&self) -> &SyncRenderer {
        &self.renderer_system
    }
}

impl TestLoopAsync {
    pub async fn block_on_main_loop(&mut self) {
        while self.should_run.fetch_and(true, Ordering::SeqCst) {
            self.renderer_system.tick(1.0 / 30.0);

            tokio::task::yield_now().await;
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
        self.should_run.fetch_and(false, Ordering::SeqCst);
    }

    pub fn renderer_impl(&self) -> &TestRendererImpl {
        &self.renderer_impl
    }
}

#[tokio::test(flavor = "current_thread")]
async fn transform_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test_sync();

    let test_task = {
        let test_client = test_client.clone();
        tokio::spawn(async move {
            let _handler = test_client
                .renderer_client()
                .create_transform(Transform::default())
                .await
                .unwrap();

            assert_eq!(1, test_client.renderer_impl().transforms.read().len());

            test_client.stop_main_loop();
        })
    };

    test_loop.block_on_main_loop().await;

    test_task.await.unwrap();

    assert_eq!(
        0,
        test_loop
            .renderer_system()
            .renderer_pri
            .renderer_transforms
            .read()
            .len()
    );
    assert_eq!(0, test_client.renderer_impl().transforms.read().len());
}

#[tokio::test(flavor = "current_thread")]
async fn update_transform() {
    let (mut test_loop, test_client) = init_test_async();

    let test_task = {
        let test_client = test_client.clone();
        tokio::spawn(async move {
            let mut transform = Transform::default();

            let handler = test_client
                .renderer_client()
                .create_transform(transform)
                .await
                .unwrap();

            transform.position += Vec3::new(1.0, 2.0, 3.0);
            test_client
                .renderer_client()
                .update_transform(handler.clone(), transform)
                .await
                .unwrap();

            assert_eq!(
                transform,
                *test_client
                    .renderer_impl()
                    .transforms
                    .read()
                    .iter()
                    .next()
                    .unwrap()
                    .1
            );

            test_client.stop_main_loop();
        })
    };

    test_loop.block_on_main_loop().await;

    test_task.await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn shader_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test_sync();

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

    assert_eq!(
        0,
        test_loop
            .renderer_system()
            .renderer_pri
            .renderer_shaders
            .read()
            .len()
    );
    assert_eq!(0, test_client.renderer_impl().shaders.read().len());
}

#[tokio::test(flavor = "current_thread")]
async fn material_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test_async();

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

    assert_eq!(
        0,
        test_loop
            .renderer_system()
            .renderer_pri
            .renderer_materials
            .read()
            .len()
    );
    assert_eq!(0, test_client.renderer_impl().materials.read().len());
}

#[tokio::test(flavor = "current_thread")]
async fn mesh_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test_sync();

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

    assert_eq!(
        0,
        test_loop
            .renderer_system()
            .renderer_pri
            .renderer_meshes
            .read()
            .len()
    );
    assert_eq!(0, test_client.renderer_impl().meshes.read().len());
}

#[tokio::test(flavor = "current_thread")]
async fn renderer_object_is_released_when_handlers_are_dropped() {
    let (mut test_loop, test_client) = init_test_async();

    let test_task = {
        let test_client = test_client.clone();
        tokio::spawn(async move {
            let transform_handler = test_client
                .renderer_client()
                .create_transform(Transform::default())
                .await
                .unwrap();

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
                .create_renderer_object_from_mesh(
                    mesh_handler,
                    shader_handler,
                    material_handler,
                    transform_handler,
                )
                .await
                .unwrap();

            test_client
                .renderer_client()
                .add_renderer_object(renderer_object_handler.clone())
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

    assert_eq!(
        0,
        test_loop
            .renderer_system()
            .renderer_pri
            .renderer_objects
            .read()
            .len()
    );
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
