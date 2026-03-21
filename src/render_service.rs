use std:: {collections::HashMap, marker::PhantomData, rc::Rc, cell::RefCell};

use crate::component_system::RenderComponent;

use image::RgbaImage;

use crate::camera::Camera;
use crate::model::{Mesh, Vertex, Transform};
use crate::gpu_resources::{MeshGPU, MaterialGPU, TextureGPU, ResourceController};

pub struct Handle<T> {
    pub handle: u32,
    pub handle_container: u32,
    pub _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new() -> Self {
        Self {
            handle: u32::MAX, 
            handle_container: u32::MAX, 
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

    tag: Option<String>
}

pub struct DrawMaterial {
    pub material: MaterialGPU,

    tag: Option<String>
}

pub struct RenderBounds {
    pub min: glam::Vec3,
    pub max: glam::Vec3
}

pub struct RenderObject {
    pub mesh: Handle<DrawMesh>,
    pub material: Handle<DrawMaterial>,
    pub render_bounds: Option<RenderBounds>,

    pub tag: Option<String>,
}

pub struct DrawCommand {
    pub transform_mat: glam::Mat4,
    pub object_id: Handle<RenderObject>,
}

pub struct DrawCommandScissor {
    object_id: Handle<RenderObject>,
    scissor_rect: [u32; 4]
}

pub struct RenderService {
    controller: Rc<ResourceController>,
    texture_bind_group_layout: Rc<wgpu::BindGroupLayout>,
    
    pub meshes:    Vec<DrawMesh>,
    pub materials: Vec<DrawMaterial>,
    pub renderables: Vec<RenderObject>,

    mesh_map: HashMap<String, Handle<DrawMesh>>,
    mat_map:  HashMap<String, Handle<DrawMaterial>>,
    obj_map:  HashMap<String, Handle<RenderObject>>
}

pub struct RenderState {
    pub draw_commands: Vec<DrawCommand>
}

pub struct RenderContext<'a> {
    pub service: &'a RenderService,
    pub state: &'a mut RenderState,
}

pub type RenderServiceRc = Rc<RefCell<RenderService>>;

// tracks all created and deleted handles in render component. //
// When drop is called, refences to RenderService and deletes all allocated GPU resources in this container by created handles. // 
pub struct HandlesContainer {
    render_service: RenderServiceRc,

    render_objects: Vec<Handle<RenderObject>>,
    allocated_meshes:    Vec<Handle<DrawMesh>>,
    allocated_materials: Vec<Handle<DrawMaterial>>,
} 

impl HandlesContainer {
    pub fn new(service: &RenderServiceRc) -> Self {
        Self {
            render_service: service.clone(),
            render_objects: Vec::new(),
            allocated_meshes: Vec::new(),
            allocated_materials: Vec::new()
        }
    }
}

impl RenderService {
    pub fn new(controller: &Rc<ResourceController>, texture_bind_group_layout: &Rc<wgpu::BindGroupLayout>) -> Self {
        Self {
            controller: controller.clone(),
            texture_bind_group_layout: texture_bind_group_layout.clone(),
            meshes:      Vec::new(),
            materials:   Vec::new(),
            renderables: Vec::new(),

            mesh_map: HashMap::new(),
            mat_map:  HashMap::new(),
            obj_map:  HashMap::new()
        }
    }
    
    pub fn render_object(
        &self, 
        object_id: Handle<RenderObject>,
        transform: &Transform,
        state: &mut RenderState) 
    {
        state.draw_commands.push( DrawCommand { transform_mat: *transform.matrix(), object_id } );
    }


    pub fn register_render_object(
        &mut self,
        mesh_handle: Handle<DrawMesh>,
        material_handle: Handle<DrawMaterial>, 
        container: &mut HandlesContainer,
        tag: Option<String>) -> Handle<RenderObject> 
    {
        let mut render_handle = Handle::<RenderObject>::new();
        render_handle.handle = self.renderables.len() as u32;
        render_handle.handle_container = container.render_objects.len() as u32;
        
        let mut render_object = RenderObject {mesh: mesh_handle, material: material_handle, render_bounds: None, tag: None};
        if let Some(tag_unwrapped) = tag {
            render_object.tag = Some(tag_unwrapped.clone());
            self.obj_map.insert(tag_unwrapped, render_handle);
        }
        self.renderables.push(render_object);

        container.render_objects.push(render_handle);

        

        return render_handle;
    }

