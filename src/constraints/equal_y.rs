use crate::{constraints::Constraint, geometry::GeometrySystem};

pub struct EqualYConstraint {
    point_id: usize,
    value: f64,
}

impl EqualYConstraint {
    pub fn new(point_id: usize, value: f64) -> Self {
        Self { point_id, value }
    }
}

impl Constraint for EqualYConstraint {
    fn error(&self, geometry: &GeometrySystem) -> f64 {
        if let Some(point) = geometry.get_point(self.point_id) {
            return point.y - self.value;
        }
        0.0
    }

    fn jacobian(&self, geometry: &GeometrySystem) -> Vec<(usize, f64, f64)> {
        let mut jacobian = Vec::new();

        if let Some(_point) = geometry.get_point(self.point_id) {
            jacobian.push((self.point_id, 0.0, 1.0));
        }

        jacobian
    }

    fn get_dependent_points(&self) -> Vec<usize> {
        vec![self.point_id]
    }

    fn constraint_type(&self) -> &'static str {
        "EqualY"
    }
}
