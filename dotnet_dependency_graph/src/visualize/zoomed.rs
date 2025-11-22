/// A struct that wraps a value and applies zoom to all calculations
#[derive(Debug, Clone, Copy)]
pub struct Zoomed<T> {
    value: T,
}

impl<T> Zoomed<T> {
    /// Create a new Zoomed value from an original value and zoom factor
    pub fn new(original: T, zoom: f32) -> Self
    where
        T: std::ops::Mul<f32, Output = T>,
    {
        Self {
            value: original * zoom,
        }
    }

    /// Convert back to value
    pub fn into_value(self) -> T {
        self.value
    }
}

// Implement arithmetic operations for Zoomed
impl<T> std::ops::Add for Zoomed<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
        }
    }
}

impl<T> std::ops::Sub for Zoomed<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            value: self.value - other.value,
        }
    }
}

impl<T> std::ops::Mul<f32> for Zoomed<T>
where
    T: std::ops::Mul<f32, Output = T>,
{
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            value: self.value * scalar,
        }
    }
}

impl<T> std::ops::Div<f32> for Zoomed<T>
where
    T: std::ops::Div<f32, Output = T>,
{
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            value: self.value / scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_zoom_factor() {
        let zoomed = Zoomed::new(10.0, 2.0);
        assert_eq!(zoomed.into_value(), 20.0);
    }

    #[test]
    fn test_new_with_zoom_less_than_one() {
        let zoomed = Zoomed::new(10.0, 0.5);
        assert_eq!(zoomed.into_value(), 5.0);
    }

    #[test]
    fn test_new_with_no_zoom() {
        let zoomed = Zoomed::new(10.0, 1.0);
        assert_eq!(zoomed.into_value(), 10.0);
    }

    #[test]
    fn test_new_with_negative_value() {
        let zoomed = Zoomed::new(-10.0, 2.0);
        assert_eq!(zoomed.into_value(), -20.0);
    }

    #[test]
    fn test_add() {
        let z1 = Zoomed::new(10.0, 2.0);
        let z2 = Zoomed::new(5.0, 2.0);
        let result = z1 + z2;
        assert_eq!(result.into_value(), 30.0);
    }

    #[test]
    fn test_add_with_different_zoom_factors() {
        // Note: This tests that we can add Zoomed values even if they
        // were created with different zoom factors (they just add their values)
        let z1 = Zoomed::new(10.0, 2.0);
        let z2 = Zoomed::new(10.0, 1.0);
        let result = z1 + z2;
        assert_eq!(result.into_value(), 30.0);
    }

    #[test]
    fn test_sub() {
        let z1 = Zoomed::new(10.0, 2.0);
        let z2 = Zoomed::new(3.0, 2.0);
        let result = z1 - z2;
        assert_eq!(result.into_value(), 14.0);
    }

    #[test]
    fn test_sub_negative_result() {
        let z1 = Zoomed::new(3.0, 2.0);
        let z2 = Zoomed::new(10.0, 2.0);
        let result = z1 - z2;
        assert_eq!(result.into_value(), -14.0);
    }

    #[test]
    fn test_mul_scalar() {
        let zoomed = Zoomed::new(10.0, 2.0);
        let result = zoomed * 3.0;
        assert_eq!(result.into_value(), 60.0);
    }

    #[test]
    fn test_mul_scalar_less_than_one() {
        let zoomed = Zoomed::new(10.0, 2.0);
        let result = zoomed * 0.5;
        assert_eq!(result.into_value(), 10.0);
    }

    #[test]
    fn test_mul_scalar_negative() {
        let zoomed = Zoomed::new(10.0, 2.0);
        let result = zoomed * -2.0;
        assert_eq!(result.into_value(), -40.0);
    }

    #[test]
    fn test_div_scalar() {
        let zoomed = Zoomed::new(10.0, 2.0);
        let result = zoomed / 4.0;
        assert_eq!(result.into_value(), 5.0);
    }

    #[test]
    fn test_div_scalar_less_than_one() {
        let zoomed = Zoomed::new(10.0, 2.0);
        let result = zoomed / 0.5;
        assert_eq!(result.into_value(), 40.0);
    }

    #[test]
    fn test_chained_operations() {
        let z1 = Zoomed::new(10.0, 2.0);
        let z2 = Zoomed::new(5.0, 2.0);
        let result = (z1 + z2) * 2.0 / 3.0;
        assert_eq!(result.into_value(), 20.0);
    }

    #[test]
    fn test_zero_values() {
        let zoomed = Zoomed::new(0.0, 2.0);
        assert_eq!(zoomed.into_value(), 0.0);
    }

    #[test]
    fn test_zero_zoom() {
        let zoomed = Zoomed::new(10.0, 0.0);
        assert_eq!(zoomed.into_value(), 0.0);
    }

    #[test]
    fn test_clone() {
        let z1 = Zoomed::new(10.0, 2.0);
        #[allow(clippy::clone_on_copy)]
        let z2 = z1.clone();
        assert_eq!(z1.into_value(), z2.into_value());
    }

    #[test]
    fn test_copy() {
        let z1 = Zoomed::new(10.0, 2.0);
        let z2 = z1;
        assert_eq!(z1.into_value(), z2.into_value());
    }
}
