use glam::{Mat4, Vec2, Vec3};
use flecs_ecs::prelude::*;
use crate::graphics::Shader;
// --- Component Struct Definitions ---


#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Transform (pub Mat4);
#[derive(Component,Debug)]
pub struct Local;

#[derive(Component,Debug)]
pub struct Global;

#[derive(Component, Clone, Copy, Debug)]
pub struct Position(pub Vec3);

#[derive(Component, Clone, Copy, Debug)]
pub struct Rotation(pub Vec3); // Euler angles in degrees

#[derive(Component, Clone, Copy, Debug)]
pub struct Scale(pub Vec3);

#[derive(Component, Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Texture {
    pub id: u32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Light {
    pub color: Vec3,
    pub intensity: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Camera {
    pub projection: Mat4,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Emission {
    pub orb_color: Vec3,
    pub intensity: f32,
    pub center_position: Vec3,
    pub radius: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Skybox {
    pub cubemap_id: u32,
}

// --- Tag Components ---
#[derive(Component,Clone, Copy, Debug)]
pub struct PBRShader(pub Shader);

#[derive(Component,Clone, Copy, Debug)]
pub struct EmissiveShader(pub Shader);

// --- Singleton Resources ---
#[derive(Component, Debug)]
pub struct ActiveCameraData {
    pub pos: Vec3,
    pub view: Mat4,
    pub projection: Mat4,
}

#[derive(Component, Debug)]
pub struct ActiveLightData {
    pub pos: Vec3,
    pub color: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}
