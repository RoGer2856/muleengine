use std::sync::Arc;

use entity_component::EntityId;
use muleengine::{
    bytifex_utils::sync::{broadcast::Receiver, types::ArcRwLock},
    font::{HackFontContainer, RenderedGlyph},
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
    mesh_creator,
    renderer::RendererGroupHandler,
    window_context::{Event, Key, WindowContext},
};
use vek::{Transform, Vec2, Vec3};

use crate::{
    essential_services::EssentialServices,
    game_objects::tools::game_object_builder::GameObjectBuilder,
};

const PIXEL_SCALE: usize = 128;
const PIXEL_SCALE_F32: f32 = PIXEL_SCALE as f32;
const TEXT_SCALE: f32 = 0.1;
const START_POSITION: Vec2<f32> = Vec2::new(-1.0 + TEXT_SCALE / 2.0, 0.0);

struct PrintedCharacter {
    entity_id: Option<EntityId>,
    advance: f32,
}

struct Terminal {
    essentials: Arc<EssentialServices>,
    event_receiver: Receiver<Event>,
    printed_characters: Vec<PrintedCharacter>,
    next_character_position: Vec3<f32>,
    renderer_group_handler: RendererGroupHandler,
    next_command_text: String,
    terminal_text: String,
}

impl Terminal {
    async fn run(&mut self) {
        let mut is_terminal_opened = false;
        self.add_character('>', false).await;

        while let Ok(event) = self.event_receiver.pop().await {
            if let Event::KeyDown { key } = event {
                if is_terminal_opened {
                    if key == Key::Return {
                        if !self.next_command_text.is_empty() {
                            self.execute_command(&self.next_command_text, &self.essentials)
                                .await;
                        }
                        self.next_command_text = String::new();
                        self.next_character_position = Vec3::new(
                            START_POSITION.x,
                            self.next_character_position.y - TEXT_SCALE,
                            self.next_character_position.z,
                        );
                        self.add_character('>', false).await;
                    } else if key == Key::Backspace {
                        if let Some(_chr) = self.next_command_text.pop() {
                            if let Some(printed_character) = self.printed_characters.pop() {
                                if let Some(entity_id) = printed_character.entity_id {
                                    self.essentials
                                        .entity_container
                                        .lock()
                                        .remove_entity(&entity_id);
                                }
                                self.next_character_position.x -= printed_character.advance;
                            }
                            self.terminal_text.pop();
                        }
                    }
                }
            } else if let Event::Text { text } = event {
                if text == "~" {
                    is_terminal_opened = !is_terminal_opened;
                    log::info!("terminal opened = {is_terminal_opened}");
                } else if is_terminal_opened {
                    for chr in text.chars() {
                        self.add_character(chr, true).await;
                    }
                }
            }
        }
    }

    fn create_material_for_char(
        &self,
        chr: char,
        font: &mut HackFontContainer,
        pixel_scale: usize,
    ) -> Option<(Material, RenderedGlyph)> {
        let glyph = font.get_rendered_glyph(chr, pixel_scale)?;
        Some((
            Material {
                textures: vec![MaterialTexture {
                    image: glyph.image().clone(),
                    texture_type: MaterialTextureType::Albedo,
                    texture_map_mode: TextureMapMode::Clamp,
                    blend: 1.0,
                    uv_channel_id: 0,
                }],
                opacity: 1.0,
                albedo_color: Vec3::broadcast(1.0),
                shininess_color: Vec3::broadcast(0.0),
                emissive_color: Vec3::broadcast(0.0),
            },
            glyph,
        ))
    }

    async fn execute_command(&self, command: &str, _essentials: &Arc<EssentialServices>) {
        log::info!("executing command = {command}");
    }

    async fn add_character(&mut self, chr: char, is_command_character: bool) {
        self.terminal_text.push(chr);
        if is_command_character {
            self.next_command_text.push(chr);
        }

        let material_and_glyph =
            self.create_material_for_char(chr, &mut self.essentials.hack_font.write(), PIXEL_SCALE);

        let (advance, entity_id) = if let Some((material, glyph)) = material_and_glyph {
            let entity_builder = GameObjectBuilder::new(&self.essentials)
                .mesh(Arc::new(mesh_creator::rectangle2d::create(1.0, 1.0)))
                .await
                .shader("Assets/shaders/unlit")
                .await
                .transform(Transform {
                    position: self.next_character_position
                        + glyph.compute_render_offset_px() / PIXEL_SCALE_F32
                            * TEXT_SCALE
                            * Vec3::new(1.0, -1.0, 1.0),
                    scale: Vec3::new(TEXT_SCALE, -TEXT_SCALE, 1.0),
                    ..Default::default()
                })
                .await
                .material(material)
                .await
                .renderer_group_handler(self.renderer_group_handler.clone())
                .build()
                .await;

            let entity_id = entity_builder.build();

            (
                glyph.h_advance() / PIXEL_SCALE_F32 * TEXT_SCALE,
                Some(entity_id),
            )
        } else {
            (TEXT_SCALE, None)
        };

        self.printed_characters
            .push(PrintedCharacter { entity_id, advance });
        self.next_character_position.x += advance;
    }
}

pub fn run(essentials: &Arc<EssentialServices>, window_context: ArcRwLock<dyn WindowContext>) {
    let event_receiver = window_context.read().event_receiver();
    let essentials = essentials.clone();

    tokio::spawn(async move {
        let mut terminal = Terminal {
            essentials: essentials.clone(),
            event_receiver,
            printed_characters: Vec::new(),
            next_character_position: Vec3::from(START_POSITION),
            renderer_group_handler: essentials
                .renderer_configuration
                .ortho_overlay_renderer_group_handler()
                .await,
            terminal_text: String::new(),
            next_command_text: String::new(),
        };

        terminal.run().await;
    });
}
