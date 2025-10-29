use std::{ffi::CString, fs, mem, ptr, vec};

use gl::types::{GLchar, GLsizeiptr, GLuint, GLvoid};
use noise::Vector4;
use sdl3::{
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    video::{GLContext, GLProfile},
};

const WINDOW_WIDTH: u32 = (640.0 * 1.5) as u32;
const WINDOW_HEIGHT: u32 = (480.0 * 1.5) as u32;

const WORLD_WIDTH: i32 = 1000;
const WORLD_HEIGTH: i32 = 1000;

struct Camera2D {
    offset: (f32, f32),
    position: (f32, f32),
    rotation: f32,
    zoom: f32,
}

impl Camera2D {
    fn get_screen_to_world(self: &Self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let mut world_x = screen_x;
        let mut world_y = screen_y;
        world_x -= self.offset.0;
        world_y -= self.offset.1;
        world_x /= self.zoom;
        world_y /= self.zoom;
        world_x -= -self.offset.0;
        world_y -= -self.offset.1;

        (world_x, world_y)
    }
}

enum CellType {
    Dirt,
}

impl CellType {
    fn get_weight(self: &Self) -> i32 {
        match self {
            CellType::Dirt => 1,
        }
    }

    fn get_color(self: &Self) -> (f32, f32, f32) {
        match self {
            CellType::Dirt => (0.0, 255.0, 0.0),
        }
    }
}

struct Cell {
    cell_type: CellType,
}

fn generate_world() -> Vec<Vec<Cell>> {
    let mut world: Vec<Vec<Cell>> = vec![];

    for _y in 0..WORLD_HEIGTH {
        let mut row = Vec::new();
        for _x in 0..WORLD_WIDTH {
            row.push(Cell {
                cell_type: CellType::Dirt,
            });
        }
        world.push(row);
    }

    world
}

fn generate_world_colors(map: &Vec<Vec<Cell>>) -> Vec<f32> {
    let mut colors: Vec<f32> = vec![];

    for y in 0..WORLD_HEIGTH as usize {
        for x in 0..WORLD_WIDTH as usize {
            let cell = &map[y][x];
            let (r, g, b) = cell.cell_type.get_color();
            colors.push(r);
            colors.push(g);
            colors.push(b);
            colors.push(255.0);
        }
    }

    colors
}

fn compile_shader(src: &str, shader_type: u32) -> Result<u32, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success != (gl::TRUE as gl::types::GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = Vec::with_capacity(len as usize);
            buffer.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
            return Err(String::from_utf8_lossy(&buffer).into_owned());
        }

        Ok(shader)
    }
}

fn create_shader_program(vertex_src: &str, fragment_src: &str) -> Result<u32, String> {
    unsafe {
        let vertex_shader = compile_shader(vertex_src, gl::VERTEX_SHADER)?;
        let fragment_shader = compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;

        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

        if success != (gl::TRUE as gl::types::GLint) {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = Vec::with_capacity(len as usize);
            buffer.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
            return Err(String::from_utf8_lossy(&buffer).into_owned());
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        Ok(program)
    }
}

fn create_ssbo(data: &Vec<f32>) -> GLuint {
    unsafe {
        let mut ssbo: GLuint = 0;
        gl::GenBuffers(1, &mut ssbo);
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);

        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER,
            (data.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
            data.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, ssbo);

        ssbo
    }
}

fn draw(shader_program: u32, camera: &Camera2D) {
    unsafe {
        gl::UseProgram(shader_program);

        let camera_zoom_loc =
            gl::GetUniformLocation(shader_program, b"camera_zoom\0".as_ptr() as *const GLchar);
        let camera_position_loc = gl::GetUniformLocation(
            shader_program,
            b"camera_position\0".as_ptr() as *const GLchar,
        );
        let camera_offset_loc =
            gl::GetUniformLocation(shader_program, b"camera_offset\0".as_ptr() as *const GLchar);
        let size_loc = gl::GetUniformLocation(shader_program, b"size\0".as_ptr() as *const GLchar);
        let world_height_loc =
            gl::GetUniformLocation(shader_program, b"world_height\0".as_ptr() as *const GLchar);
        let world_width_loc =
            gl::GetUniformLocation(shader_program, b"world_width\0".as_ptr() as *const GLchar);

        let size: f32 = 20.0;

        gl::Uniform1f(camera_zoom_loc, camera.zoom);
        gl::Uniform2f(camera_position_loc, camera.position.0, camera.position.1);
        gl::Uniform2f(camera_offset_loc, camera.offset.0, camera.offset.1);
        gl::Uniform1f(size_loc, size);
        gl::Uniform1f(world_height_loc, WORLD_HEIGTH as f32);
        gl::Uniform1f(world_width_loc, WORLD_WIDTH as f32);

        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        let vertices: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u32>()) as isize,
            indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            (2 * std::mem::size_of::<f32>()) as i32,
            std::ptr::null(),
        );

        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

        gl::BindVertexArray(0);
        gl::UseProgram(0);
    }
}

fn handle_input(sdl_context: &sdl3::Sdl, camera: &mut Camera2D) -> bool {
    let mut events = sdl_context.event_pump().unwrap();

    for event in events.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return false,
            Event::MouseWheel {
                y,
                mouse_x,
                mouse_y,
                ..
            } => {
                let mouse_world_pos = camera.get_screen_to_world(mouse_x, mouse_y);

                camera.zoom = (camera.zoom.ln() + y * 0.1).exp().clamp(0.001, 20.0);

                let mouser_world_pos2 = camera.get_screen_to_world(mouse_x, mouse_y);

                camera.position = (
                    camera.position.0 + mouse_world_pos.0 - mouser_world_pos2.0,
                    camera.position.1 + mouse_world_pos.1 - mouser_world_pos2.1,
                );
            }
            Event::MouseMotion {
                xrel,
                yrel,
                mousestate,
                ..
            } if mousestate.is_mouse_button_pressed(MouseButton::Left) => {
                camera.position = (
                    camera.position.0 - xrel / camera.zoom,
                    camera.position.1 - yrel / camera.zoom,
                );
            }
            _ => {}
        }
    }

    true
}

pub fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(4, 6);

    let window = video_subsystem
        .window("Civilizations", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context: GLContext = window.gl_create_context().unwrap();

    gl::load_with(|s| video_subsystem.gl_get_proc_address(s).unwrap() as *const _);

    let mut camera = Camera2D {
        offset: (WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0),
        position: (0.0, 0.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    let vertex_src = fs::read_to_string("./assets/vertex.glsl").unwrap();
    let fragment_src = fs::read_to_string("./assets/fragment.glsl").unwrap();

    let shader_program = create_shader_program(&vertex_src, &fragment_src).unwrap();

    let world = generate_world();
    let colors = generate_world_colors(&world);

    create_ssbo(&colors);

    while handle_input(&sdl_context, &mut camera) {
        draw(shader_program, &camera);
        window.gl_swap_window();
    }
}
