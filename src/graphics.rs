use super::components::{Mesh, Texture, Vertex};
use sdl2::video::{GLProfile, Window};
use sdl2::{Sdl, VideoSubsystem};
use std::ffi::{c_void, CString};
use std::fs;
use std::ptr;
use flecs_ecs::macros::Component;
use gl::types::GLsizei;
use glam::{vec2, vec3, Vec3};
use noise::{Fbm, NoiseFn, Perlin};

#[derive(Clone, Copy, Debug)]
pub struct Shader {
    pub id: u32
}

impl Shader {
    pub  fn set_uniform_mat4(&self, name: &str, mat: &glam::Mat4) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.to_cols_array().as_ptr());
        }
    }

    pub fn set_uniform_vec3(&self, name: &str, vec: &glam::Vec3) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform3fv(location, 1, vec.to_array().as_ptr());
        }
    }

    pub  fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

pub struct Graphics {
    pub sdl_context: Sdl,
    pub window: Window,
    _gl_context: sdl2::video::GLContext,
}

impl Graphics {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        gl_attr.set_double_buffer(true);
        gl_attr.set_depth_size(24);
        gl_attr.set_context_flags().debug().set();

        let window = video_subsystem
            .window(title, width, height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let _gl_context = window.gl_create_context()?;
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

        // // Setup OpenGL debug callback
        // unsafe {
        //     gl::Enable(gl::DEBUG_OUTPUT);
        //     gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        //     //gl::DebugMessageCallback(Some(gl_debug_callback), ptr::null());
        //    // gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, ptr::null(), gl::TRUE);
        // }

        Ok(Graphics {
            sdl_context,
            window,
            _gl_context,
        })
    }

    pub fn begin_frame(&self) {
        unsafe {
            gl::Viewport(0, 0, self.window.size().0 as i32, self.window.size().1 as i32);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
        }
    }

