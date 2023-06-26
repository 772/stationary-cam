use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Write;

const USAGE: &str = "
Cam disc designer.

Usage:
  stationary-cam <filename.toml>
  stationary-cam -h | --h
";

#[derive(Deserialize, Serialize)]
pub struct Settings {
    pub strokes: Vec<[String; 2]>,
    pub diameter_mm: usize,
    pub center_circle_radius_mm: f32,
    pub outer_circles_radius_mm: f32,
    pub outer_circles_margin_mm: f32,
    pub vertices_per_millimeter: usize,
    pub generate_tooth: bool,
    pub generate_gaps: bool,
    pub display_stroke_names: bool,
}

/// Returns vertices in a 2D room that build a circle.
fn circle(
    radius: f32,
    vertices_per_millimeter: usize,
    center_x: f32,
    center_y: f32,
) -> (Vec<f32>, Vec<f32>) {
    let steps = vertices_per_millimeter * 2 * (std::f32::consts::PI * radius) as usize;
    debug_assert!(steps >= 3);
    let pi = std::f64::consts::PI as f32;
    let mut x_values: Vec<f32> = vec![];
    let mut y_values: Vec<f32> = vec![];
    for i in 0..steps {
        x_values.push(center_x + radius * (pi * i as f32 / steps as f32 * 2.0 - pi / 2.0).cos());
        y_values.push(center_y + radius * (pi * i as f32 / steps as f32 * 2.0 - pi / 2.0).sin());
    }
    (x_values, y_values)
}

/// Rotates a point (x, y) around the center of a circle by a certain angle.
fn rotate_around_center(x: f32, y: f32, angle: f32, radius: f32) -> (f32, f32) {
    (
        angle.cos() * (x - radius) - angle.sin() * (y - radius) + radius,
        angle.sin() * (x - radius) + angle.cos() * (y - radius) + radius,
    )
}

