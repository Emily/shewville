# Shewville creature workbench

A small Rust + Bevy experiment: procedural flat creatures, adjustable traits,
timestamp-based idle/walk cycles, and PNG sprite-sheet baking.

## Run

Install stable Rust, then `cargo run`. The first build downloads and compiles Bevy.
Linux needs a working X11/XWayland desktop, graphics drivers, and development
libraries for X11 and xkbcommon. This initial configuration uses X11.

Use the sliders and presets to change proportions. Pause or scrub the cycle to
inspect a pose. The actual-size preview displays 64×64 physical pixels, compensating
for display scaling and UI zoom; the enlarged preview below uses the same frame.
Export writes a new timestamped directory under `exports` with
`sprites.png` and `sprites.json`, including traits, timing, and the ground anchor.

`cargo run -- --bake exports/example` exports the default creature without opening
a window. The destination must not already exist. `cargo test --lib` checks
animation looping and frame bounds across extreme proportions.

## Structure

- `src/lib.rs`: phenotype and sprite-sheet baking. No Bevy types.
- `src/pixel.rs`: procedural contours, local surface markings, feature anchors,
  and deterministic animation sampling on a two-pixel grid.
- `src/main.rs`: Bevy host and egui controls. Preview uses the exact baked renderer.

Frames are 64×64, arranged as 16 columns and two rows (idle, walk), producing a
1024×128 sheet. Coordinates start at the upper-left; anchor (32,58.5) is the ground
point in tile pixels. All builds share the same scale and anchor, leaving room for
ears and movement while preserving relative heights. Shapes are rasterized directly
at 64×64; the GUI magnifies those pixels at integer scales using nearest filtering.
Trait sliders use design units, not output pixels. Idle lasts two seconds,
walk one second. PNG uses transparency, with no baked ground shadow.

This is an initial front-facing art/animation study. The walk is an in-place stylized
cycle, not a locomotion solver. Snout size changes the frontal muzzle; profile snout
length, directional views, pose editing, save/load UI, and village simulation are
future work. The application repaints a small CPU image each preview frame; gameplay
should cache and play the baked sheets instead.

## Pixel-art rendering

The references in `example.webp` and `example2.jpg` guide the palette and surface
markings. The body is a procedural closed contour; lobe count and waist depth
change that contour. Ears, muzzle, feet, and eyes are separate procedural shapes
placed at proportion-relative anchors. Small outlined hands attach to the lower
body and swing with walking. A one-logical-pixel (2×2 output pixels) black tail
curves up behind the body. All part positions and animation poses share that grid.
Feet keep a fixed stance across builds, with a complete black border around a
colored center in both tucked and planted poses. Their upper four pixels overlap
the belly and only the two-pixel sole projects below it. The lower curve
deforms as the upper body bobs; it has no flat baseline or
long leg-like connectors. Shell shading follows the same deformation.
Walking alternates a planted foot and a tucked foot lifted by one logical pixel
(two output pixels). Outer toe corners are cut away for a rounded pixel contour,
while the black rim, colored center, and body attachment remain intact.
Connectivity checks cover the full foot region.
Outlines come from shape boundaries.
The shell surface uses staggered ochre dimples and short raised highlights, with
darker pits on the shaded side. This pattern is sampled in body-local coordinates
and clipped to the geometry; no character bitmap is stretched to create a build.
The muzzle stays smooth. Movement is pixel-snapped and there is no antialiasing.
The two-pixel drawing grid preserves chunky clusters within the 64×64 tile.

`cargo run --example style_sheet` generates a native comparison of peanut, round,
and tall builds in `exports/style-sheet.png`, plus a four-times enlarged comparison
in `exports/geometry-preview.png`. This remains a procedural style study for review.
