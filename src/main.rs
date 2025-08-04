use std::ffi::CString;
use glam::{Mat4, Vec2, Vec3};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::components::Vertex;
use crate::ecs::Ecs;
use crate::graphics::{load_shader, Graphics};

mod graphics;
mod components;
mod ecs;

const CUBE_VERTICES: [Vertex; 24] = [
    // Front face
    Vertex { position: Vec3::new(-0.5, -0.5, 0.5), normal: Vec3::new(0.0, 0.0, 1.0), uv: Vec2::new(0.0, 0.0) },
    Vertex { position: Vec3::new(0.5, -0.5, 0.5), normal: Vec3::new(0.0, 0.0, 1.0), uv: Vec2::new(1.0, 0.0) },
    Vertex { position: Vec3::new(0.5, 0.5, 0.5), normal: Vec3::new(0.0, 0.0, 1.0), uv: Vec2::new(1.0, 1.0) },
    Vertex { position: Vec3::new(-0.5, 0.5, 0.5), normal: Vec3::new(0.0, 0.0, 1.0), uv: Vec2::new(0.0, 1.0) },
    // Back face
    Vertex { position: Vec3::new(0.5, -0.5, -0.5), normal: Vec3::new(0.0, 0.0, -1.0), uv: Vec2::new(0.0, 0.0) },
    Vertex { position: Vec3::new(-0.5, -0.5, -0.5), normal: Vec3::new(0.0, 0.0, -1.0), uv: Vec2::new(1.0, 0.0) },
    Vertex { position: Vec3::new(-0.5, 0.5, -0.5), normal: Vec3::new(0.0, 0.0, -1.0), uv: Vec2::new(1.0, 1.0) },
    Vertex { position: Vec3::new(0.5, 0.5, -0.5), normal: Vec3::new(0.0, 0.0, -1.0), uv: Vec2::new(0.0, 1.0) },
    // Left face
    Vertex { position: Vec3::new(-0.5, -0.5, -0.5), normal: Vec3::new(-1.0, 0.0, 0.0), uv: Vec2::new(0.0, 0.0) },
    Vertex { position: Vec3::new(-0.5, -0.5, 0.5), normal: Vec3::new(-1.0, 0.0, 0.0), uv: Vec2::new(1.0, 0.0) },
    Vertex { position: Vec3::new(-0.5, 0.5, 0.5), normal: Vec3::new(-1.0, 0.0, 0.0), uv: Vec2::new(1.0, 1.0) },
    Vertex { position: Vec3::new(-0.5, 0.5, -0.5), normal: Vec3::new(-1.0, 0.0, 0.0), uv: Vec2::new(0.0, 1.0) },
    // Right face
    Vertex { position: Vec3::new(0.5, -0.5, 0.5), normal: Vec3::new(1.0, 0.0, 0.0), uv: Vec2::new(0.0, 0.0) },
    Vertex { position: Vec3::new(0.5, -0.5, -0.5), normal: Vec3::new(1.0, 0.0, 0.0), uv: Vec2::new(1.0, 0.0) },
    Vertex { position: Vec3::new(0.5, 0.5, -0.5), normal: Vec3::new(1.0, 0.0, 0.0), uv: Vec2::new(1.0, 1.0) },
    Vertex { position: Vec3::new(0.5, 0.5, 0.5), normal: Vec3::new(1.0, 0.0, 0.0), uv: Vec2::new(0.0, 1.0) },
    // Bottom face
    Vertex { position: Vec3::new(-0.5, -0.5, -0.5), normal: Vec3::new(0.0, -1.0, 0.0), uv: Vec2::new(0.0, 0.0) },
    Vertex { position: Vec3::new(0.5, -0.5, -0.5), normal: Vec3::new(0.0, -1.0, 0.0), uv: Vec2::new(1.0, 0.0) },
    Vertex { position: Vec3::new(0.5, -0.5, 0.5), normal: Vec3::new(0.0, -1.0, 0.0), uv: Vec2::new(1.0, 1.0) },
    Vertex { position: Vec3::new(-0.5, -0.5, 0.5), normal: Vec3::new(0.0, -1.0, 0.0), uv: Vec2::new(0.0, 1.0) },
    // Top face
    Vertex { position: Vec3::new(-0.5, 0.5, 0.5), normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(0.0, 0.0) },
    Vertex { position: Vec3::new(0.5, 0.5, 0.5), normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(1.0, 0.0) },
    Vertex { position: Vec3::new(0.5, 0.5, -0.5), normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(1.0, 1.0) },
    Vertex { position: Vec3::new(-0.5, 0.5, -0.5), normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(0.0, 1.0) },
];

const CUBE_INDICES: [u32; 36] = [
    0, 1, 2, 0, 2, 3,       // Front
    4, 5, 6, 4, 6, 7,       // Back
    8, 9, 10, 8, 10, 11,    // Left
    12, 13, 14, 12, 14, 15, // Right
    16, 17, 18, 16, 18, 19, // Bottom
    20, 21, 22, 20, 22, 23, // Top
];

fn main() -> Result<(), String> {
    let graphics = Graphics::new("Rust Engine", 1280, 720)?;
    let mut event_pump = graphics.sdl_context.event_pump()?;
    let shader = load_shader("assets/light.vert", "assets/light.frag")?;
    let cube_mesh = graphics.create_mesh(CUBE_VERTICES.to_vec(), CUBE_INDICES.to_vec());
    //let (width, height) = window.size();
    let projection = Mat4::perspective_rh_gl(45.0f32.to_radians(), 1280 as f32 / 720 as f32, 0.1, 100.0);
    let view = Mat4::look_at_rh(
        Vec3::new(3.0, 2.0, 3.0), // Camera position
        Vec3::ZERO,               // Target
        Vec3::Y,                  // Up vector
    );
    let model = Mat4::IDENTITY;
    let camera_pos = Vec3::new(3.0, 2.0, 3.0);
    let light_pos = Vec3::new(4.0, 4.0, 2.0);

   let mut world =  Ecs::new();


    'running: loop {
        // --- Input Handling ---
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        // --- Logic Update ---

        world.run_systems();
        // --- Rendering ---
        graphics.begin_frame();
        shader.use_program();

        unsafe {
            // Set all uniforms required by your shader
            shader.set_uniform_mat4("projection", &projection);
            shader.set_uniform_mat4("view", &view);
            shader.set_uniform_mat4("model", &model);

            shader.set_uniform_vec3("lightPos", &light_pos);
            shader.set_uniform_vec3("lightColor", &Vec3::ONE);
            shader.set_uniform_vec3("viewPos", &camera_pos);

            // Tell the shader to use the default color since we have no texture
            let c_name_has_tex = CString::new("has_texture").unwrap();
            let loc_has_tex = gl::GetUniformLocation(shader.id, c_name_has_tex.as_ptr());
            gl::Uniform1i(loc_has_tex, 0); // 0 for false

            let c_name_def_col = CString::new("default_color").unwrap();
            let loc_def_col = gl::GetUniformLocation(shader.id, c_name_def_col.as_ptr());
            gl::Uniform3f(loc_def_col, 0.8, 0.5, 0.2); // An orange color
        }

    // Draw the mesh
    graphics.draw_mesh(&cube_mesh);
        // The render systems are now run from within the ECS schedule
        // The results will be printed to the console.

        graphics.end_frame();
    }
    Ok(())
}
