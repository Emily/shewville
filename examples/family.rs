//! `cargo run --example family`, then `python3 examples/family.py`.
use shewville::{Animation, FRAMES, SIZE, Traits, offspring, render};
use tiny_skia::{Pixmap, Transform};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cast: serde_json::Value =
        serde_json::from_slice(&std::fs::read("exports/walking-cast.json")?)?;
    let parent = |name: &str| -> Result<Traits, Box<dyn std::error::Error>> {
        let member = cast
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == name)
            .ok_or_else(|| format!("Missing parent {name}"))?;
        Ok(serde_json::from_value(member["traits"].clone())?)
    };
    let a = parent("Jamothy")?;
    let b = parent("Gwendolump")?;
    let seed = 42;
    let child = offspring(&a, &b, seed);
    let family = [("Jamothy", a), ("Jamolump", child), ("Gwendolump", b)];
    let mut atlas = Pixmap::new(SIZE * 3, SIZE * FRAMES).unwrap();
    for (column, (_, traits)) in family.iter().enumerate() {
        for frame in 0..FRAMES {
            let sprite = render(traits, Animation::Walk, frame as f32 / FRAMES as f32);
            atlas.draw_pixmap(
                column as i32 * SIZE as i32,
                (frame * SIZE) as i32,
                sprite.as_ref(),
                &Default::default(),
                Transform::identity(),
                None,
            );
        }
    }
    atlas.save_png("exports/jamolump-family-atlas.png")?;
    let members: Vec<_> = family
        .iter()
        .map(|(name, traits)| serde_json::json!({"name":name,"traits":traits}))
        .collect();
    std::fs::write(
        "exports/jamolump-family.json",
        serde_json::to_vec_pretty(
            &serde_json::json!({"seed":seed,"model":"blended phenotype; inherited lobe count; no mutation or age scaling","members":members}),
        )?,
    )?;
    println!("{}", serde_json::to_string_pretty(&family[1].1)?);
    Ok(())
}
