#version 460 core

// Utility: axial <-> cube coordinate conversion for hex grids
vec3 axial_to_cube(vec2 hex) {
    float x = hex.x;
    float y = hex.y;
    float z = -x - y;
    return vec3(x, y, z);
}

vec3 cube_round(vec3 frac) {
    float x = round(frac.x);
    float y = round(frac.y);
    float z = round(frac.z);

    float q_diff = abs(x - frac.x);
    float r_diff = abs(y - frac.y);
    float s_diff = abs(z - frac.z);

    if (q_diff > r_diff && q_diff > s_diff) {
        x = -y - z;
    } else if (r_diff > s_diff) {
        y = -x - z;
    } else {
        z = -x - y;
    }

    return vec3(x, y, z);
}

vec2 cube_to_axial(vec3 cube) {
    return vec2(cube.x, cube.y);
}

vec2 axial_round(vec2 hex) {
    return cube_to_axial(cube_round(axial_to_cube(hex)));
}

vec2 pixel_to_pointy_hex(vec2 point, float size) {
    float x = point.x / size;
    float y = point.y / size;

    float q = (sqrt(3.0) / 3.0) * x - (1.0 / 3.0) * y;
    float r = (2.0 / 3.0) * y;

    return axial_round(vec2(q, r));
}

vec2 axial_to_doublewidth(vec2 hex) {
    float col = 2.0 * hex.x + hex.y;
    float row = hex.y;
    return vec2(col, row);
}

vec2 pixel_to_doublewidth(vec2 pixel, float size) {
    vec2 axial_pos = pixel_to_pointy_hex(pixel, size);
    return axial_to_doublewidth(axial_pos);
}

vec2 screen_to_world(vec2 position, float camera_zoom, vec2 camera_target, vec2 camera_offset) {
    position.x -= camera_offset.x;
    position.y -= camera_offset.y;
    position.x /= camera_zoom;
    position.y /= camera_zoom;
    position.x -= -camera_target.x;
    position.y -= -camera_target.y;

    return position;
}

uniform float window_height;
uniform float camera_zoom;
uniform vec2 camera_target;
uniform vec2 camera_offset;
uniform float size;

out vec4 fragColor;

void main()
{
    vec2 cords = gl_FragCoord.xy;
    
    vec2 world_pos = screen_to_world(vec2(cords.x, window_height - cords.y), camera_zoom, camera_target, camera_offset);
    vec2 hex_pos = pixel_to_doublewidth(world_pos, size);

    if (hex_pos.x < 0 || hex_pos.y < 0) {
        discard;
    }

    float color = mod((hex_pos.x * 2.2 + hex_pos.y * 6.3), 1.0);
    
    fragColor = vec4(vec3(color, color, color), 1.0);
}