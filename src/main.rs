use raylib::prelude::*;

fn main() {
    unsafe {
        ffi::SetConfigFlags(ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
    }

    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        //Draw polygons
        let polygon_radius = 20.0;
        let mut offset = 0.0;
        for y in 0..15 {
            if y % 2 == 0 {
                offset = polygon_radius * (3.0 as f32).sqrt() * 0.5;
            } else {
                offset = 0.0;
            }
            for x in 0..18 {
                // d.draw_circle(
                //     (15.0 + x as f32 * polygon_radius * 2.0 + offset) as i32,
                //     (15.0 + y as f32 * polygon_radius * 2.0) as i32,
                //     polygon_radius,
                //     Color::BLACK,
                // );
                d.draw_poly(
                    Vector2::new(
                        polygon_radius * (3.0 as f32).sqrt() * 0.5
                            + polygon_radius * (3.0 as f32).sqrt() * (x as f32)
                            + offset,
                        polygon_radius + (y as f32) * polygon_radius * 3.0 / 2.0,
                    ),
                    6,
                    polygon_radius,
                    30.0,
                    Color::BLACK,
                );
                d.draw_poly(
                    Vector2::new(
                        polygon_radius * (3.0 as f32).sqrt() * 0.5
                            + polygon_radius * (3.0 as f32).sqrt() * (x as f32)
                            + offset,
                        polygon_radius + (y as f32) * polygon_radius * 3.0 / 2.0,
                    ),
                    6,
                    polygon_radius * 0.90,
                    30.0,
                    Color::YELLOW,
                );
            }
        }
    }
}
