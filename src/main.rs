use gl::types::{GLchar, GLsizeiptr, GLuint, GLvoid};
use noise::{
    Abs, Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti, ScalePoint, Turbulence, Vector4,
    utils::{self, PlaneMapBuilder},
};
use rand::{Rng, seq::SliceRandom, thread_rng};
use sdl3::{
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    video::{GLContext, GLProfile},
};
use std::{
    ffi::CString,
    fs, mem, ptr,
    time::{Instant, SystemTime},
    vec,
};

const WINDOW_WIDTH: u32 = 1650;
const WINDOW_HEIGHT: u32 = 1080;

const WORLD_WIDTH: i32 = 1000;
const WORLD_HEIGTH: i32 = 1000;

struct Camera2D {
    offset: (f64, f64),
    position: (f64, f64),
    rotation: f64,
    zoom: f64,
}

impl Camera2D {
    fn get_screen_to_world(self: &Self, screen_x: f64, screen_y: f64) -> (f64, f64) {
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
    Grass,
    Dirt,
    Tree,
    River,
    Water,
    MediumWater,
    DeepWater,
    Sand,
    Snow,
    Mountain,
    MediumMountain,
    HighMountain,
    Tundra,
    ShallowWater,
    Ice,
    Cliff,
    MediumCliff,
    Lake,
}

fn increase_color_by_height(color: (f64, f64, f64), height: f64) -> (f64, f64, f64) {
    (
        (color.0 / (1.0 - height * 1.4)).max(0.0).min(255.0),
        (color.1 / (1.0 - height * 1.4)).max(0.0).min(255.0),
        (color.2 / (1.0 - height * 1.4)).max(0.0).min(255.0),
    )
}

fn increase_color_by_height_water(color: (f64, f64, f64), height: f64) -> (f64, f64, f64) {
    (
        (color.0 / (1.0 - height * 7.0)).max(0.0).min(255.0),
        (color.1 / (1.0 - height * 7.0)).max(0.0).min(255.0),
        (color.2 / (1.0 - height * 7.0)).max(0.0).min(255.0),
    )
}

fn decrease_color_by_height(color: (f64, f64, f64), height: f64) -> (f64, f64, f64) {
    (
        (color.0 * (1.0 - height.cbrt())).max(0.0).min(255.0),
        (color.1 * (1.0 - height.cbrt())).max(0.0).min(255.0),
        (color.2 * (1.0 - height.cbrt())).max(0.0).min(255.0),
    )
}

impl CellType {
    fn get_weight(self: &Self) -> i32 {
        match self {
            CellType::Grass => 1,
            CellType::Water => 5,
            CellType::Sand => 2,
            CellType::Snow => 2,
            CellType::Mountain => 100,
            CellType::DeepWater => 20,
            CellType::MediumWater => 10,
            CellType::River => 6,
            CellType::Tundra => 2,
            CellType::MediumMountain => 200,
            CellType::HighMountain => 300,
            CellType::Dirt => 1,
            CellType::ShallowWater => 2,
            CellType::Tree => 5,
            CellType::Ice => 3,
            CellType::Cliff => 5,
            CellType::MediumCliff => 5,
            CellType::Lake => 5,
        }
    }
}

struct Cell {
    cell_type: CellType,
    altitude: f64,
    relative_altitude: f64,
}

impl Cell {
    fn get_color(self: &Self) -> (f64, f64, f64) {
        match self.cell_type {
            CellType::Grass => {
                increase_color_by_height((125.0, 205.0, 127.0), self.relative_altitude)
            }
            CellType::ShallowWater => {
                increase_color_by_height_water((40.0, 100.0, 160.0), self.relative_altitude)
            }
            CellType::Water => {
                increase_color_by_height_water((15.0, 15.0, 160.0), self.relative_altitude)
            }
            CellType::MediumWater => {
                increase_color_by_height_water((22.0, 30.0, 64.0), self.relative_altitude)
            }
            CellType::DeepWater => {
                decrease_color_by_height((30.0, 50.0, 100.0), self.relative_altitude)
            }
            CellType::Sand => {
                increase_color_by_height((230.0, 210.0, 100.0), self.relative_altitude)
            }
            CellType::Snow => {
                increase_color_by_height((230.0, 230.0, 230.0), self.relative_altitude)
            }
            CellType::River => {
                increase_color_by_height((50.0, 100.0, 150.0), self.relative_altitude)
            }
            CellType::Tundra => {
                increase_color_by_height((20.0, 100.0, 20.0), self.relative_altitude)
            }
            CellType::Mountain => {
                decrease_color_by_height((100.0, 100.0, 100.0), self.relative_altitude)
            }
            CellType::MediumMountain => {
                decrease_color_by_height((80.0, 80.0, 80.0), self.relative_altitude)
            }
            CellType::HighMountain => {
                decrease_color_by_height((60.0, 60.0, 60.0), self.relative_altitude)
            }
            CellType::Dirt => {
                increase_color_by_height((196.0, 210.0, 130.0), self.relative_altitude)
            }
            CellType::Tree => increase_color_by_height((50.0, 150.0, 50.0), self.relative_altitude),
            CellType::Ice => {
                increase_color_by_height_water((150.0, 150.0, 200.0), self.relative_altitude)
            }
            CellType::Cliff => {
                decrease_color_by_height((150.0, 150.0, 130.0), self.relative_altitude)
            }
            CellType::MediumCliff => {
                decrease_color_by_height((130.0, 130.0, 110.0), self.relative_altitude)
            }
            CellType::Lake => {
                increase_color_by_height((40.0, 100.0, 160.0), self.relative_altitude)
            }
        }
    }
}

fn get_neighbors(map: &Vec<Vec<Cell>>, x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut neighbors: Vec<(usize, usize)> = vec![];

    let h = map.len() as i32;
    let w = map[0].len() as i32 * 2;

    if x as i32 + 2 < w {
        neighbors.push((x + 2, y));
    }
    if x as i32 + 1 < w && y as i32 - 1 >= 0 {
        neighbors.push((x + 1, y - 1));
    }
    if x as i32 - 1 >= 0 && y as i32 - 1 >= 0 {
        neighbors.push((x - 1, y - 1));
    }
    if x as i32 - 2 >= 0 {
        neighbors.push((x - 2, y));
    }
    if x as i32 - 1 >= 0 && y as i32 + 1 < h {
        neighbors.push((x - 1, y + 1));
    }
    if x as i32 + 1 < w && y as i32 + 1 < h {
        neighbors.push((x + 1, y + 1));
    }

    neighbors
}

fn generate_river(map: &mut Vec<Vec<Cell>>) {
    let mut rng = rand::rng();

    for y in 0..WORLD_HEIGTH as usize {
        for x in 0..WORLD_WIDTH as usize {
            if (matches!(map[y][x].cell_type, CellType::Cliff)
                || matches!(map[y][x].cell_type, CellType::MediumCliff)
                || matches!(map[y][x].cell_type, CellType::Mountain)
                || matches!(map[y][x].cell_type, CellType::MediumMountain)
                || matches!(map[y][x].cell_type, CellType::HighMountain))
                && rng.random_bool(0.0008)
            {
                let mut x = x * 2 + y % 2;
                let mut y = y;

                let mut visited = vec![];

                'finish: loop {
                    map[y][x / 2].cell_type = CellType::River;
                    visited.push((x, y));
                    let neighbors_pos = get_neighbors(map, x, y);
                    if neighbors_pos.len() < 6 {
                        break;
                    }
                    let mut neighbors_pos: Vec<(usize, usize)> = neighbors_pos
                        .iter()
                        .filter(|cell_pos| !(visited.contains(cell_pos)))
                        .cloned()
                        .collect();
                    if neighbors_pos.is_empty() {
                        break;
                    }
                    neighbors_pos.shuffle(&mut rng);
                    let mut min_altitude_cell_pos = neighbors_pos[0];
                    for cell_pos in neighbors_pos {
                        if map[cell_pos.1][cell_pos.0 / 2].altitude
                            < map[min_altitude_cell_pos.1][min_altitude_cell_pos.0 / 2].altitude
                        {
                            min_altitude_cell_pos = cell_pos;
                            break;
                        }

                        if matches!(map[cell_pos.1][cell_pos.0 / 2].cell_type, CellType::Lake)
                            || matches!(map[cell_pos.1][cell_pos.0 / 2].cell_type, CellType::Water)
                            || matches!(
                                map[cell_pos.1][cell_pos.0 / 2].cell_type,
                                CellType::ShallowWater
                            )
                            || matches!(
                                map[cell_pos.1][cell_pos.0 / 2].cell_type,
                                CellType::MediumWater
                            )
                            || matches!(
                                map[cell_pos.1][cell_pos.0 / 2].cell_type,
                                CellType::DeepWater
                            )
                        {
                            break 'finish;
                        }
                    }

                    if map[min_altitude_cell_pos.1][min_altitude_cell_pos.0 / 2].altitude
                        > map[y][x / 2].altitude + 0.04
                    {
                        break;
                    }

                    x = min_altitude_cell_pos.0;
                    y = min_altitude_cell_pos.1;
                }
            }
        }
    }
}

