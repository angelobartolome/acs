use crate::{constraints::Constraint, geometry::GeometrySystem};

pub struct VerticalConstraint {
    line_id: usize,
}

impl VerticalConstraint {
    pub fn new(line_id: usize) -> Self {
        Self { line_id }
    }
}

impl Constraint for VerticalConstraint {
    fn error(&self, geometry: &GeometrySystem) -> f64 {
        if let Some(line) = geometry.get_line(self.line_id) {
            if let (Some(start), Some(end)) =
                (geometry.get_point(line.start), geometry.get_point(line.end))
            {
                return end.x - start.x; // Error is the x-difference
            }
        }
        0.0
    }

    fn jacobian(&self, geometry: &GeometrySystem) -> Vec<(usize, f64, f64)> {
        let mut jacobian = Vec::new();

        if let Some(line) = geometry.get_line(self.line_id) {
            // Partial derivative with respect to start point: -1 in x, 0 in y
            jacobian.push((line.start, -1.0, 0.0));
            // Partial derivative with respect to end point: 1 in x, 0 in y
            jacobian.push((line.end, 1.0, 0.0));
        }

        jacobian
    }

    fn get_dependent_points(&self) -> Vec<usize> {
        vec![self.line_id] // This would need to be resolved to actual point IDs
    }

    fn constraint_type(&self) -> &'static str {
        "Vertical"
    }
}
