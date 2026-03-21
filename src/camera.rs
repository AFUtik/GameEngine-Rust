use glam;

pub struct Camera {
    position: glam::Vec3,
    fov: f32,
    aspect: f32,
    far: f32,
    near: f32,
    pub pitch: f32,
    pub yaw: f32,

    x_dir: glam::Vec3,
	y_dir: glam::Vec3,
	z_dir: glam::Vec3,
    
    projview: glam::Mat4,
    rotation: glam::Quat
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            fov: 45.0f32.to_radians(),
            aspect: 16.0/9.0,
            far: 0.1,
            near: 100.0,
            pitch: 0.0,
            yaw: 0.0,

            x_dir: glam::Vec3::ONE,
            y_dir: glam::Vec3::ONE,
            z_dir: glam::Vec3::ONE,
            
            projview: glam::Mat4::IDENTITY,
            rotation: glam::Quat::IDENTITY
        }
    }

    fn update_vectors(&mut self) {
        self.x_dir = self.rotation * glam::Vec3::X;
        self.y_dir = self.rotation * glam::Vec3::Y;
        self.z_dir = self.rotation * -glam::Vec3::Z;
    }

    pub fn update(&mut self) {
        let forward = self.rotation * -glam::Vec3::Z;
        let right   = self.rotation * glam::Vec3::X;
        let up      = self.rotation * glam::Vec3::Y;

        let view = glam::Mat4::look_at_rh(
            self.position,
            self.position + forward,
            up,
        );

        let proj = glam::Mat4::perspective_rh(
            self.fov,
            self.aspect,
            0.1,
            1000.0,
        );

        self.projview = proj * view;
    }

    pub fn translate(&mut self, delta: &glam::Vec3) {
        self.position += *delta;
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = 
            glam::Quat::from_axis_angle(glam::Vec3::Z, z) *
            glam::Quat::from_axis_angle(glam::Vec3::Y, y) *
            glam::Quat::from_axis_angle(glam::Vec3::X, x);
        self.update_vectors();
    }

    pub fn rotate_yaw_pitch(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-1.54, 1.54);

        self.rotation =
            glam::Quat::from_axis_angle(glam::Vec3::Y, self.yaw) *
            glam::Quat::from_axis_angle(glam::Vec3::X, self.pitch);
    }

    pub fn forward(&self) -> glam::Vec3 {
        self.rotation * -glam::Vec3::Z
    }

    pub fn right(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::X
    }

    pub fn up(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::Y
    }

    pub fn get_projview(&self) -> &glam::Mat4 { 
        &self.projview
    }
}