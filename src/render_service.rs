use std:: {collections::HashMap, marker::PhantomData, rc::Rc};

use crate::component_system::RenderComponent;

use image::RgbaImage;

use crate::model::{Mesh, Vertex, Transform};
use crate::gpu_resources::{MeshGPU, MaterialGPU, TextureGPU, ResourceController};

pub struct Handle<T> {
    pub handle: u32,
    pub _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new() -> Self {
        Self {
            handle: u32::MAX, 
            _phantom: PhantomData
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub enum RenderLayer {
    Opaque, 
    Transparent,
    Solid
}

pub struct DrawMesh {
    pub mesh: MeshGPU,
    ref_count: u32,
}

pub struct DrawMaterial {
    pub material: MaterialGPU,
    ref_count: u32,
}

pub struct RenderObject {
    pub mesh: Handle<MeshGPU>,
    pub material: Handle<MaterialGPU>,
    layer: RenderLayer
}

pub struct DrawCommand {
    pub transform_mat: glam::Mat4,
    pub object_id: Handle<RenderObject>,
}

pub struct DrawCommandScissor {
    object_id: Handle<RenderObject>,
    scissor_rect: [u32; 4]
}

pub struct RenderingService {
    controller: Rc<ResourceController>,
    texture_bind_group_layout: Rc<wgpu::BindGroupLayout>,
    
    pub meshes:    Vec<DrawMesh>,
    pub materials: Vec<DrawMaterial>,
    pub renderables: Vec<RenderObject>
}

pub struct RenderState {
    pub draw_commands: Vec<DrawCommand>
}

impl RenderingService {
    pub fn new(controller: &Rc<ResourceController>, texture_bind_group_layout: &Rc<wgpu::BindGroupLayout>) -> Self {
        Self {
            controller: controller.clone(),
            texture_bind_group_layout: texture_bind_group_layout.clone(),
            meshes:      Vec::new(),
            materials:   Vec::new(),
            renderables: Vec::new()
        }
    }

    pub fn register_render_object(
        &mut self,
        mesh_handle: Handle<MeshGPU>,
        material_handle: Handle<MaterialGPU>, 
        render_layer: RenderLayer) -> Handle<RenderObject> 
    {
        let render_handle = Handle::<RenderObject> {handle: self.renderables.len() as u32, _phantom: PhantomData};
        self.renderables.push(RenderObject {mesh: mesh_handle, material: material_handle, layer: render_layer});
        return render_handle;
    }

    pub fn render_object(
        &self, 
        object_id: Handle<RenderObject>,
        transform: &Transform,
        state: &mut RenderState) 
    {
        state.draw_commands.push( DrawCommand { transform_mat: (glam::Mat4::from_translation(transform.pos.as_vec3())
                                                              * glam::Mat4::from_quat(transform.rot.as_quat())
                                                              * glam::Mat4::from_scale(transform.scl.as_vec3())), object_id } );

    }

    pub fn create_mesh(&mut self, mesh: &Mesh) -> Handle<MeshGPU> {
        let mesh_gpu = self.controller.create_mesh_gpu(mesh);
        let mesh_handle     = Handle::<MeshGPU> {handle: self.meshes.len() as u32, _phantom: PhantomData};

        self.meshes.push(DrawMesh { mesh: mesh_gpu, ref_count: 1 });

        mesh_handle
    }

    pub fn create_material(&mut self, albedo: &RgbaImage) -> Handle<MaterialGPU> {
        let tex_gpu  = self.controller.create_texture_gpu(albedo);
        let mate_gpu = self.controller.create_material_gpu(&self.texture_bind_group_layout, tex_gpu);

        let mate_handle     = Handle::<MaterialGPU> {handle: self.materials.len() as u32, _phantom: PhantomData};

        self.materials.push(DrawMaterial { material: mate_gpu, ref_count: 1 });

        mate_handle
    }

    pub fn ref_cnt_mesh_add(&mut self, handle: Handle<MeshGPU>) {
        self.meshes[handle.handle as usize].ref_count += 1;
    }

    pub fn ref_cnt_material_add(&mut self, handle: Handle<MaterialGPU>) {
        self.materials[handle.handle as usize].ref_count += 1;
    }

    pub fn delete_render_object(&mut self, handle: Handle<RenderObject>) {
        let mesh_handle     = self.renderables[handle.handle as usize].mesh;
        let material_handle = self.renderables[handle.handle as usize].material;

        self.delete_mesh(mesh_handle);
        self.delete_material(material_handle);
    }

    pub fn delete_mesh(&mut self, handle: Handle<MeshGPU>) {
        let mesh = &mut self.meshes[handle.handle as usize];
        mesh.ref_count-=1;

        if mesh.ref_count == 0 {
            self.meshes.remove(handle.handle as usize);
        }
    }

    pub fn delete_material(&mut self, handle: Handle<MaterialGPU>) {
        let mat = &mut self.materials[handle.handle as usize];
        mat.ref_count-=1;

        if mat.ref_count == 0 {
            self.materials.remove(handle.handle as usize);
        }
    }
}
