use super::components::*;
use flecs_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};
use crate::graphics::Shader;

pub struct Ecs {
    pub world: World,
}


impl Ecs {

    pub fn new() -> Self {

       let world = World::new();
        world
            .system_named::<(&(Transform,Local), Option<&(Transform,Global)>, &mut (Transform, Global),)>("Move").term_at(1).parent().cascade()
            .each(|(local,parent_world, world)| {
                world.0 = local.0;
                if let Some(parent_world) = parent_world {

                    world.0 = parent_world.0 * local.0 ;
                }

            });

        world
            .system_named::<(&(Transform,Global), &Mesh, Option<&Texture>, &mut PBRShader, &mut Camera)>("Shader").term_at(3).singleton().term_at(4).singleton()
            .each(|(world, mesh,texture,pbr, ActiveCameraData)| {

            });

        Self { world }
    }

    pub fn run_systems(&mut self) {


        self.world.progress();
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
    pub fn add_mesh(&mut self, e: &mut Entity, mesh: Mesh, texture: Option<Texture>) {


        e.entity_view(&self.world).set(mesh);
        if let Some(texture) = texture {
            e.entity_view(&self.world).set(texture);
        }
    }

}