fn generate_world() -> Vec<Vec<Cell>> {
    let time = Instant::now();

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut world: Vec<Vec<Cell>> = vec![];

    let altitude_noise = Fbm::<Perlin>::new(seed as u32)
        .set_octaves(10)
        .set_frequency(0.2);

    let temperature_noise = Fbm::<Perlin>::new(seed as u32 + 10).set_frequency(0.25);

    let vegetation_noise = Fbm::<Perlin>::new(seed as u32 + 20).set_frequency(0.3);
    let vegetation_noise = Turbulence::<Fbm<Perlin>, Perlin>::new(vegetation_noise)
        .set_roughness(20)
        .set_power(2.0);

    let beach_noise = Fbm::<Perlin>::new(seed as u32 + 30).set_frequency(0.35);

    let cliff_noise = RidgedMulti::<Perlin>::new(seed as u32 + 40)
        .set_frequency(1.0)
        .set_attenuation(0.8)
        .set_persistence(5.0)
        .set_octaves(10);

    let lake_noise = Fbm::<Perlin>::new(seed as u32 + 50)
        .set_octaves(10)
        .set_frequency(0.2);

    for y in 0..WORLD_HEIGTH {
        let mut row = Vec::new();
        for x in 0..WORLD_WIDTH {
            let sx = x as f64 * 0.01 as f64;
            let sy = y as f64 * 0.01 as f64;
            let altitude = (altitude_noise.get([sx, sy]) + 1.0) / 2.0;
            let temp = (temperature_noise.get([sx, sy]) + 1.0) / 2.0;
            let vegetation = (vegetation_noise.get([sx, sy]) + 1.0) / 2.0;
            let beach_bias = (beach_noise.get([sx, sy]) + 1.0) / 2.0;
            let cliff_bias = ((cliff_noise.get([sx, sy]) + 1.0) / 2.0).powf(0.1);
            let lake_bias = (lake_noise.get([sx, sy]) + 1.0) / 2.0;

            if altitude > 0.85 {
                row.push(Cell {
                    cell_type: CellType::Snow,
                    relative_altitude: altitude - 0.85,
                    altitude: altitude,
                });
            } else if altitude > 0.81 {
                row.push(Cell {
                    cell_type: CellType::HighMountain,
                    relative_altitude: altitude - 0.81,
                    altitude: altitude,
                });
            } else if altitude > 0.8 {
                row.push(Cell {
                    cell_type: CellType::MediumMountain,
                    relative_altitude: altitude - 0.8,
                    altitude: altitude,
                });
            } else if altitude > 0.78 {
                row.push(Cell {
                    cell_type: CellType::Mountain,
                    relative_altitude: altitude - 0.78,
                    altitude: altitude,
                });
            } else if altitude > 0.65 && cliff_bias > 0.95 {
                row.push(Cell {
                    cell_type: CellType::MediumCliff,
                    relative_altitude: altitude - 0.65,
                    altitude: altitude,
                });
            } else if altitude > 0.65 && cliff_bias > 0.8 {
                row.push(Cell {
                    cell_type: CellType::Cliff,
                    relative_altitude: altitude - 0.65,
                    altitude: altitude,
                });
            }
            // else if altitude > 0.61 && altitude < 0.65 && lake_bias > 0.75 {
            //     row.push(Cell {
            //         cell_type: CellType::Lake,
            //         relative_altitude: altitude - 0.61,
            //         altitude: altitude,
            //     });
            // }
            else if altitude > 0.6 {
                if temp > 0.7 {
                    row.push(Cell {
                        cell_type: CellType::Sand,
                        relative_altitude: altitude - 0.6,
                        altitude: altitude,
                    });
                } else if temp > 0.5 {
                    row.push(Cell {
                        cell_type: CellType::Dirt,
                        relative_altitude: altitude - 0.6,
                        altitude: altitude,
                    });
                } else if temp > 0.4 {
                    if vegetation > 0.6 {
                        row.push(Cell {
                            cell_type: CellType::Tree,
                            relative_altitude: altitude - 0.6,
                            altitude: altitude,
                        });
                    } else {
                        row.push(Cell {
                            cell_type: CellType::Grass,
                            relative_altitude: altitude - 0.6,
                            altitude: altitude,
                        });
                    }
                } else if temp > 0.3 {
                    row.push(Cell {
                        cell_type: CellType::Tundra,
                        relative_altitude: altitude - 0.6,
                        altitude: altitude,
                    });
                } else {
                    row.push(Cell {
                        cell_type: CellType::Snow,
                        relative_altitude: altitude - 0.6,
                        altitude: altitude,
                    });
                }
            } else if altitude > 0.59 && beach_bias > 0.65 && temp > 0.4 {
                row.push(Cell {
                    cell_type: CellType::Sand,
                    relative_altitude: altitude - 0.56,
                    altitude: altitude,
                });
            } else if altitude > 0.56 && temp > 0.3 {
                row.push(Cell {
                    cell_type: CellType::ShallowWater,
                    relative_altitude: altitude - 0.56,
                    altitude: altitude,
                });
            } else if altitude > 0.56 {
                row.push(Cell {
                    cell_type: CellType::Ice,
                    relative_altitude: altitude - 0.56,
                    altitude: altitude,
                });
            } else if altitude > 0.52 {
                row.push(Cell {
                    cell_type: CellType::Water,
                    relative_altitude: altitude - 0.52,
                    altitude: altitude,
                });
            } else if altitude > 0.48 {
                row.push(Cell {
                    cell_type: CellType::MediumWater,
                    relative_altitude: altitude - 0.48,
                    altitude: altitude,
                });
            } else {
                row.push(Cell {
                    cell_type: CellType::DeepWater,
                    relative_altitude: altitude,
                    altitude: altitude,
                });
            }
        }
        world.push(row);
    }

    generate_river(&mut world);

    println!("World generated in: {}s", time.elapsed().as_secs_f32());

    world
}