    pub fn create_mesh(&mut self, mesh: &Mesh, container: &mut HandlesContainer, tag: Option<String>) -> Handle<DrawMesh> {
        let mesh_gpu = self.controller.create_mesh_gpu(mesh);
        let mut mesh_handle = Handle::<DrawMesh>::new();
        mesh_handle.handle = self.meshes.len() as u32;
        mesh_handle.handle_container = container.allocated_meshes.len() as u32;
        
        let mut draw_mesh = DrawMesh { mesh: mesh_gpu, tag: None };
        if let Some(tag_unwrapped) = tag {
            draw_mesh.tag = Some(tag_unwrapped.clone());
            self.mesh_map.insert(tag_unwrapped, mesh_handle);
        }
        self.meshes.push(draw_mesh);

        container.allocated_meshes.push(mesh_handle);

        mesh_handle
    }

    pub fn create_material(&mut self, albedo: &RgbaImage, container: &mut HandlesContainer, tag: Option<String>) -> Handle<DrawMaterial> {
        let tex_gpu = self.controller.create_texture_gpu(albedo);
        let mat_gpu = self.controller.create_material_gpu(&self.texture_bind_group_layout, tex_gpu);

        let mut mat_handle     = Handle::<DrawMaterial>::new();
        mat_handle.handle = self.materials.len() as u32;
        mat_handle.handle_container = container.allocated_materials.len() as u32;

        let mut draw_mat = DrawMaterial { material: mat_gpu, tag: None };
        if let Some(tag_unwrapped) = tag {
            draw_mat.tag = Some(tag_unwrapped.clone());
            self.mat_map.insert(tag_unwrapped, mat_handle);
        }
        self.materials.push(draw_mat);

        container.allocated_materials.push(mat_handle);

        mat_handle
    }

    pub fn get_render_object(&self, tag: String) -> Option<&Handle<RenderObject>> {
        self.obj_map.get(&tag)
    }

    pub fn get_mesh_handle(&self, tag: String) -> Option<&Handle<DrawMesh>> {
        self.mesh_map.get(&tag)
    }

    pub fn get_material_handle(&self, tag: String) -> Option<&Handle<DrawMaterial>> {
        self.mat_map.get(&tag)
    }

    pub fn remove_render_object(&mut self, handle: Handle<RenderObject>, container: &mut HandlesContainer) {
        let obj = &self.renderables[handle.handle as usize];
        if let Some(tag) = &obj.tag 
        {
            self.obj_map.remove(tag);
        }

        self.renderables.remove(handle.handle as usize);
        container.render_objects.remove(handle.handle_container as usize);
    }

    pub fn delete_mesh(&mut self, handle: Handle<DrawMesh>, container: &mut HandlesContainer) {
        let mesh = &self.meshes[handle.handle as usize];
        if let Some(tag) = &mesh.tag 
        {
            self.mesh_map.remove(tag);
        }

        self.meshes.remove(handle.handle as usize);
        container.allocated_meshes.remove(handle.handle_container as usize);
    }

    pub fn delete_material(&mut self, handle: Handle<DrawMaterial>, container: &mut HandlesContainer) {
        let mat = &self.materials[handle.handle as usize];
        if let Some(tag) = &mat.tag 
        {
            self.mat_map.remove(tag);
        }

        self.materials.remove(handle.handle as usize);
        container.allocated_materials.remove(handle.handle_container as usize);
    }

    fn remove_render_object_direct(&mut self, handle: Handle<RenderObject>) {
        self.renderables.remove(handle.handle as usize);
    }

    fn delete_mesh_direct(&mut self, handle: Handle<DrawMesh>) {
        self.meshes.remove(handle.handle as usize);
    }

    fn delete_material_direct(&mut self, handle: Handle<DrawMaterial>) {
        self.materials.remove(handle.handle as usize);
    }
}

impl Drop for HandlesContainer {
    fn drop(&mut self) {
        let mut service_mut = self.render_service.borrow_mut();
        for mesh_h in self.allocated_meshes.iter()    {service_mut.delete_mesh_direct(*mesh_h);}
        for mat_h in  self.allocated_materials.iter() {service_mut.delete_material_direct(*mat_h);}
        for obj_h in  self.render_objects.iter()      {service_mut.remove_render_object_direct(*obj_h);}
    }
}
