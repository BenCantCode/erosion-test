use image::{ImageBuffer, Rgb, Luma};
use noise::{Fbm, NoiseFn};
use palette::{Hsl, IntoColor, Srgb, RgbHue};
use std::f32;
use std::io;

fn main() {
    let fbm = Fbm::new();

    // 256x256 heightmap
    let mut map: Vec<f32> = Vec::with_capacity(256 * 256);

    // Generates terrain based on layered perlin noise.
    for y in 0..256 {
        for x in 0..256 {
            map.push(
                (fbm.get([(x as f64) / 500. + 86485.4, (y as f64) / 500., 1.]) * (256 as f64)) as f32,
            );
        }
    }

    let mut heightmap_img = ImageBuffer::new(256, 256);
    for x in 0..256 {
        for y in 0..256 {
            heightmap_img.put_pixel(x as u32, y as u32, Luma([map[y * 256 + x] as u8]));
        }
    }
    heightmap_img.save("heightmap.bmp").unwrap();

    let mut angle_img = ImageBuffer::new(256, 256);
    for x in 0..256 {
        for y in 0..256 {
            if x != 0 && x != 255 && y != 0 && y != 255 {
                let angle = get_slope_vector(x as f32, y as f32, &map, 256).0;
                let rgb = Srgb::from(Hsl::new(RgbHue::from_radians(angle), 1.0, 0.5));

                angle_img.put_pixel(x as u32, y as u32, Rgb([(rgb.red * 255.) as u8, (rgb.green * 255.) as u8, (rgb.blue * 255.) as u8]));
            }
        }
    }
    angle_img.save("angle.bmp").unwrap();

    let mut steep_img = ImageBuffer::new(256, 256);
    for x in 0..256 {
        for y in 0..256 {
            if x != 0 && x != 255 && y != 0 && y != 255 {
                steep_img.put_pixel(
                    x as u32,
                    y as u32,
                    Luma([(get_slope_vector(x as f32, y as f32, &map, 256).1 * 64.0) as u8]),
                );
            }
        }
    }
    steep_img.save("steep.bmp").unwrap();
}

struct Rect {
    ymin: f32,
    ymax: f32,
    xmin: f32,
    xmax: f32,
}

fn rect_from_subpixel(x: f32, y: f32) -> Rect {
    Rect {
        ymin: y - 0.5,
        ymax: y + 0.5,
        xmin: x - 0.5,
        xmax: x + 0.5,
    }
}

fn max_f32(a: f32, b: f32) -> f32 {
    if a > b {
        return a;
    }
    return b;
}

fn min_f32(a: f32, b: f32) -> f32 {
    if a < b {
        return a;
    }
    return b;
}

//
fn overlap_area(a: &Rect, b: &Rect) -> f32 {
    let dx = min_f32(a.xmax, b.xmax) - max_f32(a.xmin, b.xmin);
    let dy = min_f32(a.ymax, b.ymax) - max_f32(a.ymin, b.ymin);
    dx * dy
}


// Interpolation method based on overlapping area between pixels.
// I'm not sure if this has a formal name, but it makes intuitive sense to me.
fn get_subpixel_value(x: f32, y: f32, map: &Vec<f32>, len: u32) -> f32 {
    let pixel_rects: Vec<Rect>;
    let subpixel_rect: Rect = rect_from_subpixel(x, y);
    if x % 1.0 != 0.0 && y % 1.0 != 0.0 { //At pixel boundaries we can save some performance by weighing less pixels.
        pixel_rects = vec![
            rect_from_subpixel(x.floor() as f32, y.floor() as f32),
            rect_from_subpixel(x.floor() as f32, y.ceil() as f32),
            rect_from_subpixel(x.ceil() as f32, y.floor() as f32),
            rect_from_subpixel(x.ceil() as f32, y.ceil() as f32),
        ];
    } else if y % 1.0 != 0.0 {
        pixel_rects = vec![
            rect_from_subpixel(x, y.floor() as f32),
            rect_from_subpixel(x, y.ceil() as f32),
        ];
    } else if x % 1.0 != 0.0 {
        pixel_rects = vec![
            rect_from_subpixel(x.floor() as f32, y),
            rect_from_subpixel(x.ceil() as f32, y),
        ];
    } else {
        pixel_rects = vec![rect_from_subpixel(x, y)];
    }

    //Get the values at each surrounding pixel, weigh them based on area overlap, and add them together.
    return pixel_rects.into_iter().fold(0.0, |a, i| {
        a + overlap_area(&i, &subpixel_rect)
            * (map[((i.ymin + 0.5) as usize) * (len) as usize + ((i.xmin + 0.5) as usize)])
    });
}

fn get_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    return ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
}

//Takes point and value map, returns downhill vector
fn get_slope_vector(x: f32, y: f32, map: &Vec<f32>, len: u32) -> (f32, f32) {
    let base_value: f32 = get_subpixel_value(x, y, map, len);
    let left_value: f32 = get_subpixel_value(x - 1., y, map, len) - base_value;
    let right_value: f32 = get_subpixel_value(x + 1., y, map, len) - base_value;
    let up_value: f32 = get_subpixel_value(x, y - 1., map, len) - base_value;
    let down_value: f32 = get_subpixel_value(x, y + 1., map, len) - base_value;

    let x_weighted: f32 = left_value * 1f32 + right_value * -1f32; //Weights are inverted because it goes downhill
    let y_weighted: f32 = up_value * 1f32 + down_value * -1f32;

    let angle: f32;
    if x != 0.0 { //The slope will only be atainable if there is a "run"
        angle = y_weighted.atan2(x_weighted);
    } else {
        if y >= 0.0 {
            angle = f32::consts::PI / 2.;
        } else {
            angle = f32::consts::PI * 1.5;
        }
    }

    let magnitude = get_distance(0., 0., x_weighted, y_weighted);

    return (angle, magnitude);
}

//Currently unused. Will be used when implementing actual particle-based erosion.
fn offset_vector(xy: (f32, f32), vector: (f32, f32)) -> (f32, f32) {
    return (
        xy.0 + vector.0.cos() * vector.1,
        xy.1 + vector.0.sin() * vector.1 * -1.0,
    ); // Inverse y axis
}
