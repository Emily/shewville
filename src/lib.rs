//! Creature geometry and baking. No Bevy types or running engine required.
use serde::{Deserialize, Serialize};
use std::path::Path;
use tiny_skia::{Pixmap, Transform};

mod pixel;

pub const SIZE: u32 = 64;
pub const FRAMES: u32 = 16;
// Keep proportions in design coordinates and rasterize directly at tile size.
// One shared scale preserves height differences and avoids per-frame resizing.
const DESIGN_SIZE: f32 = 256.;
const TILE_SCALE: f32 = SIZE as f32 / DESIGN_SIZE;
pub const ANCHOR: [f32; 2] = [128. * TILE_SCALE, 234. * TILE_SCALE];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Traits {
    pub height: f32,
    pub width: f32,
    pub lobes: bool,
    pub waist: f32,
    pub eye_spacing: f32,
    pub snout: f32,
    pub snout_length: f32,
    pub ears: f32,
    pub ear_angle: f32,
    pub color: [u8; 3],
}

impl Default for Traits {
    fn default() -> Self {
        Self {
            height: 130.,
            width: 80.,
            lobes: true,
            waist: 0.25,
            eye_spacing: 0.42,
            snout: 12.,
            snout_length: 12.,
            ears: 30.,
            ear_angle: 10.,
            color: [215, 185, 117],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub enum Animation {
    Idle,
    Walk,
}

/// A simple, repeatable inheritance model for phenotype traits, not an allele
/// simulation. Numeric traits blend independently; lobe count comes from one
/// parent. No mutation or age scaling is applied. Equal traits stay equal.
pub fn offspring(a: &Traits, b: &Traits, seed: u64) -> Traits {
    let mut state = seed;
    let mut next = || {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        (z >> 40) as f32 / 16777216.
    };
    let lobes = if next() < 0.5 { a.lobes } else { b.lobes };
    let color_mix = 0.25 + next() * 0.5;
    let mut blend = |x: f32, y: f32| x + (y - x) * (0.25 + next() * 0.5);
    Traits {
        height: blend(a.height, b.height),
        width: blend(a.width, b.width),
        lobes,
        waist: blend(a.waist, b.waist),
        eye_spacing: blend(a.eye_spacing, b.eye_spacing),
        snout: blend(a.snout, b.snout),
        snout_length: blend(a.snout_length, b.snout_length),
        ears: blend(a.ears, b.ears),
        ear_angle: blend(a.ear_angle, b.ear_angle),
        color: std::array::from_fn(|i| {
            (a.color[i] as f32 + (b.color[i] as f32 - a.color[i] as f32) * color_mix).round() as u8
        }),
    }
}

/// Phase is measured in cycles; phase 0 and phase 1 are the same pose.
pub fn render(traits: &Traits, animation: Animation, phase: f32) -> Pixmap {
    pixel::render(traits, animation, phase)
}

/// Creates a new directory: existing exports are never overwritten.
pub fn bake(traits: &Traits, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(directory.parent().unwrap_or(Path::new(".")))?;
    std::fs::create_dir(directory)?;
    let mut sheet = Pixmap::new(SIZE * FRAMES, SIZE * 2).unwrap();
    for (row, animation) in [Animation::Idle, Animation::Walk].into_iter().enumerate() {
        for frame in 0..FRAMES {
            let image = render(traits, animation, frame as f32 / FRAMES as f32);
            sheet.draw_pixmap(
                (frame * SIZE) as i32,
                row as i32 * SIZE as i32,
                image.as_ref(),
                &Default::default(),
                Transform::identity(),
                None,
            );
        }
    }
    sheet.save_png(directory.join("sprites.png"))?;
    let metadata = serde_json::json!({
        "schema_version": 1, "traits": traits, "image": "sprites.png",
        "frame_size": [SIZE, SIZE], "anchor_pixels": ANCHOR,
        "animations": [
            {"name": "idle", "row": 0, "frames": FRAMES, "fps": 8, "loop": true},
            {"name": "walk", "row": 1, "frames": FRAMES, "fps": 16, "loop": true}
        ]
    });
    std::fs::write(
        directory.join("sprites.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offspring_is_repeatable_and_inherits_parent_traits() {
        let a = Traits::default();
        let b = Traits {
            height: 90.,
            width: 105.,
            ears: 64.,
            lobes: false,
            color: [199, 178, 123],
            ..a.clone()
        };
        let first = serde_json::to_value(offspring(&a, &b, 42)).unwrap();
        assert_eq!(first, serde_json::to_value(offspring(&a, &b, 42)).unwrap());
        assert_eq!(
            serde_json::to_value(offspring(&a, &a, 42)).unwrap(),
            serde_json::to_value(&a).unwrap()
        );
        let mut lobe_counts = std::collections::HashSet::new();
        for seed in 0..32 {
            let child = offspring(&a, &b, seed);
            lobe_counts.insert(child.lobes);
            assert!((90. ..=130.).contains(&child.height));
            assert!((80. ..=105.).contains(&child.width));
            assert!((30. ..=64.).contains(&child.ears));
            assert_eq!(child.snout, a.snout);
            for i in 0..3 {
                assert!(
                    (b.color[i].min(a.color[i])..=b.color[i].max(a.color[i]))
                        .contains(&child.color[i])
                );
            }
        }
        assert_eq!(lobe_counts.len(), 2);
    }
    #[test]
    fn pixel_art_has_a_small_palette_and_body_local_texture() {
        let traits = Traits::default();
        let rest = render(&traits, Animation::Idle, 0.);
        let raised = render(&traits, Animation::Idle, 0.25);
        let mut colors = std::collections::HashSet::new();
        for pixel in rest.data().chunks_exact(4) {
            assert!(pixel[3] == 0 || pixel[3] == 255, "no antialiased pixels");
            if pixel[3] != 0 {
                colors.insert(pixel.to_vec());
            }
        }
        assert!(colors.len() <= 9, "palette grew to {} colors", colors.len());
        // Idle translates the body by one logical (two physical) pixel. Face, outline, and shading
        // must all translate identically; exclude the lower-body deformation
        // that keeps the planted feet attached.
        for y in 0..44 {
            for x in 0..SIZE {
                assert_eq!(rest.pixel(x, y), raised.pixel(x, y + 2));
            }
        }
    }
    #[test]
    fn exported_sheet_matches_preview_and_protects_existing_exports() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("shewville-test-{}-{stamp}", std::process::id()));
        let traits = Traits::default();
        bake(&traits, &directory).unwrap();
        let sheet = Pixmap::load_png(directory.join("sprites.png")).unwrap();
        assert_eq!((sheet.width(), sheet.height()), (1024, 128));
        let preview = render(&traits, Animation::Walk, 0.25);
        for y in 0..SIZE {
            for x in 0..SIZE {
                assert_eq!(sheet.pixel(4 * SIZE + x, SIZE + y), preview.pixel(x, y));
            }
        }
        let metadata: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("sprites.json")).unwrap())
                .unwrap();
        assert_eq!(metadata["frame_size"], serde_json::json!([64, 64]));
        assert_eq!(metadata["anchor_pixels"], serde_json::json!([32.0, 58.5]));
        assert!(bake(&traits, &directory).is_err());
        std::fs::remove_file(directory.join("sprites.png")).unwrap();
        std::fs::remove_file(directory.join("sprites.json")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
    #[test]
    fn loops_and_traits_are_repeatable() {
        let traits = Traits::default();
        for animation in [Animation::Idle, Animation::Walk] {
            assert_eq!(
                render(&traits, animation, 0.).data(),
                render(&traits, animation, 1.).data()
            );
        }
        let mut round = traits.clone();
        round.lobes = false;
        assert_ne!(
            render(&traits, Animation::Idle, 0.).data(),
            render(&round, Animation::Idle, 0.).data()
        );
    }
    #[test]
    fn walking_alternates_the_feet_even_when_body_bob_is_identical() {
        for height in [85., 130., 155.] {
            for width in [50., 80., 110.] {
                for lobes in [false, true] {
                    let traits = Traits {
                        height,
                        width,
                        lobes,
                        ..Default::default()
                    };
                    let left_step = render(&traits, Animation::Walk, 0.25);
                    let right_step = render(&traits, Animation::Walk, 0.75);
                    // Opposite steps have the same body height. Compare only
                    // the bottom rows so hands/tail cannot make this test pass.
                    let feet_start = (58 * SIZE * 4) as usize;
                    assert_ne!(
                        &left_step.data()[feet_start..],
                        &right_step.data()[feet_start..],
                        "feet did not move for {traits:?}"
                    );
                    assert_feet_connected(&left_step);
                    assert_feet_connected(&right_step);
                    // The swing sole must actually clear the ground row,
                    // while the opposite foot remains planted.
                    for (image, lifted, planted) in [(&left_step, 38, 26), (&right_step, 26, 38)] {
                        assert_eq!(image.pixel(lifted, 58).unwrap().alpha(), 0);
                        assert_eq!(image.pixel(planted, 58).unwrap().alpha(), 255);
                        let cap = image.pixel(lifted, 54).unwrap();
                        assert!(
                            cap.alpha() != 0 && [cap.red(), cap.green(), cap.blue()] != [8, 7, 5]
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn eyes_are_two_cells_wide_and_separate_from_outline() {
        for height in [85., 130., 155.] {
            for width in [60., 80., 110.] {
                for lobes in [false, true] {
                    for eye_spacing in [0.2, 0.65] {
                        let traits = Traits {
                            height,
                            width,
                            lobes,
                            eye_spacing,
                            ..Default::default()
                        };
                        for frame in 0..FRAMES {
                            let image =
                                render(&traits, Animation::Walk, frame as f32 / FRAMES as f32);
                            let ink = |x, y| {
                                let p = image.pixel(x, y).unwrap();
                                [p.red(), p.green(), p.blue(), p.alpha()] == [8, 7, 5, 255]
                            };
                            let mut eyes = Vec::new();
                            let coat =
                                |x, y| image.pixel(x, y).unwrap().alpha() == 255 && !ink(x, y);
                            let h = (34. * height / 130. / 2.).round() * 2.;
                            let bob = ((frame as f32 / FRAMES as f32 * std::f32::consts::TAU * 2.)
                                .cos())
                            .round()
                                * 2.;
                            let eye_y = (54. - h + bob + (h * 0.26 / 2.).round() * 2.) as u32;
                            // Keep the eye band above the nose and jaw rim.
                            for y in ((eye_y - 4)..=(eye_y + 2)).step_by(2) {
                                for x in (16..46).step_by(2) {
                                    if ink(x, y)
                                        && ink(x + 2, y)
                                        && coat(x - 2, y)
                                        && coat(x + 4, y)
                                        && [x, x + 2]
                                            .into_iter()
                                            .all(|xx| coat(xx, y - 2) && coat(xx, y + 2))
                                    {
                                        eyes.push((x, y));
                                    }
                                }
                            }
                            assert_eq!(
                                eyes.len(),
                                2,
                                "missing or merged eyes: {traits:?}, frame {frame}, cells {eyes:?}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn snout_length_moves_the_nose_across_builds() {
        for (height, width, lobes) in [(130., 80., true), (90., 105., false), (155., 60., true)] {
            let mut nose_positions = Vec::new();
            for snout_length in [4., 12., 30.] {
                let traits = Traits {
                    height,
                    width,
                    lobes,
                    snout_length,
                    ..Default::default()
                };
                let image = render(&traits, Animation::Idle, 0.);
                // The right edge of the gray transition tracks the back of
                // the nose even when the cap's left edge meets the outline.
                let x = image
                    .pixels()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| {
                        ([p.red(), p.green(), p.blue(), p.alpha()] == [225, 227, 226, 255])
                            .then_some(i as u32 % SIZE)
                    })
                    .max()
                    .unwrap_or_else(|| panic!("nose needs a visible gray transition: {traits:?}"));
                nose_positions.push(x);
                for i in 0..SIZE {
                    assert_eq!(image.pixel(0, i).unwrap().alpha(), 0);
                    assert_eq!(image.pixel(SIZE - 1, i).unwrap().alpha(), 0);
                }
            }
            assert!(
                nose_positions[0] > nose_positions[1],
                "short/default: {nose_positions:?}"
            );
            assert!(
                nose_positions[1] >= nose_positions[2] + 4,
                "default/long: {nose_positions:?}"
            );
        }
    }
    #[test]
    fn extreme_builds_stay_inside_frame() {
        for height in [85., 130., 155.] {
            for width in [50., 80., 110.] {
                for lobes in [false, true] {
                    let traits = Traits {
                        height,
                        width,
                        lobes,
                        ears: 32.,
                        ..Default::default()
                    };
                    for (animation, frame) in [Animation::Idle, Animation::Walk]
                        .into_iter()
                        .flat_map(|animation| (0..FRAMES).map(move |frame| (animation, frame)))
                    {
                        let image = render(&traits, animation, frame as f32 / FRAMES as f32);
                        assert_eq!((image.width(), image.height()), (64, 64));
                        assert!(image.pixels().iter().any(|pixel| pixel.alpha() > 0));
                        assert_feet_connected(&image);
                        // Fixed stance: both colored centers remain twelve
                        // output pixels apart regardless of the body width.
                        for (x, side) in [(26, -1.), (38, 1.)] {
                            let phase = frame as f32 / FRAMES as f32 * std::f32::consts::TAU;
                            let lift = if animation == Animation::Walk && phase.sin() * side > 0.25
                            {
                                2
                            } else {
                                0
                            };
                            let center = image.pixel(x, 56 - lift).unwrap();
                            assert!(center.alpha() != 0);
                            assert_ne!([center.red(), center.green(), center.blue()], [8, 7, 5]);
                            for y in [54 - lift, 58 - lift] {
                                let border = image.pixel(x, y).unwrap();
                                assert_eq!(
                                    [border.red(), border.green(), border.blue(), border.alpha()],
                                    [8, 7, 5, 255]
                                );
                            }
                        }
                        for x in 30..36 {
                            assert_eq!(
                                image.pixel(x, 58).unwrap().alpha(),
                                0,
                                "feet merged at {x}"
                            );
                        }
                        // Every part, including the tail and overlapping shapes,
                        // must use a single shared 2×2 logical-pixel grid.
                        for y in (0..SIZE).step_by(2) {
                            for x in (0..SIZE).step_by(2) {
                                let pixel = image.pixel(x, y);
                                assert_eq!(pixel, image.pixel(x + 1, y));
                                assert_eq!(pixel, image.pixel(x, y + 1));
                                assert_eq!(pixel, image.pixel(x + 1, y + 1));
                            }
                        }
                        for i in 0..SIZE {
                            for (x, y) in [(i, 0), (i, SIZE - 1), (0, i), (SIZE - 1, i)] {
                                assert_eq!(image.pixel(x, y).unwrap().alpha(), 0);
                            }
                        }
                    }
                }
            }
        }
    }

    fn assert_feet_connected(image: &Pixmap) {
        // Require edge-connected pixels, not diagonal touching.
        let solid = |x: u32, y: u32| {
            let p = image.pixel(x, y).unwrap();
            p.alpha() != 0
        };
        assert!(solid(32, 48), "lower-body anchor is missing");
        let mut visited = vec![false; (SIZE * SIZE) as usize];
        let mut queue = vec![(32_u32, 48_u32)];
        while let Some((x, y)) = queue.pop() {
            let index = (y * SIZE + x) as usize;
            if visited[index] || !solid(x, y) {
                continue;
            }
            visited[index] = true;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if (0..SIZE as i32).contains(&nx) && (0..SIZE as i32).contains(&ny) {
                    queue.push((nx as u32, ny as u32));
                }
            }
        }
        for y in 54..SIZE {
            for x in 0..SIZE {
                if solid(x, y) {
                    assert!(visited[(y * SIZE + x) as usize], "detached foot at {x},{y}");
                }
            }
        }
    }
}
