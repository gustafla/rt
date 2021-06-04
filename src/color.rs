use ultraviolet::Vec3;

pub struct Color(Vec3);

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

impl From<Color> for Vec3 {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl Color {
    fn sqrt(self) -> Self {
        Self(Vec3::new(self.0.x.sqrt(), self.0.y.sqrt(), self.0.z.sqrt()))
    }

    fn clamp(self, min: f32, max: f32) -> Color {
        Self(self.0.clamped(Vec3::broadcast(min), Vec3::broadcast(max)))
    }
}

pub const COLOR_CHANNELS: usize = 3;
pub type OutputColor = [u8; COLOR_CHANNELS];

impl From<Color> for OutputColor {
    fn from(color: Color) -> Self {
        let c = Vec3::from(color.sqrt().clamp(0., 0.999)) * 256.;
        [c.x as u8, c.y as u8, c.z as u8]
    }
}
