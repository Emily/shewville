use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use shewville::{Animation, SIZE, Traits};

#[derive(Resource)]
struct Workbench {
    traits: Traits,
    animation: Animation,
    phase: f32,
    playing: bool,
    texture: Option<egui::TextureHandle>,
    message: String,
}

impl Default for Workbench {
    fn default() -> Self {
        Self {
            traits: Traits::default(),
            animation: Animation::Walk,
            phase: 0.,
            playing: true,
            texture: None,
            message: String::new(),
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--bake") {
        let path = args.get(2).map(String::as_str).unwrap_or("exports/default");
        if let Err(error) = shewville::bake(&Traits::default(), std::path::Path::new(path)) {
            eprintln!("Export failed: {error}");
            std::process::exit(1);
        }
        println!("Exported to {path}");
        return;
    }
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shewville · Creature workbench".into(),
                resolution: (1100, 760).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_resource::<Workbench>()
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .add_systems(EguiPrimaryContextPass, workbench)
        .run();
}

fn workbench(mut contexts: EguiContexts, time: Res<Time>, mut state: ResMut<Workbench>) -> Result {
    let ctx = contexts.ctx_mut()?;
    if state.playing {
        let duration = if state.animation == Animation::Idle {
            2.
        } else {
            1.
        };
        state.phase = (state.phase + time.delta_secs() / duration).rem_euclid(1.);
    }
    let mut root = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("workbench"),
        egui::UiBuilder::new()
            .max_rect(ctx.content_rect())
            .layer_id(egui::LayerId::background()),
    );
    egui::Panel::left("traits")
        .exact_size(290.)
        .show(&mut root, |ui| {
            ui.add_space(16.);
            ui.heading("SHEWVILLE");
            ui.label("Creature workbench / 001");
            ui.separator();
            ui.heading("Body");
            ui.add(egui::Slider::new(&mut state.traits.height, 85.0..=155.0).text("Height"));
            ui.add(egui::Slider::new(&mut state.traits.width, 60.0..=110.0).text("Build"));
            ui.checkbox(&mut state.traits.lobes, "Two lobes");
            ui.add(egui::Slider::new(&mut state.traits.waist, 0.0..=0.48).text("Waist pinch"));
            ui.add_space(12.);
            ui.heading("Face & ears");
            ui.add(
                egui::Slider::new(&mut state.traits.eye_spacing, 0.2..=0.65).text("Eye spacing"),
            );
            ui.add(egui::Slider::new(&mut state.traits.snout, 5.0..=22.0).text("Snout width"));
            ui.add(
                egui::Slider::new(&mut state.traits.snout_length, 4.0..=30.0).text("Snout length"),
            );
            ui.add(egui::Slider::new(&mut state.traits.ears, 30.0..=64.0).text("Ear length"))
                .on_hover_text("Larger ears are concentrated near the top of the range.");
            ui.add(
                egui::Slider::new(&mut state.traits.ear_angle, -30.0..=65.0).text("Ear angle °"),
            );
            ui.horizontal(|ui| {
                ui.label("Coat");
                ui.color_edit_button_srgb(&mut state.traits.color);
            });
            ui.add_space(12.);
            ui.horizontal(|ui| {
                if ui.button("Peanut").clicked() {
                    state.traits = Traits::default();
                }
                if ui.button("Round").clicked() {
                    state.traits = Traits {
                        height: 90.,
                        width: 105.,
                        lobes: false,
                        ..Default::default()
                    };
                }
                if ui.button("Tall").clicked() {
                    state.traits = Traits {
                        height: 155.,
                        width: 60.,
                        ears: 30.,
                        ..Default::default()
                    };
                }
            });
            ui.separator();
            ui.heading("Animation");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.animation, Animation::Idle, "Idle");
                ui.selectable_value(&mut state.animation, Animation::Walk, "Walk");
            });
            ui.checkbox(&mut state.playing, "Play");
            if ui
                .add(egui::Slider::new(&mut state.phase, 0.0..=1.0).text("Cycle"))
                .changed()
            {
                state.playing = false;
            }
            ui.separator();
            if ui.button("Export sprite sheet + JSON").clicked() {
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                let path = format!("exports/shewshew-{stamp}");
                state.message = match shewville::bake(&state.traits, std::path::Path::new(&path)) {
                    Ok(()) => format!("Saved {path}"),
                    Err(error) => format!("Export failed: {error}"),
                };
            }
            ui.label("16 frames per animation · 64×64 px\nIdle and walk on separate rows.");
            ui.label(&state.message);
        });
    let pixels = shewville::render(&state.traits, state.animation, state.phase);
    let image =
        egui::ColorImage::from_rgba_premultiplied([SIZE as usize, SIZE as usize], pixels.data());
    if let Some(texture) = &mut state.texture {
        // Keep the backing Bevy image alive across frames. Full updates replace
        // its asset handle in bevy_egui, racing the render-world texture upload.
        texture.set_partial([0, 0], image, egui::TextureOptions::NEAREST);
    } else {
        state.texture = Some(ctx.load_texture("shewshew", image, egui::TextureOptions::NEAREST));
    }
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(244, 239, 225))
                .inner_margin(24.),
        )
        .show(&mut root, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(65, 64, 51));
            ui.heading("Meet your Shewshew");
            ui.label("One creature, a family of shapes.");
            ui.add_space(8.);
            ui.horizontal(|ui| {
                if let Some(texture) = &state.texture {
                    // egui measures in points. Compensate for display scaling
                    // and UI zoom so this stays exactly 64 physical pixels.
                    let pixels_per_point = ui.ctx().pixels_per_point();
                    let size = egui::Vec2::splat(SIZE as f32 / pixels_per_point);
                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let min = egui::pos2(
                        (rect.min.x * pixels_per_point).round() / pixels_per_point,
                        (rect.min.y * pixels_per_point).round() / pixels_per_point,
                    );
                    ui.painter().image(
                        texture.id(),
                        egui::Rect::from_min_size(min, size),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                        egui::Color32::WHITE,
                    );
                }
                ui.vertical(|ui| {
                    ui.strong("Actual size");
                    ui.label("64×64 · 1 sprite pixel = 1 display pixel");
                });
            });
            ui.separator();
            let available = ui
                .available_width()
                .min(ui.available_height() - 35.)
                .clamp(64., 600.);
            let side = (available / SIZE as f32).floor() * SIZE as f32;
            ui.vertical_centered(|ui| {
                if let Some(texture) = &state.texture {
                    ui.image((texture.id(), egui::vec2(side, side)));
                }
                ui.label("64×64 sprite · enlarged with crisp pixels for inspection.");
            });
        });
    Ok(())
}