    pub fn end_frame(&self) {
        self.window.gl_swap_window();
    }
    fn perlin_noise(x: f32, y: f32, octaves: i32, lacunarity: f32, persistence: f32) -> f32 {
        // Create a new Fractal Brownian Motion instance with a Perlin noise source.
        let mut fbm = Fbm::<Perlin>::new(0); // Using a seed of 0 for reproducibility
        fbm.octaves = octaves as usize;
        fbm.lacunarity = lacunarity as f64;
        fbm.persistence = persistence as f64;

        // The noise function expects a 2D point as an array of f64.
        let point = [x as f64, y as f64];

        // Get the noise value, which is in the range of roughly [-1.0, 1.0].

        let noise_value = fbm.get(point) as f32;

        // Normalize the value to the [0.0, 1.0] range, which is ideal for heightmaps.
        (noise_value + 1.0) / 2.0
    }
    pub fn create_terrain(terrain_width: u32, terrain_height: u32) -> Mesh {
        let num_vertices = (terrain_width * terrain_height) as usize;
        let mut vertices = vec![Vertex::default(); num_vertices];

        // Pre-allocate vector capacity for performance.
        let num_indices = ((terrain_width - 1) * (terrain_height - 1) * 6) as usize;
        let mut indices = Vec::with_capacity(num_indices);

        let perlin_scale = 0.02;     // Zoom out to create large features
        let mountain_height = 10.0; // A multiplier to make mountains taller

        // --- 1. Generate Vertices and Height ---
        for y in 0..terrain_height {
            for x in 0..terrain_width {
                let index = (y * terrain_width + x) as usize;

                // Generate height using the Perlin noise function.
                let height = Graphics::perlin_noise(
                    x as f32 * perlin_scale,
                    y as f32 * perlin_scale,
                    6,      // octaves
                    2.0,    // lacunarity
                    0.5,    // persistence
                );

                // Set vertex position and UVs
                vertices[index].position = vec3(x as f32, height * mountain_height, y as f32);
                vertices[index].uv = vec2(
                    x as f32 / (terrain_width - 1) as f32,
                    y as f32 / (terrain_height - 1) as f32,
                );
            }
        }

        // --- 2. Calculate Normals ---
        for y in 0..terrain_height {
            for x in 0..terrain_width {
                let index = (y * terrain_width + x) as usize;

                // Default normal pointing straight up.
                let mut normal = Vec3::Y;

                // Compute normal using the central difference method, if not on an edge.
                if x > 0 && x < terrain_width - 1 && y > 0 && y < terrain_height - 1 {
                    let left = vertices[index - 1].position;
                    let right = vertices[index + 1].position;
                    let down = vertices[index - terrain_width as usize].position;
                    let up = vertices[index + terrain_width as usize].position;

                    let dx = right - left;
                    let dy = up - down; // In C, you had up - down, which is conventional too.

                    normal = dx.cross(dy).normalize();
                }

                vertices[index].normal = normal;
            }
        }

        // --- 3. Generate Indices ---
        for y in 0..terrain_height - 1 {
            for x in 0..terrain_width - 1 {
                let top_left = y * terrain_width + x;
                let top_right = top_left + 1;
                let bottom_left = (y + 1) * terrain_width + x;
                let bottom_right = bottom_left + 1;

                // First triangle (top-left, bottom-left, top-right)
                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                // Second triangle (top-right, bottom-left, bottom-right)
                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }
      Graphics::create_mesh(vertices, indices)
    }
    pub fn create_mesh(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<Vertex>()) as isize,
                vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Position
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, ptr::null());
            gl::EnableVertexAttribArray(0);
            // Normal
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, (3 * std::mem::size_of::<f32>()) as *const c_void);
            gl::EnableVertexAttribArray(1);
            // UV
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, (6 * std::mem::size_of::<f32>()) as *const c_void);
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);
        }

        Mesh {
            vertices,
            indices,
            vao,
            vbo,
            ebo,
        }
    }

    pub fn load_texture(&self, path: &str) -> Result<Texture, String> {
        let img = image::open(path).map_err(|e| e.to_string())?.into_rgb8();
        let (width, height) = img.dimensions();
        let data = img.into_raw();

        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                width as i32,
                height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Ok(Texture { id })
    }
}

pub fn load_shader(vs_path: &str, fs_path: &str) -> Result<Shader, String> {
    let vs_src = fs::read_to_string(vs_path).map_err(|e| format!("Failed to read vertex shader {}: {}", vs_path, e))?;
    let fs_src = fs::read_to_string(fs_path).map_err(|e| format!("Failed to read fragment shader {}: {}", fs_path, e))?;

    unsafe {
        let vs = compile_shader(&vs_src, gl::VERTEX_SHADER)?;
        let fs = compile_shader(&fs_src, gl::FRAGMENT_SHADER)?;
        let program = link_program(vs, fs)?;
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        Ok(Shader { id: program })
    }
}

// Helper functions for loading shaders
fn compile_shader(src: &str, ty: gl::types::GLenum) -> Result<u32, String> {
    unsafe {
        let shader = gl::CreateShader(ty);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut info_log = Vec::with_capacity(len as usize);
            info_log.set_len(len as usize);
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut i8);
            return Err(String::from_utf8_lossy(&info_log).to_string());
        }
        Ok(shader)
    }
}

fn link_program(vs: u32, fs: u32) -> Result<u32, String> {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut info_log = Vec::with_capacity(len as usize);
            info_log.set_len(len as usize);
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut i8);
            return Err(String::from_utf8_lossy(&info_log).to_string());
        }
        Ok(program)
    }
}

// OpenGL Debug Callback
extern "system" fn gl_debug_callback(
    _source: u32, _type: u32, _id: u32, _severity: u32,
    _length: i32, message: *const i8, _user_param: *mut c_void,
) {
    let message = unsafe { std::ffi::CStr::from_ptr(message).to_string_lossy() };
    println!("GL DEBUG: {}", message);
}