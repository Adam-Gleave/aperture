#[derive(Default)]
pub struct InputState {
    pub mouse_left_down: bool,
    pub mouse_right_down: bool,
    pub position_delta: Option<[f32; 2]>,
    pub wheel_delta: Option<f32>,
}

impl InputState {
    pub fn tick(&mut self) {
        self.position_delta = None;
        self.wheel_delta = None;
    }
}
