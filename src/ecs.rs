use std::ffi::CString;
use std::ptr;
use super::components::*;
use flecs_ecs::prelude::*;
use flecs_ecs::prelude::system::System;
use gl::types::GLsizei;
use glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use crate::graphics;
use crate::graphics::{Graphics, Shader};

pub struct Ecs {
    pub world:World,
}

impl Ecs {

    pub fn new() -> Self {

       let world = World::new();
        world.set(ActiveCameraData {
            pos: Default::default(),
            view: Default::default(),
            projection: Default::default(),
        });
        Self {world: world}
    }
    pub fn create_system(&self) -> (System, System, System)
    {
        let csys=   self.world
            .system_named::<(&(Transform,Global),&Camera, &mut ActiveCameraData)>("Camera System").term_at(2).singleton()
            .each(|(world, camera, active_camera)| {

                active_camera.projection = camera.projection;
                active_camera.view =  world.0.inverse();
                active_camera.pos =  (world.0 * Vec4{x:0.0,y:0.0,z:0.0,w:1.0}).xyz();

            });

        let usys=   self.world
            .system_named::<(&(Transform,Local), Option<&(Transform,Global)>, &mut (Transform, Global))>("Update System").term_at(1).parent().cascade()
            .each(|(local,parent_world,world)| {
                world.0 = local.0;
                if let Some(parent_world) = parent_world {

                    world.0 = parent_world.0 * local.0 ;
                }

            });

        let rsys= self.world
            .system_named::<(&(Transform,Global), &Mesh, Option<&Texture>, &mut PBRShader, &mut ActiveCameraData)>("Render System").term_at(4).singleton()
            .each(|(world, mesh,texture,pbr, camera)| {

                pbr.0.use_program();
                pbr.0.set_uniform_mat4("view",&camera.view);
                pbr.0.set_uniform_mat4("projection",&camera.projection);
                pbr.0.set_uniform_mat4("model",&world.0);

                pbr.0.set_uniform_vec3("lightPos", &Vec3::ONE);
                pbr.0.set_uniform_vec3("lightColor", &Vec3::ONE);
                pbr.0.set_uniform_vec3("viewPos", &camera.pos);
                unsafe {
                    let c_name_has_tex = CString::new("has_texture").unwrap();
                    let loc_has_tex = gl::GetUniformLocation(pbr.0.id, c_name_has_tex.as_ptr());
                    if let Some(texture) = texture
                    {
                        gl::Uniform1i(loc_has_tex, 1); // 0 for false
                    }
                    else
                    {
                        gl::Uniform1i(loc_has_tex, 0);
                        let c_name_def_col = CString::new("default_color").unwrap();
                        let loc_def_col = gl::GetUniformLocation(pbr.0.id, c_name_def_col.as_ptr());
                        gl::Uniform3f(loc_def_col, 0.8, 0.5, 0.2); // An orange col// 0 for false
                    }

                    //
                    gl::BindVertexArray(mesh.vao);
                    gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as GLsizei, gl::UNSIGNED_INT, ptr::null());
                    gl::BindVertexArray(0);
                }
            });
        (usys,rsys, csys)
    }

    pub fn create_entity(&mut self, name: &str, pos: Vec3, scale: Vec3, rot_euler_deg: Vec3, parent: Option<Entity>) -> Entity {
        let local_transform_matrix = Mat4::from_scale_rotation_translation(
            scale,
            Quat::from_euler(glam::EulerRot::YXZ, rot_euler_deg.y.to_radians(), rot_euler_deg.x.to_radians(), rot_euler_deg.z.to_radians()),
            pos,
        );
        let entity = self.world.entity_named(name);
        entity.set(Position(pos))
            .set(Rotation(rot_euler_deg))
            .set(Scale(scale))
            .set_pair::<Transform, Global>(Transform::default())
            .set_pair::<Transform, Local>(Transform (local_transform_matrix));

        if let Some(parent_entity) = parent {
            entity.child_of_id(parent_entity);
        }
        entity.id()
    }
    pub fn add_mesh(&mut self, e: Entity, mesh: Mesh, texture: Option<Texture>) {

        e.entity_view(&self.world).set(mesh);
        if let Some(texture) = texture {
            e.entity_view(&self.world).set(texture);
        }
    }
    pub fn add_pbr_shader(&mut self, e: Entity, shader: Shader) {
        e.entity_view(&self.world).set(PBRShader(shader));
    }

    pub fn add_camera(&mut self, e: Entity, camera: Camera) {
       e.entity_view(&self.world).set(camera);
    }
}