fn generate_world_colors(map: &Vec<Vec<Cell>>) -> Vec<f32> {
    let mut colors: Vec<f32> = vec![];

    for y in 0..WORLD_HEIGTH as usize {
        for x in 0..WORLD_WIDTH as usize {
            let cell = &map[y][x];
            let (r, g, b) = cell.get_color();
            colors.push(r as f32);
            colors.push(g as f32);
            colors.push(b as f32);
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

        gl::Uniform1f(camera_zoom_loc, camera.zoom as f32);
        gl::Uniform2f(
            camera_position_loc,
            camera.position.0 as f32,
            camera.position.1 as f32,
        );
        gl::Uniform2f(
            camera_offset_loc,
            camera.offset.0 as f32,
            camera.offset.1 as f32,
        );
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
                let mouse_world_pos = camera.get_screen_to_world(mouse_x as f64, mouse_y as f64);

                camera.zoom = (camera.zoom.ln() + y as f64 * 0.1).exp().clamp(0.01, 5.0);

                let mouser_world_pos2 = camera.get_screen_to_world(mouse_x as f64, mouse_y as f64);

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
                    camera.position.0 - xrel as f64 / camera.zoom,
                    camera.position.1 - yrel as f64 / camera.zoom,
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
    gl_attr.set_context_version(4, 3);

    let window = video_subsystem
        .window("Civilizations", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context: GLContext = window.gl_create_context().unwrap();

    gl::load_with(|s| {
        video_subsystem
            .gl_get_proc_address(s)
            .map(|f| f as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null()) as *const _
    });

    let mut camera = Camera2D {
        offset: (WINDOW_WIDTH as f64 / 2.0, WINDOW_HEIGHT as f64 / 2.0),
        position: (0.0, 0.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    let vertex_src = include_str!("../assets/vertex.glsl");
    let fragment_src = include_str!("../assets/fragment.glsl");

    let shader_program = create_shader_program(vertex_src, fragment_src).unwrap();

    let world = generate_world();
    let colors = generate_world_colors(&world);

    create_ssbo(&colors);

    while handle_input(&sdl_context, &mut camera) {
        draw(shader_program, &camera);
        window.gl_swap_window();
    }
}
