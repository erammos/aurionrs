use std::ffi::CString;
use std::time::Instant;
use flecs_ecs::prelude::{Entity, EntityView};
use glam::{Mat4, Vec2, Vec3};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use crate::components::{ActiveCameraData, Camera, Local, Mesh, Position, Rotation, Transform, Vertex};
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
fn get_height_on_terrain(
    x: f32,
    z: f32,
    terrain: &Mesh,
    terrain_width: u32,
    terrain_depth: u32,
) -> f32 {
    // Find the integer grid coordinates.
    let grid_x = x as i32;
    let grid_z = z as i32;

    // --- Boundary Check ---
    if grid_x < 0
        || grid_x >= terrain_width as i32 - 1
        || grid_z < 0
        || grid_z >= terrain_depth as i32 - 1
    {
        return 0.0; // Return a default height if outside the terrain
    }

    // --- Fractional Coordinates ---
    // Calculate how far across the grid cell the point is (from 0.0 to 1.0).
    let frac_x = x - grid_x as f32;
    let frac_z = z - grid_z as f32;

    // --- Get the Four Corner Vertices of the Quad ---
    let v00 = terrain.vertices[(grid_z * terrain_width as i32 + grid_x) as usize];
    let v10 = terrain.vertices[(grid_z * terrain_width as i32 + (grid_x + 1)) as usize];
    let v01 = terrain.vertices[((grid_z + 1) * terrain_width as i32 + grid_x) as usize];
    let v11 = terrain.vertices[((grid_z + 1) * terrain_width as i32 + (grid_x + 1)) as usize];

    // --- Bilinear Interpolation ---
    // Helper function for linear interpolation (lerp).
    let lerp = |a: f32, b: f32, t: f32| a * (1.0 - t) + b * t;

    // 1. Interpolate in the X direction for the top and bottom edges.
    let height_top = lerp(v00.position.y, v10.position.y, frac_x);
    let height_bottom = lerp(v01.position.y, v11.position.y, frac_x);

    // 2. Interpolate in the Z direction between the two results.
    let final_height = lerp(height_top, height_bottom, frac_z);

    final_height
}

pub fn player_move(
    player: &mut EntityView,
    mouse_delta: Vec2,
    input_axis: Vec2,
    speed: f32,
    sensitivity: f32,
    dt: f32,
    terrain: &Mesh,
    terrain_width: u32,
    terrain_depth: u32,
) {
    // --- Get Player Components ---
    // We get a copy of the current components from the entity.
    // We'll modify these copies and then write them back at the end.

    //let mut pos = *player.get::<&Position>().expect("Player entity must have a Position component");
    player.get::<(&mut (Transform,Local), &mut Position, &mut Rotation)>(|(transform, pos, rot)| {

        println!("{}", mouse_delta);
        rot.0.y += mouse_delta.x * sensitivity;
        rot.0.x -= mouse_delta.y * sensitivity;
        rot.0.x = rot.0.x.clamp(-89.0, 89.0);
        let yaw_rad = rot.0.y.to_radians();
        let pitch_rad = rot.0.x.to_radians();

        let front = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        ).normalize();

        let right = front.cross(Vec3::Y).normalize();
        let up = right.cross(front).normalize();

        // 4. APPLY MOVEMENT based on input axis and direction vectors
        let move_vertical = front * speed * dt * input_axis.y;
        let move_horizontal = right * speed * dt * input_axis.x;
        pos.0 += move_vertical + move_horizontal;

        // 5. UPDATE PLAYER HEIGHT based on terrain
        let terrain_height = get_height_on_terrain(
            pos.0.x,
            pos.0.z,
            terrain,
            terrain_width,
            terrain_depth
        );
        pos.0.y = terrain_height + 10.0; // Eye-level adjustment

        // 6. UPDATE ENTITY TRANSFORM for rendering
        let center = pos.0 + front;
        let view_matrix = Mat4::look_at_rh(pos.0, center, up);
        transform.0 = view_matrix.inverse();
    });
}
fn main() -> Result<(), String> {
    let graphics = Graphics::new("Rust Engine", 1280, 720)?;
    let mut event_pump = graphics.sdl_context.event_pump()?;
    let shader = load_shader("assets/light.vert", "assets/light.frag")?;
    let cube_mesh = Graphics::create_mesh(CUBE_VERTICES.to_vec(), CUBE_INDICES.to_vec());
    let projection = Mat4::perspective_rh_gl(45.0f32.to_radians(), 1280 as f32 / 720 as f32, 0.1, 100.0);

   let mut world =  Ecs::new();
    let texture = Graphics::load_texture("assets/marble2.jpg")?;

    let cube =  world.create_entity("cube",Vec3::ZERO,Vec3::ONE,Vec3::ZERO,None);
    let camera =  world.create_entity("camera",Vec3 {
        x: -1.0,
        y: 0.0,
        z: 2.0,
    },Vec3::ONE,Vec3::ZERO,None);

    world.add_pbr_shader(cube,shader);
    world.add_mesh(cube,cube_mesh, Some(texture));
    let terrain =  world.create_entity("terrain",Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },Vec3::ONE,Vec3::ZERO,None);

   let terrrain_mesh =  Graphics::create_terrain(100,100);
    world.add_pbr_shader(terrain,shader);
    world.add_mesh(terrain,terrrain_mesh.clone(), Some(texture));

    world.add_camera(camera,Camera {
        projection: projection,
    });

    let (update_system, render_system,camera_system) = world.create_system();
    let mut last_frame_time = Instant::now();


    'running: loop {
        // --- Input Handling ---
        let current_time = Instant::now();
        let dt = (current_time - last_frame_time).as_secs_f32();
        last_frame_time = current_time;

        if dt > 0.0 { // Avoid printing too fast if the window is frozen
            println!(
                " FPS: {:.1}",
                1.0 / dt
            );
        }
        let mut mouse_delta = Vec2::ZERO;
        let mut input_axis = Vec2::ZERO;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                , Event::KeyDown {
                 keycode: Some(Keycode::W) ,..
                }=> {
                    input_axis.y = 1.0;
                },
                // Capture relative mouse movement
                Event::MouseMotion { xrel, yrel, .. } => {
                    mouse_delta.x = xrel as f32;
                    mouse_delta.y = yrel as f32;
                }
                _ => {}
            }
        }

        // --- Keyboard State for Continuous Movement ---
        // `keyboard_state` gives us a snapshot of all keys, which is perfect for
        // smooth movement, rather than relying on single KeyDown/KeyUp events.
        let keyboard_state = event_pump.keyboard_state();

        if keyboard_state.is_scancode_pressed(Scancode::W) {
            input_axis.y = 1.0;
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            input_axis.y = -1.0;
        }
        if keyboard_state.is_scancode_pressed(Scancode::A) {
            input_axis.x = -1.0;
        }
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            input_axis.x = 1.0;
        }

        player_move(&mut camera.entity_view(&world.world),mouse_delta,input_axis,25.0,1.0,dt,&terrrain_mesh,100,100);
        // --- Logic Update ---
        update_system.run();
        camera_system.run();
        graphics.begin_frame();
        render_system.run();

        graphics.end_frame();
    }
    Ok(())
}
