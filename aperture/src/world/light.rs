use cgmath::Point3;

pub trait Light {
    fn position(&self) -> [f32; 4];
    
    fn color(&self) -> [f32; 4];
    
    fn power(&self) -> [u32; 4];
}

#[derive(Debug)]
pub struct PointLight {
    pub position: Point3<f32>,
    pub color: [f32; 3],
}

impl Light for PointLight {
    fn position(&self) -> [f32; 4] {
        [
            self.position.x,
            self.position.y,
            self.position.z,
            0.0,
        ]
    }

    fn color(&self) -> [f32; 4] {
        [
            self.color[0],
            self.color[1],
            self.color[2],
            1.0,
        ]
    }
    
    // Constant for now.
    fn power(&self) -> [u32; 4] {
        [2400, 0, 0, 0]
    }
}
