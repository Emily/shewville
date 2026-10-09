//! Render the 25-creature animation atlas for `python3 examples/walking_cast.py`.
use shewville::{Animation, FRAMES, SIZE, Traits, render};
use tiny_skia::{Pixmap, Transform};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let names = [
        "Gregary",
        "Marcussed",
        "Timothan",
        "Jessifer",
        "Bertolomew",
        "Kevinald",
        "Deborath",
        "Carlotron",
        "Elizabert",
        "Nicholump",
        "Samuella",
        "Patricles",
        "Franswick",
        "Robothy",
        "Megalyn",
        "Alansworth",
        "Susannifer",
        "Bradward",
        "Philbertson",
        "Dorothan",
        "Jamothy",
        "Steverson",
        "Claudbert",
        "Williambus",
        "Gwendolump",
    ];
    let coats = [
        [215, 185, 117],
        [225, 185, 107],
        [204, 174, 115],
        [229, 201, 143],
        [199, 178, 123],
    ];
    let mut atlas = Pixmap::new(SIZE * 5, SIZE * 5 * FRAMES).unwrap();
    let mut cast = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let traits = Traits {
            height: 90. + ((i * 17) % 66) as f32,
            width: 60. + ((i * 23 + 20) % 51) as f32,
            lobes: i % 5 != 1 && i % 5 != 4,
            waist: 0.12 + ((i * 7) % 30) as f32 / 100.,
            eye_spacing: 0.30 + ((i * 3) % 25) as f32 / 100.,
            snout: 9. + ((i * 5) % 10) as f32,
            snout_length: 6. + ((i * 7) % 15) as f32,
            ears: 30. + ((i * 11) % 35) as f32,
            ear_angle: -15. + ((i * 13) % 66) as f32,
            color: coats[(i / 5 + i % 5) % coats.len()],
        };
        for frame in 0..FRAMES {
            let sprite = render(&traits, Animation::Walk, frame as f32 / FRAMES as f32);
            atlas.draw_pixmap(
                (i as u32 % 5 * SIZE) as i32,
                (frame * SIZE * 5 + i as u32 / 5 * SIZE) as i32,
                sprite.as_ref(),
                &Default::default(),
                Transform::identity(),
                None,
            );
        }
        cast.push(serde_json::json!({"name": name, "traits": traits}));
    }
    std::fs::create_dir_all("exports")?;
    atlas.save_png("exports/walking-cast-atlas.png")?;
    std::fs::write(
        "exports/walking-cast.json",
        serde_json::to_vec_pretty(&cast)?,
    )?;
    Ok(())
}
