/// A struct that wraps a value and applies zoom to all calculations
#[derive(Debug, Clone, Copy)]
pub struct Zoomed {
    value: f32,
}

impl Zoomed {
    /// Create a new Zoomed value from an original value and zoom factor
    pub fn new(original: f32, zoom: f32) -> Self {
        Self {
            value: original * zoom,
        }
    }

    /// Convert back to f32
    pub fn to_f32(self) -> f32 {
        self.value
    }
}

// Implement arithmetic operations for Zoomed
impl std::ops::Add for Zoomed {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
        }
    }
}

impl std::ops::Sub for Zoomed {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            value: self.value - other.value,
        }
    }
}

impl std::ops::Mul<f32> for Zoomed {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            value: self.value * scalar,
        }
    }
}

impl std::ops::Div<f32> for Zoomed {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            value: self.value / scalar,
        }
    }
}
