use std::array;

use raylib::prelude::*;

const camera_width: i32 = (640.0 * 1.5) as i32;
const camera_height: i32 = (480.0 * 1.5) as i32;

const world_width: usize = 1000;
const world_heigth: usize = 1000;

enum CellType {
    Dirt,
}

impl CellType {
    fn get_weidth(cell: &Self) -> i32 {
        match cell {
            CellType::Dirt => 1,
        }
    }
}

struct Cell {
    cell_type: CellType,
}

fn generate_world() -> [[Cell; world_width]; world_heigth] {
    let world: [[Cell; world_width]; world_heigth] = array::from_fn(|_| {
        array::from_fn(|_| Cell {
            cell_type: CellType::Dirt,
        })
    });

    world
}

fn draw_world(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    camera: &Camera2D,
    shader: &mut Shader,
    map: &[[Cell; world_width]; world_heigth],
) {
    // for y in (((camera.target.y - camera.offset.y / camera.zoom) / polygon_radius) as i32).min(0)
    //     ..(((camera.target.y - camera.offset.y / camera.zoom) / polygon_radius) as i32)
    //         .max(map.len() as i32)
    // {
    //     if y % 2 == 0 {
    //         offset = polygon_radius * (3.0 as f32).sqrt() * 0.5;
    //     } else {
    //         offset = 0.0;
    //     }
    //     for x in (((camera.target.x - camera.offset.x / camera.zoom)
    //         / (polygon_radius * (3.0 as f32).sqrt())) as i32)
    //         .max(0)
    //         ..(((camera.target.y - camera.offset.y / camera.zoom)
    //             / (polygon_radius * (3.0 as f32).sqrt())
    //             * (3.0 as f32).sqrt()) as i32)
    //             .min(map[0].len() as i32)
    //     {

    let polygon_radius: f32 = 20.0;
    let mut offset = 0.0;

    let screen_mouse_pos = rl.get_mouse_position();
    fn screen_to_world(screen_position: &Vector2, camera: &Camera2D) -> Vector2 {
        let mut position = screen_position.clone();
        position.x -= camera.offset.x;
        position.y -= camera.offset.y;
        position.x /= camera.zoom;
        position.y /= camera.zoom;
        position.x -= -camera.target.x;
        position.y -= -camera.target.y;

        position
    }

    // let mouser_world_pos = screen_to_world(
    //     &Vector2::new(screen_mouse_pos.x, screen_mouse_pos.y),
    //     camera,
    // );

    let mouser_world_pos = screen_to_world(&screen_mouse_pos, camera);

    fn axial_to_cube(hex: &Vector2) -> Vector3 {
        let x = hex.x;
        let y = hex.y;
        let z = -x - y;

        Vector3::new(x, y, z)
    }

    fn cube_round(frac: &Vector3) -> Vector3 {
        let mut x = frac.x.round();
        let mut y = frac.y.round();
        let mut z = frac.z.round();

        let q_diff = (x - frac.x).abs();
        let r_diff = (y - frac.y).abs();
        let s_diff = (z - frac.z).abs();

        if q_diff > r_diff && q_diff > s_diff {
            x = -y - z;
        } else if r_diff > s_diff {
            y = -x - z;
        } else {
            z = -x - y;
        }

        Vector3::new(x, y, z)
    }

    fn cube_to_axial(cube: &Vector3) -> Vector2 {
        let x = cube.x;
        let y = cube.y;

        Vector2::new(x, y)
    }

    fn axial_round(hex: &Vector2) -> Vector2 {
        cube_to_axial(&cube_round(&axial_to_cube(&hex)))
    }

    fn pixel_to_pointy_hex(point: &Vector2, size: f32) -> Vector2 {
        let x = point.x / size;
        let y = point.y / size;

        let q = f32::sqrt(3.0) / 3.0 * x - 1.0 / 3.0 * y;
        let r = 2.0 / 3.0 * y;

        axial_round(&Vector2::new(q, r))
    }

    fn axial_to_doublewidth(hex: &Vector2) -> Vector2 {
        let col = 2.0 * hex.x + hex.y;
        let row = hex.y;

        Vector2::new(col, row)
    }

    fn pixel_to_doublewidth(pixel: &Vector2, size: f32) -> Vector2 {
        let axial_pos = pixel_to_pointy_hex(&pixel, size);
        axial_to_doublewidth(&axial_pos)
    }

    // fn pixel_to_doublewidth_simple(pixel: &Vector2, size: f32) -> Vector2 {
    //     let mut x = pixel.x / size;
    //     let mut y = pixel.y / size;

    //     x = x / f32::sqrt(3.0) / 2.0;
    //     y = y / 3.0 / 2.0;

    //     Vector2::new(x * 4.0, y * 4.0)
    // }

    // print!("m-X: {}, m-Y: {}\n", mouser_world_pos.x, mouser_world_pos.y);
    // print!(
    //     "m-X-off: {}, m-Y-off: {}\n",
    //     mouser_world_pos.x - camera.target.x,
    //     mouser_world_pos.y - camera.target.y
    // );
    // print!(
    //     "Camera-y: {}\n",
    //     ((camera.target.y - camera.offset.y / camera.zoom) / polygon_radius) as i32
    // );
    // let x = (mouser_world_pos.x / polygon_radius) as i32;
    // let y = (mouser_world_pos.y / polygon_radius) as i32;
    // print!(
    //     "m2-X: {}, m2-Y: {}\n",
    //     ((3 as f32).sqrt() / 3.0 * x as f32 - 1.0 / 3.0 * y as f32) as i32,
    //     (2.0 / 3.0 * y as f32) as i32
    // );

    // let hex_pos = pixel_to_doublewidth(
    //     &Vector2::new(mouser_world_pos.x, mouser_world_pos.y),
    //     polygon_radius,
    // );

    // print!("Hex-cord: x: {}, y: {}\n", hex_pos.x, hex_pos.y);
    //---------------------------------------
    // let mut d = rl.begin_drawing(&thread);
    // d.clear_background(Color::WHITE);
    // d.draw_fps(0, 0);
    // let mut world_renderer = d.begin_mode2D(camera);

    // //Draw polygons

    // for y in 0..map.len() {
    //     if y % 2 == 1 {
    //         offset = polygon_radius * (3.0 as f32).sqrt() * 0.5;
    //     } else {
    //         offset = 0.0;
    //     }
    //     if polygon_radius + (y as f32) * polygon_radius * 3.0 / 2.0 + polygon_radius * 3.0 / 2.0
    //         < camera.target.y - camera.offset.y / camera.zoom
    //     {
    //         continue;
    //     }

    //     if polygon_radius + (y as f32) * polygon_radius * 3.0 / 2.0 - polygon_radius * 3.0 / 2.0
    //         > camera.target.y + camera.offset.y / camera.zoom
    //     {
    //         break;
    //     }
    //     for x in 0..map[0].len() {
    //         if polygon_radius * (3.0 as f32).sqrt() * 0.5
    //             + polygon_radius * (3.0 as f32).sqrt() * (x as f32)
    //             + offset
    //             + polygon_radius * (3.0 as f32).sqrt()
    //             + offset
    //             < camera.target.x - camera.offset.x / camera.zoom
    //         {
    //             continue;
    //         }

    //         if polygon_radius * (3.0 as f32).sqrt() * 0.5
    //             + polygon_radius * (3.0 as f32).sqrt() * (x as f32)
    //             + offset
    //             - polygon_radius * (3.0 as f32).sqrt()
    //             - offset
    //             > camera.target.x + camera.offset.x / camera.zoom
    //         {
    //             break;
    //         }

    //         world_renderer.draw_poly(
    //             Vector2::new(
    //                 polygon_radius * (3.0 as f32).sqrt() * (x as f32) + offset,
    //                 (y as f32) * polygon_radius * 1.5,
    //             ),
    //             6,
    //             polygon_radius,
    //             30.0,
    //             Color::BLACK,
    //         );
    //         world_renderer.draw_poly(
    //             Vector2::new(
    //                 polygon_radius * (3.0 as f32).sqrt() * (x as f32) + offset,
    //                 (y as f32) * polygon_radius * 3.0 / 2.0,
    //             ),
    //             6,
    //             polygon_radius * 0.90,
    //             30.0,
    //             Color::YELLOW,
    //         );
    //     }
    // }
    //---------------------------------

    let window_height_loc = shader.get_shader_location("window_height");
    let camera_zoom_loc = shader.get_shader_location("camera_zoom");
    let camera_target_loc = shader.get_shader_location("camera_target");
    let camera_offset_loc = shader.get_shader_location("camera_offset");
    let size_loc = shader.get_shader_location("size");

    let w = rl.get_screen_width() as f32;
    shader.set_shader_value(window_height_loc, w);
    shader.set_shader_value(camera_zoom_loc, camera.zoom);
    shader.set_shader_value(camera_target_loc, camera.target);
    shader.set_shader_value(camera_offset_loc, camera.offset);
    shader.set_shader_value(size_loc, polygon_radius);

    let mut d = rl.begin_drawing(thread);
    d.clear_background(Color::WHITE);
    {
        let mut d = d.begin_shader_mode(shader);
        d.draw_rectangle(0, 0, camera_width, camera_height, Color::BLACK);
    }
    d.draw_fps(0, 0);
}

