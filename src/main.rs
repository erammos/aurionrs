use std::ffi::CString;
use glam::{Mat4, Vec2, Vec3};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::components::{ActiveCameraData, Camera, Vertex};
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
    let cube_mesh = Graphics::create_mesh(CUBE_VERTICES.to_vec(), CUBE_INDICES.to_vec());
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
   let cube =  world.create_entity("cube",Vec3::ZERO,Vec3::ONE,Vec3::ZERO,None);
    let camera =  world.create_entity("camera",Vec3 {
        x: -1.0,
        y: 0.0,
        z: 2.0,
    },Vec3::ONE,Vec3::ZERO,None);

    let terrain =  world.create_entity("terrain",Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },Vec3::ONE,Vec3::ZERO,None);

    world.add_camera(camera,Camera {
        projection: projection,
    });
    world.add_pbr_shader(cube,shader);
    world.add_mesh(cube,cube_mesh, None);

    let (update_system, render_system,camera_system) =  world.create_system();


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
        update_system.run();
        camera_system.run();
        graphics.begin_frame();
        render_system.run();

        graphics.end_frame();
    }
    Ok(())
}
