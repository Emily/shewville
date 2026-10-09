//! `cargo run --example style_sheet` writes a native-resolution comparison.
use shewville::{Animation, SIZE, Traits, render};
use tiny_skia::{Pixmap, Transform};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builds = [
        Traits::default(),
        Traits {
            height: 90.,
            width: 105.,
            lobes: false,
            ..Default::default()
        },
        Traits {
            height: 155.,
            width: 60.,
            ears: 30.,
            ..Default::default()
        },
    ];
    let mut sheet = Pixmap::new(SIZE * 3, SIZE * 4).unwrap();
    for (column, traits) in builds.iter().enumerate() {
        for (row, phase) in [0., 0.25, 0.5, 0.75].into_iter().enumerate() {
            let frame = render(traits, Animation::Walk, phase);
            sheet.draw_pixmap(
                column as i32 * SIZE as i32,
                row as i32 * SIZE as i32,
                frame.as_ref(),
                &Default::default(),
                Transform::identity(),
                None,
            );
        }
    }
    std::fs::create_dir_all("exports")?;
    sheet.save_png("exports/style-sheet.png")?;
    magnify(&sheet).save_png("exports/geometry-preview.png")?;
    // Columns: short, default, long. Rows: peanut, round, tall.
    let mut lengths = Pixmap::new(SIZE * 3, SIZE * 3).unwrap();
    for (row, traits) in builds.iter().enumerate() {
        for (column, length) in [4., 12., 30.].into_iter().enumerate() {
            let mut traits = traits.clone();
            traits.snout_length = length;
            let frame = render(&traits, Animation::Idle, 0.);
            lengths.draw_pixmap(
                column as i32 * SIZE as i32,
                row as i32 * SIZE as i32,
                frame.as_ref(),
                &Default::default(),
                Transform::identity(),
                None,
            );
        }
    }
    lengths.save_png("exports/snout-lengths.png")?;
    magnify(&lengths).save_png("exports/snout-lengths-preview.png")?;
    Ok(())
}

fn magnify(sheet: &Pixmap) -> Pixmap {
    // Integer magnification makes the source pixels easy to review without
    // introducing filtering colors. Keep the native transparent sheet as well.
    let mut preview = Pixmap::new(sheet.width() * 4, sheet.height() * 4).unwrap();
    for y in 0..preview.height() {
        for x in 0..preview.width() {
            let pixel = sheet.pixel(x / 4, y / 4).unwrap();
            let rgba = if pixel.alpha() == 0 {
                [224, 220, 206, 255]
            } else {
                [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
            };
            let offset = ((y * preview.width() + x) * 4) as usize;
            preview.data_mut()[offset..offset + 4].copy_from_slice(&rgba);
        }
    }
    preview
}