fn draw_gui(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut d = rl.begin_drawing(thread);
    d.draw_fps(0, 0);
}

fn handle_input(rl: &mut RaylibHandle, camera: &mut Camera2D) {
    let mouse_wheel_move = rl.get_mouse_wheel_move();

    if mouse_wheel_move != 0.0 {
        // camera.offset = rl.get_mouse_position();
        // camera.target = rl.get_screen_to_world2D(rl.get_mouse_position(), *camera);
        let mouser_world_pos = rl.get_screen_to_world2D(rl.get_mouse_position(), *camera);

        camera.zoom = (camera.zoom.ln() + mouse_wheel_move * 0.1)
            .exp()
            .clamp(0.001, 20.0);

        let mouser_world_pos2 = rl.get_screen_to_world2D(rl.get_mouse_position(), *camera);

        camera.target += mouser_world_pos - mouser_world_pos2;
    }

    let mouse_delta = rl.get_mouse_delta() / camera.zoom;

    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        camera.target = Vector2::new(
            camera.target.x - mouse_delta.x,
            camera.target.y - mouse_delta.y,
        );
    }
}

fn main() {
    // unsafe {
    //     ffi::SetConfigFlags(ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
    // }

    let (mut rl, thread) = raylib::init()
        .size(camera_width, camera_height)
        .title("Hello, World")
        .build();

    let mut camera = Camera2D {
        offset: Vector2::new(camera_width as f32 / 2.0, camera_height as f32 / 2.0),
        target: Vector2::new(0.0, 0.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    let world = generate_world();

    let mut shader: Shader = rl.load_shader(&thread, None, Some("assets/fragment.glsl"));

    while !rl.window_should_close() {
        //draw_gui(&mut rl, &thread);
        draw_world(&mut rl, &thread, &camera, &mut shader, &world);
        handle_input(&mut rl, &mut camera);
    }
}