/// Gives the (wdt, hgt) for ```n``` rectangles in a circle with a certain radius.
fn size_of_rectangles_in_circle(
    radius: f32,
    n: usize,
    _width_relative_to_height: f32,
) -> (f32, f32) {
    let a: f32 = (360.0 * (std::f32::consts::PI / 180.0)) / n as f32 / 2.0;
    let t: f32 = 4.0 / 3.0;
    let u: f32 = 13.0 / 9.0;
    (
        radius / (1.0 + t * a.tan() + u * a.tan().powf(2.0)).sqrt(),
        (radius * a.tan()) / (1.0 + t * a.tan() + u * a.tan().powf(2.0)).sqrt(),
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if (args.len() != 2) ^ (args.len() == 2 && (&args[1] == "--h" || &args[1] == "-h")) {
        println!("{USAGE}");
        std::process::exit(0);
    }
    let input_file = &args[1];
    let len: usize = input_file.len();
    let svg_bottom_filename: String = input_file.chars().take(len - 5).collect::<String>() + ".svg";
    let content = std::fs::read_to_string(input_file).unwrap();
    let settings: Settings = toml::from_str(&content).unwrap();
    println!(
        "For 3D model: Set 'Resolution Preview U' in Blender = {}.",
        (std::f32::consts::PI * settings.diameter_mm as f32 / settings.strokes.len() as f32
            * settings.vertices_per_millimeter as f32)
            .floor()
    );
    let mut svg_bottom = format!(
        "<svg style='height:{}mm;width:{}mm;'><g transform='scale(3.543307)'>",
        settings.diameter_mm, settings.diameter_mm
    );
    let radius_usize = settings.diameter_mm / 2;
    let radius: f32 = radius_usize as f32;
    let mut angle = std::f32::consts::PI / 180.0 / 180.0;
    let mut d_cam_disc = String::from("");
    let mut d_hills = String::from("");
    let mut d_tooths = String::from("");
    let mut stroke_names = String::from("");
    let mut stroke_names_bottom = String::from("");
    let max_width = 30.0;
    let (q, p) = size_of_rectangles_in_circle(radius, settings.strokes.len(), 3.0);

    for i in 0..settings.strokes.len() {
        // Stroke names.
        let angle_orig = (360.0 / settings.strokes.len() as f32) * i as f32;
        let (xm, ym) = rotate_around_center(radius, radius - q + p + p, angle, radius);
        stroke_names += &format!("<text text-anchor='middle' stroke='#000' transform='translate({xm},{ym}) rotate({angle_orig})'>{name}</text>", name = settings.strokes[i][0]);
        let strokes_bottom: Vec<[String; 2]> = settings.strokes.clone();
        stroke_names_bottom += &format!("<text text-anchor='middle' stroke='#000' transform='translate({xm},{ym}) rotate({angle_orig})'>{name}</text>", name = strokes_bottom[i][0]);

        // Border and hills.
        let mut stroke: String = settings.strokes[i][1].to_owned();
        if i > 0 {
            stroke = stroke.replace('M', "L");
        }
        let stroke: Vec<&str> = stroke.split(' ').collect();
        let mut neu = String::from("");
        let (x1, y1) = rotate_around_center(-p + radius, radius - q, angle, radius); // left point
        let (x2, y2) = rotate_around_center(p + radius, radius - q, angle, radius); // right point
        let begin_vec: Vec<&str> = stroke[1].split(',').collect();
        let begin = begin_vec[1];
        if begin != "0" {
            let percent = begin.parse::<f32>().unwrap() / max_width;
            let px = -p + percent * 2.0 * p + radius;
            let py = radius - q;
            let x = angle.cos() * (px - radius) - angle.sin() * (py - radius) + radius;
            let y = angle.sin() * (px - radius) + angle.cos() * (py - radius) + radius;
            d_hills += &format!(" M {},{} ", (radius + x1) / 2.0, (radius + y1) / 2.0);
            d_hills += &format!(" L {x1},{y1} ");
            d_hills += &format!(" L {x},{y} ");
            d_hills += &format!(" L {},{} ", (radius + x) / 2.0, (radius + y) / 2.0);
            d_hills += &format!(" L {},{} ", (radius + x1) / 2.0, (radius + y1) / 2.0);
        }
        let end_vec: Vec<&str> = stroke.last().unwrap().split(',').collect();
        let end = end_vec[1];
        if end != max_width.to_string() {
            let percent = end.parse::<f32>().unwrap() / max_width;
            let px = -p + percent * 2.0 * p + radius;
            let py = radius - q;
            let x = angle.cos() * (px - radius) - angle.sin() * (py - radius) + radius;
            let y = angle.sin() * (px - radius) + angle.cos() * (py - radius) + radius;
            d_hills += &format!(" M {},{} ", (radius + x) / 2.0, (radius + y) / 2.0);
            d_hills += &format!(" L {x},{y} ");
            d_hills += &format!(" L {x2},{y2} ");
            d_hills += &format!(" L {},{} ", (radius + x2) / 2.0, (radius + y2) / 2.0);
            d_hills += &format!(" L {},{} ", (radius + x) / 2.0, (radius + y) / 2.0);
        }

        // Cam disc.
        for element in &stroke {
            if !element.contains(',') {
                neu += element;
            } else {
                let elemente: Vec<&str> = element.split(',').collect();
                let px = elemente[1].parse::<f32>().unwrap() * (2.0 * p / max_width) + radius - p;
                let py = elemente[0].parse::<f32>().unwrap() * (2.0 * p / max_width) + radius
                    - q
                    - 2.0 / 3.0 * p;
                let (x, y) = rotate_around_center(px, py, angle, radius);
                neu += &format!(" {x},{y} ");
            }
        }

        if settings.generate_tooth {
            // Bottom right.
            let t = 0.2; // 20 % to the center.
            let (x_temp, y_temp) = (p + radius, radius - q); // basically x2 and y2 without rotation.
            let (x3, y3) = rotate_around_center(
                (1.0 - t) * x_temp + t * radius,
                (1.0 - t) * y_temp + t * radius,
                angle,
                radius,
            );
            // Bottom left.
            let t = 0.235;
            let (x_temp, y_temp) = (-p + radius, radius - q);
            let (x4, y4) = rotate_around_center(
                (1.0 - t) * x_temp + t * radius,
                (1.0 - t) * y_temp + t * radius,
                angle,
                radius,
            );
            // Top left.
            let t = 0.035;
            let (x_temp, y_temp) = (-p + radius, radius - q);
            let (_x5, _y5) = rotate_around_center(
                (1.0 - t) * x_temp + t * radius,
                (1.0 - t) * y_temp + t * radius,
                angle,
                radius,
            );
            d_tooths += &format!(" M {x3},{y3} L {x4},{y4} L {x2},{y2} z");
        }
        d_cam_disc += &neu;
        let angle_degree = 360.0 - angle / (2.0 * std::f32::consts::PI) * 360.0;
        svg_bottom += &format!("<defs><linearGradient id='verlauf{i}' x1='0%' y1='0%' x2='100%' y2='0%' gradientTransform='rotate({angle_degree})'><stop offset='0%' stop-color='#fff' /><stop offset='100%' stop-color='#fff' /></linearGradient></defs>");
        angle += (360.0 * (std::f32::consts::PI / 180.0)) / settings.strokes.len() as f32;
    }
    d_cam_disc += "z ";

    if settings.center_circle_radius_mm > 0.0 {
        let (circle_x, circle_y) = circle(
            settings.center_circle_radius_mm,
            settings.vertices_per_millimeter,
            radius,
            radius,
        );
        d_cam_disc += &format!("M {},{} ", circle_x[0], circle_y[0]);
        for (x, y) in circle_x.iter().zip(circle_y) {
            d_cam_disc += &format!("L {x},{y} ");
        }
        d_cam_disc += "z";
    }

    if settings.outer_circles_margin_mm > 0.0 {
        let half_margin = settings.outer_circles_margin_mm / 2.0;
        let (circle_x, circle_y) = circle(
            settings.outer_circles_radius_mm,
            settings.vertices_per_millimeter,
            radius - half_margin,
            radius - half_margin,
        );
        d_cam_disc += &format!("M {},{} ", circle_x[0], circle_y[0]);
        for (x, y) in circle_x.iter().zip(circle_y) {
            d_cam_disc += &format!("L {x},{y} ");
        }
        d_cam_disc += "z";
        let (circle_x, circle_y) = circle(
            settings.outer_circles_radius_mm,
            settings.vertices_per_millimeter,
            radius + half_margin,
            radius - half_margin,
        );
        d_cam_disc += &format!("M {},{} ", circle_x[0], circle_y[0]);
        for (x, y) in circle_x.iter().zip(circle_y) {
            d_cam_disc += &format!("L {x},{y} ");
        }
        d_cam_disc += "z";
        let (circle_x, circle_y) = circle(
            settings.outer_circles_radius_mm,
            settings.vertices_per_millimeter,
            radius + half_margin,
            radius + half_margin,
        );
        d_cam_disc += &format!("M {},{} ", circle_x[0], circle_y[0]);
        for (x, y) in circle_x.iter().zip(circle_y) {
            d_cam_disc += &format!("L {x},{y} ");
        }
        d_cam_disc += "z";
        let (circle_x, circle_y) = circle(
            settings.outer_circles_radius_mm,
            settings.vertices_per_millimeter,
            radius - half_margin,
            radius + half_margin,
        );
        d_cam_disc += &format!("M {},{} ", circle_x[0], circle_y[0]);
        for (x, y) in circle_x.iter().zip(circle_y) {
            d_cam_disc += &format!("L {x},{y} ");
        }
        d_cam_disc += "z";
    }
    svg_bottom +=
        &format!("<path d='{d_cam_disc}' stroke='none' fill='#ddd' fill-rule='evenodd'/>");
    if settings.generate_gaps {
        svg_bottom += &format!("<path d='{d_hills}' stroke='#999' fill='none' />");
    }
    if settings.generate_tooth {
        svg_bottom += &format!("<path d='{d_tooths}' stroke='#000' fill='none' />");
    }
    if settings.display_stroke_names {
        svg_bottom += &stroke_names_bottom;
    }
    svg_bottom += "</g></svg>";

    File::create(svg_bottom_filename)
        .unwrap()
        .write_all(svg_bottom.as_bytes())
        .unwrap();
}
