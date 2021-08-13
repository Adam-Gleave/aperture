use cgmath::{EuclideanSpace, InnerSpace, Matrix3, Matrix4, One, Point3, Rad, Vector3};

const SPEED: f32 = 2.0;

pub struct Camera {
    pub view_matrix: Matrix4<f32>,
    pub eye: Point3<f32>,
    pub look_at: Point3<f32>,
    pub up: Vector3<f32>,
}

impl Camera {
    pub fn new(eye: Point3<f32>, look_at: Point3<f32>, up: Vector3<f32>) -> Self {
        let mut camera = Self {
            view_matrix: Matrix4::one(),
            eye,
            look_at,
            up,
        };

        camera.update_view_matrix();
        camera
    }

    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let delta_x = delta_x * SPEED;
        let delta_y = delta_y * SPEED;

        let cos_angle = self.view_dir().dot(self.up);
        let delta_y = if cos_angle * delta_y.signum() >= 0.9999 {
            0.0
        } else {
            delta_y
        };

        let position = self.eye;
        let pivot = self.look_at;

        let rotation_x = Matrix3::from_axis_angle(self.up, Rad(delta_x));
        let position = (rotation_x * (position - pivot)) + pivot.to_vec();

        let rotation_y = Matrix3::from_axis_angle(self.right_vector(), Rad(delta_y));
        let final_position = (rotation_y * (Point3::from_vec(position) - pivot)) + pivot.to_vec();

        self.up = rotation_y * self.up;
        self.set_position(Point3::from_vec(final_position));
    }

    pub fn zoom(&mut self, delta: f32) {
        let mut position = self.eye;
        position += self.view_dir() * delta * SPEED;

        self.set_position(position);
    }

    pub fn translate(&mut self, delta_x: f32, delta_y: f32) {
        let mut position = self.eye.to_vec();
        position -= self.up * delta_y * SPEED * 0.01;
        position += self.right_vector() * delta_x * SPEED * 0.01;

        let mut center = self.look_at.to_vec();
        center -= self.up * delta_y * SPEED * 0.01;
        center += self.right_vector() * delta_x * SPEED * 0.01;

        self.eye = Point3::from_vec(position);
        self.look_at = Point3::from_vec(center);

        self.update_view_matrix();
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    pub fn update_view_matrix(&mut self) {
        self.view_matrix = Matrix4::look_at_rh(self.eye, self.look_at, self.up);
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.eye = position;
        self.update_view_matrix();
    }

    pub fn view_dir(&self) -> Vector3<f32> {
        Vector3::new(
            -self.view_matrix[0][2],
            -self.view_matrix[1][2],
            -self.view_matrix[2][2],
        )
    }

    pub fn right_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.view_matrix[0][0],
            self.view_matrix[1][0],
            self.view_matrix[2][0],
        )
    }
}
