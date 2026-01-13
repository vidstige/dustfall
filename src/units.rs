#[derive(Debug, Clone, Copy)]
pub struct PressureScale {
    pascal_per_unit: f32,
}

impl PressureScale {
    pub fn new(pascal_per_unit: f32) -> Self {
        assert!(pascal_per_unit > 0.0, "pascal_per_unit must be positive");
        Self { pascal_per_unit }
    }

    pub fn to_pascal(self, pressure_units: i32) -> f32 {
        pressure_units as f32 * self.pascal_per_unit
    }

    pub fn from_pascal(self, pascal: f32) -> i32 {
        assert!(pascal >= 0.0, "pascal must be non-negative");
        (pascal / self.pascal_per_unit).round() as i32
    }

    pub fn pressure_for_parts(self, pascal: f32) -> i32 {
        self.from_pascal(pascal)
    }
}

// 100 Pa per unit puts 6-10 units in the 600-1000 Pa Mars range.
pub const MARS_ATMOSPHERE_PRESSURE_SCALE: PressureScale = PressureScale {
    pascal_per_unit: 100.0,
};
