mod test_renderer;

use std::sync::Arc;

use tokio::sync::RwLock as AsyncRwLock;
use vek::{Transform, Vec3};

use crate::{
    mesh::{Material, Mesh},
    renderer::tests::test_renderer::{init_test_async, init_test_sync},
    renderer::RendererGroupHandler,
};

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

    let renderer_group_handler: Arc<AsyncRwLock<Option<RendererGroupHandler>>> =
        Arc::new(AsyncRwLock::new(None));

    let test_task = {
        let test_client = test_client.clone();
        let renderer_group_handler = renderer_group_handler.clone();
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

            *renderer_group_handler.write().await = Some(
                test_client
                    .renderer_client()
                    .create_renderer_group()
                    .await
                    .unwrap(),
            );

            test_client
                .renderer_client()
                .add_renderer_object_to_group(
                    renderer_object_handler.clone(),
                    renderer_group_handler.read().await.clone().unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(1, test_client.renderer_impl().renderer_objects.read().len());

            let renderer_groups = test_client.renderer_impl().renderer_groups.read();
            assert_eq!(1, renderer_groups.len());
            let renderer_group = renderer_groups.iter().next().unwrap().1;
            assert_eq!(1, renderer_group.renderer_objects.read().len());

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

    let renderer_groups = test_client.renderer_impl().renderer_groups.read();
    assert_eq!(1, renderer_groups.len());
    let renderer_group = renderer_groups.iter().next().unwrap().1;
    assert_eq!(0, renderer_group.renderer_objects.read().len());
}
