//! Procedural silhouettes rasterized on a two-pixel grid. Surface clusters are
//! local polygon markings, not sprite parts: the same shapes drive their masks,
//! outlines, shading, feature anchors, and animation.
use crate::{Animation, SIZE, Traits};
use std::f32::consts::TAU;
use tiny_skia::Pixmap;

type Point = [f32; 2];
const INK: u8 = b'#';
const PIXEL: i32 = 2;

fn snap(value: i32) -> i32 {
    (value as f32 / PIXEL as f32).round() as i32 * PIXEL
}

fn color(symbol: u8, coat: [u8; 3]) -> [u8; 4] {
    let base = match symbol {
        b'#' => return [8, 7, 5, 255],
        b's' => [153, 135, 78],
        b'o' => [193, 166, 99],
        b'g' => [215, 185, 117],
        b'h' => [237, 207, 134],
        b'c' => return [225, 227, 226, 255],
        b'w' => return [247, 246, 235, 255],
        _ => return [0; 4],
    };
    let reference = [215, 185, 117];
    let rgb: [u8; 3] =
        std::array::from_fn(|i| (base[i] + coat[i] as i32 - reference[i]).clamp(0, 255) as u8);
    [rgb[0], rgb[1], rgb[2], 255]
}

fn put(image: &mut Pixmap, x: i32, y: i32, rgba: [u8; 4]) {
    if rgba[3] != 0 && (0..SIZE as i32).contains(&x) && (0..SIZE as i32).contains(&y) {
        let offset = (y as usize * SIZE as usize + x as usize) * 4;
        image.data_mut()[offset..offset + 4].copy_from_slice(&rgba);
    }
}

