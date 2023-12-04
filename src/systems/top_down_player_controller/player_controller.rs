use std::{
    ops::Deref,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use method_taskifier::{method_taskifier_impl, task_channel::TaskReceiver};
use muleengine::{
    application_runner::ClosureTaskSender,
    camera::Camera,
    renderer::{renderer_system::RendererClient, RendererTransformHandler},
    system_container::System,
};
use vek::{Transform, Vec3};

use crate::{
    components::CurrentlyControlledCharacter, essential_services::EssentialServices,
    physics::character_controller::CharacterControllerHandler,
};

use super::input::{InputProvider, InputReceiver};

pub struct PlayerController {
    should_run: Arc<AtomicBool>,
    task_receiver: TaskReceiver<client::ChanneledTask>,
    input_receiver: InputReceiver,
    entity_container: EntityContainer,
    entity_group: EntityGroup,
    renderer_client: RendererClient,
    main_camera_transform_handler: RendererTransformHandler,
    skydome_camera_transform_handler: RendererTransformHandler,
}

#[method_taskifier_impl(module_name = client)]
impl PlayerController {
    pub async fn new(
        task_receiver: TaskReceiver<client::ChanneledTask>,
        input_receiver: InputReceiver,
        essentials: &Arc<EssentialServices>,
    ) -> Self {
        let entity_container = essentials.entity_container.clone();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            CurrentlyControlledCharacter,
            CharacterControllerHandler,
            Transform<f32, f32, f32>,
        ]);

        Self {
            should_run: Arc::new(AtomicBool::new(true)),
            task_receiver,
            input_receiver,
            entity_container,
            entity_group,
            renderer_client: essentials.renderer_client.clone(),
            main_camera_transform_handler: essentials
                .renderer_configuration
                .main_camera_transform_handler()
                .await,
            skydome_camera_transform_handler: essentials
                .renderer_configuration
                .skydome_camera_transform_handler()
                .await,
        }
    }

    #[method_taskifier_client_fn]
    pub fn start(&self) {
        drop(self.async_start());
    }

    #[method_taskifier_worker_fn]
    fn async_start(&self) {
        self.should_run.store(true, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn pause(&self) {
        drop(self.async_pause());
    }

    #[method_taskifier_worker_fn]
    fn async_pause(&self) {
        self.should_run.store(false, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn remove_later(&self, closure_task_sender: &ClosureTaskSender) {
        closure_task_sender.add_task(|app_context| {
            app_context.system_container_mut().remove::<InputProvider>();
            app_context
                .sendable_system_container()
                .write()
                .remove::<PlayerController>();
            app_context
                .service_container_ref()
                .remove::<client::Client>();
        });
    }
}

impl System for PlayerController {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        while let Ok(task) = self.task_receiver.try_pop() {
            self.execute_channeled_task(task);
        }

        // moving the camera
        let movement_direction = self
            .input_receiver
            .movement_event_receiver
            .get_normalized_aggregated_moving_direction();

        if self.should_run.load(atomic::Ordering::SeqCst) {
            let mut entity_container = self.entity_container.lock();
            for entity_id in self.entity_group.iter_entity_ids() {
                if let Some(mut entity_handler) = entity_container.handler_for_entity(&entity_id) {
                    let character_specs = if let Some(character_specs) =
                        entity_handler.get_component_ref::<CurrentlyControlledCharacter>()
                    {
                        character_specs.deref().clone()
                    } else {
                        continue;
                    };

                    entity_handler.change_component(
                        |character_controller: &mut CharacterControllerHandler| {
                            character_controller
                                .set_velocity(movement_direction * character_specs.max_velocity);
                        },
                    );

                    let character_position = entity_handler
                        .get_component_ref::<Transform<f32, f32, f32>>()
                        .as_deref()
                        .map(|transform| transform.position);

                    if let Some(character_position) = character_position {
                        let looking_direction = *self.input_receiver.looking_direction.read();

                        // let angle_rad = looking_direction.angle_between(Vec2::unit_x());
                        // let character_rotation = Quaternion::from_scalar_and_vec3((angle_rad, Vec3::unit_y()));

                        let mut camera = Camera::new();
                        camera.pitch(-90.0f32.to_radians());

                        drop(self.renderer_client.update_transform(
                            self.skydome_camera_transform_handler.clone(),
                            *camera.transform_ref(),
                        ));

                        camera.move_by(
                            character_position
                                + Vec3::unit_y() * character_specs.camera_distance
                                + Vec3::new(looking_direction.x, 0.0, looking_direction.y)
                                    * character_specs.camera_distance
                                    * 0.2,
                        );
                        drop(self.renderer_client.update_transform(
                            self.main_camera_transform_handler.clone(),
                            *camera.transform_ref(),
                        ));
                    } else {
                        continue;
                    }

                    break;
                }
            }
        }
    }
}
