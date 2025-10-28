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
    let polygon_radius: f32 = 20.0;
    let mut offset = 0.0;

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