/// One logical pixel wide. Rasterize bends on the same 2×2 grid as every shape.
fn tail_segment(image: &mut Pixmap, from: [i32; 2], to: [i32; 2]) {
    let [mut x, mut y] = from.map(|v| snap(v) / PIXEL);
    let to = to.map(|v| snap(v) / PIXEL);
    let dx = (to[0] - x).abs();
    let dy = -(to[1] - y).abs();
    let sx = if x < to[0] { 1 } else { -1 };
    let sy = if y < to[1] { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        for dy in 0..PIXEL {
            for dx in 0..PIXEL {
                put(image, x * PIXEL + dx, y * PIXEL + dy, color(INK, [0; 3]));
            }
        }
        if [x, y] == to {
            break;
        }
        let twice = error * 2;
        if twice >= dy {
            error += dy;
            x += sx;
        }
        if twice <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn contains(points: &[Point], [x, y]: Point) -> bool {
    let mut inside = false;
    let mut previous = points[points.len() - 1];
    for &current in points {
        if (current[1] > y) != (previous[1] > y)
            && x < (previous[0] - current[0]) * (y - current[1]) / (previous[1] - current[1])
                + current[0]
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

/// Rasterize a local polygon and derive its outline from its actual silhouette.
/// Sampling and markings use local coordinates so integer pose shifts never
/// change the surface pattern.
fn shape(
    image: &mut Pixmap,
    points: &[Point],
    origin: [i32; 2],
    coat: [u8; 3],
    outlined: bool,
    shade: impl Fn(Point) -> u8,
) {
    masked_shape(
        image,
        points,
        origin,
        coat,
        outlined,
        |p| contains(points, p),
        shade,
    );
}

/// Outline the union's boundary, including protruding facial geometry.
fn masked_shape(
    image: &mut Pixmap,
    points: &[Point],
    origin: [i32; 2],
    coat: [u8; 3],
    outlined: bool,
    inside: impl Fn(Point) -> bool,
    shade: impl Fn(Point) -> u8,
) {
    // A shared grid also prevents overlapping parts at odd origins from
    // producing accidental one-pixel slivers or three-pixel-wide clusters.
    let origin = origin.map(snap);
    let min_x = (points.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min) / 2.).floor() as i32 * 2;
    let max_x = points
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32;
    let min_y = (points.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min) / 2.).floor() as i32 * 2;
    let max_y = points
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32;
    for y in (min_y..max_y).step_by(2) {
        for x in (min_x..max_x).step_by(2) {
            let p = [x as f32 + 1., y as f32 + 1.];
            if !inside(p) {
                continue;
            }
            let edge = outlined
                && [[-2., 0.], [2., 0.], [0., -2.], [0., 2.]]
                    .iter()
                    .any(|d| !inside([p[0] + d[0], p[1] + d[1]]));
            let rgba = color(if edge { INK } else { shade(p) }, coat);
            for dy in 0..2 {
                for dx in 0..2 {
                    put(image, origin[0] + x + dx, origin[1] + y + dy, rgba);
                }
            }
        }
    }
}

/// Thin, broken shell creases on the shared pixel grid. Each mark is a short
/// one-cell-wide stroke; no filled pit or broad highlight patch.
fn shell_tone(p: Point, radius: f32) -> u8 {
    let hash = |x: i32, y: i32| {
        let mut n = (x as u32)
            .wrapping_mul(374761393)
            .wrapping_add((y as u32).wrapping_mul(668265263));
        n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        n ^ (n >> 16)
    };
    // More stroke sites, retaining the same one-cell line thickness.
    let cell_x = (p[0] / 6.).floor() as i32;
    let cell_y = (p[1] / 6.).floor() as i32;
    let mut nearest = (f32::INFINITY, 0., 0., 0_u32);
    for row in cell_y - 1..=cell_y + 1 {
        for column in cell_x - 1..=cell_x + 1 {
            let seed = hash(column, row);
            let cx = column as f32 * 6. + 2. + (seed % 2) as f32 * 2.;
            let cy = row as f32 * 6. + 2. + ((seed >> 4) % 2) as f32 * 2.;
            let dx = p[0] - cx;
            let dy = p[1] - cy;
            // Pick a nearby stroke anchor without filling its surrounding area.
            let distance = dx.abs().max(dy.abs()) + (dx.abs() + dy.abs()) * 0.25;
            if distance < nearest.0 {
                nearest = (distance, dx, dy, seed);
            }
        }
    }
    let (_, dx, dy, seed) = nearest;
    let (u, v) = if seed & 1 == 0 { (dx, dy) } else { (dy, dx) };
    let u = (u / 2.).floor() as i32;
    let v = (v / 2.).floor() as i32;
    let crease = match (seed >> 2) % 3 {
        0 => v == 0 && (-1..=1).contains(&u),
        1 => (v == 0 && (-1..=0).contains(&u)) || (u == 0 && v == 1),
        _ => (u == -1 && v == -1) || (v == 0 && (0..=1).contains(&u)),
    };
    let ridge = seed % 3 != 0 && v == -1 && (-1..=0).contains(&u);
    // Small shadow fragments, never a continuous dark fill across the belly.
    if p[0] > radius - 2. {
        return b's';
    }
    if crease {
        return if seed % 4 == 0 && u == 0 { b's' } else { b'o' };
    }
    if ridge {
        return if p[0] > radius * 0.45 { b'g' } else { b'h' };
    }
    if p[0] < -radius + 2. { b'o' } else { b'g' }
}
fn body_radius(t: f32, width: f32, lobes: bool, waist: f32) -> f32 {
    let round = (1. - (t * 2. - 1.).powi(2)).max(0.).sqrt();
    let pinch = if lobes {
        (0.24 + waist * 0.75) * (-((t - 0.52) / 0.14).powi(2)).exp()
    } else {
        0.
    };
    width * 0.5 * round * (1. - pinch)
}

// The upper lobe overlaps the lower one. Its lower edge dips toward the
// center, rather than reading as a straight belt across the whole creature.
fn upper_lobe_bottom(x: f32, height: f32, width: f32, waist: f32) -> f32 {
    let radius = body_radius(0.5, width, true, waist).max(1.);
    // The turned cheek is fuller on the right; the jaw rises toward that side.
    let jaw_slope = 0.10 + 0.12 * ((height - 24.) / 16.).clamp(0., 1.);
    height * 0.48 + 2. * (1. - ((x + 2.) / radius).powi(2)).max(0.) - x * jaw_slope
}

pub(super) fn render(traits: &Traits, animation: Animation, phase: f32) -> Pixmap {
    let mut image = Pixmap::new(SIZE, SIZE).unwrap();
    let phase = phase.rem_euclid(1.) * TAU;
    let walking = animation == Animation::Walk;
    let bob = if walking {
        (phase * 2.).cos()
    } else {
        phase.sin()
    }
    .round() as i32
        * PIXEL;
    let height = (34. * traits.height.clamp(85., 155.) / 130. / 2.).round() * 2.;
    let width = (30. * traits.width.clamp(60., 110.) / 80. / 2.).round() * 2.;
    let waist = traits.waist.clamp(0., 0.48);
    let top = 54 - height as i32 + bob;
    let coat = traits.color;

    // Tail grows from the lower lobe and is painted behind the silhouette.
    // Its bend and tip stay outside the body, even on the widest build.
    let tail_x = 32 + body_radius(0.77, width, traits.lobes, waist).round() as i32 - 2;
    let tail_y = top + (height * 0.77).round() as i32;
    let tail_outer = 32 + width as i32 / 2 + 5;
    let wag = if walking {
        phase.sin().round() as i32
    } else {
        0
    };
    let tail_points = [
        [tail_x, tail_y],
        [tail_outer - 2, tail_y - 2],
        [tail_outer, tail_y - 6],
        [tail_outer + wag, tail_y - 12],
    ];
    for pair in tail_points.windows(2) {
        tail_segment(&mut image, pair[0], pair[1]);
    }
    // Ear roots and facial features follow the upper-body geometry.
    let ear_y = height * 0.16;
    let ear_x = body_radius(0.16, width, traits.lobes, waist) * 0.72;
    // Keep most of the control in the small-ear range. Only its upper end
    // reaches oversized ears, with a lower ceiling than the former 16 pixels.
    let ear_amount = (traits.ears.clamp(30., 64.) - 30.) / 34.;
    let ear_h = (8. + 5. * ear_amount.powi(3)).round();
    for side in [-1., 1.] {
        let points = [
            [-3., 2.],
            [-4., -ear_h + 2.],
            [-2., -ear_h],
            [2., -ear_h],
            [4., -ear_h + 2.],
            [3., 2.],
        ];
        let angle = (traits.ear_angle.clamp(-30., 65.) * side).to_radians();
        let points: Vec<Point> = points
            .into_iter()
            .map(|[x, y]| {
                [
                    x * angle.cos() - y * angle.sin(),
                    x * angle.sin() + y * angle.cos(),
                ]
            })
            .collect();
        shape(
            &mut image,
            &points,
            [
                30 + (side * ear_x * if side < 0. { 0.85 } else { 1.05 }).round() as i32,
                top + ear_y.round() as i32,
            ],
            coat,
            true,
            |p| if p[0] < 0. { b'o' } else { b'h' },
        );
    }

    // Keep the belly rounded all the way to its lowest point. Upper-body bob
    // compresses/extends the lower curve, while the belly stays near the ground.
    let bottom_y = (58 - top) as f32;
    let contour_point = |t: f32, side: f32| {
        let blend = ((t - 0.76) / 0.24).clamp(0., 1.);
        let radius = body_radius(t, width, traits.lobes, waist);
        let y = if t <= 0.76 {
            t * height
        } else {
            height * 0.76 + blend * (bottom_y - height * 0.76)
        };
        // Turn only the head/upper cheek. Keep the belly and foot anchors fixed.
        let turn = ((0.62 - t) / 0.3).clamp(0., 1.);
        let cheek_scale = if side < 0. {
            1. - turn * 0.10
        } else {
            1. + turn * 0.08
        };
        [side * radius * cheek_scale - 2. * turn, y]
    };
    let mut outline = Vec::new();
    for i in 0..=40 {
        outline.push(contour_point(i as f32 / 40., -1.));
    }
    for i in (0..=40).rev() {
        outline.push(contour_point(i as f32 / 40., 1.));
    }
    let body_contains = |p: Point| contains(&outline, p);
    let body_bounds = outline.clone();

    // Both feet sit BELOW the lowest belly row, not below the curve at each
    // side (which could leave the center hanging beneath them). Their fixed
    // stance is independent of build; the upper part overlaps the rounded belly.
    let lowest_body_row = (0..60)
        .step_by(2)
        .rev()
        .find(|&y| {
            (0..64)
                .step_by(2)
                .any(|x| body_contains([(x + 1 - 32) as f32, (y + 1 - top) as f32]))
        })
        .expect("body must have a lower edge");
    let mut feet = Vec::new();
    for side in [-1, 1] {
        let root_x = 32 + side * 6;
        let tucked = walking && phase.sin() * side as f32 > 0.25;
        // A 3×3 logical-pixel tucked foot has room for both a complete border
        // and a colored center. Planting extends only the outward-facing toe.
        let (left, right) = if tucked {
            (-2., 4.)
        } else if side < 0 {
            (-4., 4.)
        } else {
            (-2., 6.)
        };
        let lift = if tucked { PIXEL } else { 0 };
        feet.push(([root_x, lowest_body_row + 2 - lift], left, right, side));
    }

    let body_shade = |p: Point| {
        // Inverse-map the deformed belly into its original surface coordinates.
        // The old shader treated stretched pixels as outside the body and
        // painted an entire lower region shadow-colored.
        let hinge = height * 0.76;
        let t = if p[1] <= hinge {
            p[1] / height
        } else {
            0.76 + (p[1] - hinge) / (bottom_y - hinge) * 0.24
        };
        let radius = body_radius(t, width, traits.lobes, waist);
        shell_tone([p[0], t * height], radius)
    };
    let eye_y = (height * 0.26 / 2.).round() * 2.;
    let face_radius = body_radius(0.28, width, traits.lobes, waist);
    let spacing = (face_radius * traits.eye_spacing.clamp(0.2, 0.65))
        .round()
        .max(2.) as i32;
    let muzzle_w = (traits.snout.clamp(5., 22.) * 0.45)
        .min(face_radius - 2.)
        .max(3.);
    let muzzle_x = -2.;
    // Apply length after choosing the cheek-relative resting position. The old
    // tiny multiplier and final cheek clamp swallowed most slider changes.
    let muzzle_extension = (traits.snout_length.clamp(4., 30.) - 12.) * 0.4;
    let muzzle_ry = (traits.snout.clamp(5., 22.) * 0.22).clamp(4., 6.);
    // Leave room above the single-lobe muzzle's explicit black cheek rim.
    let muzzle_center_y = eye_y + if traits.lobes { 3. } else { 5. } + muzzle_ry;
    let cheek_radius = body_radius(muzzle_center_y / height, width, traits.lobes, waist);
    // Round builds carry a compact muzzle within their cheek. Two-lobe builds
    // let the tip reach the upper lobe's left contour without stretching the dome.
    let tip_x = if traits.lobes {
        (-cheek_radius + 1.6).min(muzzle_x - muzzle_w)
    } else {
        (muzzle_x - muzzle_w - 1.8).max(-cheek_radius + 4.)
    } - muzzle_extension;
    let tip_x = tip_x.min(muzzle_x - if traits.lobes { 4. } else { 6. });
    // Anchor the geometry and pigment to the same broad black end cap. Moving
    // the nose must move the taper too, rather than slide a dot over the cheek.
    let nose_x = snap((tip_x + if traits.lobes { 0. } else { 2. }).round() as i32) as f32;
    let nose_y = snap((muzzle_center_y + if traits.lobes { 0. } else { 2. }).round() as i32) as f32;
    let mut muzzle = vec![[nose_x, nose_y - 2.], [nose_x + 4., nose_y - 4.]];
    for i in 0..=12 {
        let a = std::f32::consts::PI * (1.35 + i as f32 / 12. * 1.15);
        muzzle.push([
            (muzzle_x + a.cos() * muzzle_w - (a.sin() + 1.) * 1.2).max(nose_x + 4.),
            muzzle_center_y + a.sin() * muzzle_ry,
        ]);
    }
    // On a single lobe, end the lower cheek at the nose's bottom edge.
    // Extending it below the nose left an isolated gray cell under the rim.
    let lower_tip = nose_y + if traits.lobes { 4. } else { 2. };
    muzzle.extend_from_slice(&[[nose_x + 4., lower_tip], [nose_x, nose_y + 2.]]);
    let muzzle_contains = |p| contains(&muzzle, p);
    // The accepted muzzle contour stays independent of nose pigment. Flatten
    // the black mark and move it right on two-lobe faces without reshaping it.
    let cap_contains = |p: Point| muzzle_contains(p) && p[0] >= nose_x && p[0] < nose_x + 4.;
    let pigment_x = nose_x + if traits.lobes { 2. } else { 0. };
    let nose_contains = |p: Point| {
        muzzle_contains(p)
            && p[0] >= pigment_x
            && p[0] < pigment_x + 4.
            && p[1] >= nose_y - if traits.lobes { 0. } else { 2. }
            && p[1] < nose_y + 2.
    };
    // Both polygons are expressed in the body's local coordinates. The renderer
    // derives one outline from their union: no seam can cut through the snoot.
    let mut bounds = body_bounds.clone();
    bounds.extend_from_slice(&muzzle);
    let face_contains = |p: Point| {
        (body_contains(p)
            && (!traits.lobes || p[1] <= upper_lobe_bottom(p[0], height, width, waist)))
            || muzzle_contains(p)
    };
    if traits.lobes {
        masked_shape(
            &mut image,
            &body_bounds,
            [32, top],
            coat,
            true,
            body_contains,
            body_shade,
        );
    }
    masked_shape(
        &mut image,
        &bounds,
        [32, top],
        coat,
        true,
        face_contains,
        |p| {
            if muzzle_contains(p) {
                if nose_contains(p) {
                    return INK;
                }
                // A light-gray step behind the slab, then clean white.
                // Neutral muzzle colors stay independent of coat tint.
                if cap_contains(p) || cap_contains([p[0] - 2., p[1]]) {
                    b'c'
                } else {
                    b'w'
                }
            } else {
                // The single-lobe nose meets the lower rim from above.
                // Do not add a second pair of black cells hanging below it.
                if !traits.lobes && p[0] < nose_x + 4. && p[1] >= nose_y + 2. {
                    return body_shade(p);
                }
                // Coat shadow hugs the rounded cheek and trailing cone edge;
                // it is part of the face, rather than a dark nose stamp.
                if muzzle_contains([p[0] - 2., p[1]])
                    || muzzle_contains([p[0] + 2., p[1]])
                    || muzzle_contains([p[0], p[1] + 2.])
                    || muzzle_contains([p[0], p[1] - 2.])
                {
                    let beside_nose = [[-2., 0.], [2., 0.], [0., -2.], [0., 2.]]
                        .iter()
                        .any(|d| cap_contains([p[0] + d[0], p[1] + d[1]]));
                    if !traits.lobes
                        && (beside_nose || p[0] < muzzle_x - 2. || p[1] >= muzzle_center_y + 2.)
                    {
                        INK
                    } else if beside_nose {
                        body_shade(p)
                    } else {
                        b's'
                    }
                } else {
                    body_shade(p)
                }
            }
        },
    );

    // Choose discrete eye cells with a full logical-pixel coat gap around
    // them. A spacing slider must never push an eye into the black outline.
    for side in [-1, 1] {
        let target_x = snap(30 + side * spacing) - 2;
        let target_y = snap(top + eye_y as i32);
        let mut candidates = Vec::new();
        for dy in [-4, -2, 0, 2] {
            for x in (20..44).step_by(2) {
                if (side < 0 && x > 26) || (side > 0 && x < 32) {
                    continue;
                }
                let y = target_y + dy;
                let clear = [-2, 0, 2].into_iter().all(|oy| {
                    [-2, 0, 2, 4].into_iter().all(|ox| {
                        if oy != 0 && (ox == -2 || ox == 4) {
                            return true;
                        }
                        let Some(pixel) = image.pixel((x + ox) as u32, (y + oy) as u32) else {
                            return false;
                        };
                        pixel.alpha() != 0
                            && [pixel.red(), pixel.green(), pixel.blue()] != [8, 7, 5]
                    })
                });
                if clear {
                    candidates.push(((x - target_x).abs() + dy.abs() * 2, [x, y]));
                }
            }
        }
        candidates.sort_by_key(|&(distance, _)| distance);
        if let Some(&(_, origin)) = candidates.first() {
            shape(
                &mut image,
                &[[0., 0.], [4., 0.], [4., 2.], [0., 2.]],
                origin,
                coat,
                false,
                |_| INK,
            );
        }
    }
    // Small outlined palms attach to the lower body's sides, in front of its
    // contour. Swing comes from the same cycle as the feet, with opposite phase.
    for side in [-1, 1] {
        let swing = if walking {
            -(phase.sin() * side as f32).round() as i32
        } else {
            0
        };
        let hand_x = 32 + side * (body_radius(0.70, width, traits.lobes, waist).round() as i32 - 2);
        let hand_y = top + (height * 0.70).round() as i32 + swing;
        let palm: Vec<Point> = [[-2., -2.], [2., -2.], [4., 0.], [4., 4.], [-2., 4.]]
            .into_iter()
            .map(|[x, y]| [x * side as f32, y])
            .collect();
        shape(&mut image, &palm, [hand_x, hand_y], coat, true, |p| {
            if p[1] < 2. { b'h' } else { b'o' }
        });
    }
    // The upper two logical rows overlap the belly, preserving nub length.
    // Outline the complete shape in every pose, then shade only its interior.
    for (foot, left, right, side) in feet {
        // Remove the outer toe's two corner cells while keeping a dark rim
        // around the colored middle. The ankle side stays attached to the body.
        let points = if side < 0 {
            vec![
                [left + 2., -4.],
                [right, -4.],
                [right, 2.],
                [left + 2., 2.],
                [left + 2., 0.],
                [left, 0.],
                [left, -2.],
                [left + 2., -2.],
            ]
        } else {
            vec![
                [left, -4.],
                [right - 2., -4.],
                [right - 2., -2.],
                [right, -2.],
                [right, 0.],
                [right - 2., 0.],
                [right - 2., 2.],
                [left, 2.],
            ]
        };
        shape(&mut image, &points, foot, coat, true, |p| {
            if p[0] < 1. { b'o' } else { b's' }
        });
    }
    image
}
