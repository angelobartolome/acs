use crate::constraints::base::Constraint;
use crate::geometry::GeometrySystem;

pub struct HorizontalConstraint {
    line_id: usize,
}

impl HorizontalConstraint {
    pub fn new(line_id: usize) -> Self {
        Self { line_id }
    }
}

impl Constraint for HorizontalConstraint {
    fn error(&self, geometry: &GeometrySystem) -> f64 {
        if let Some(line) = geometry.get_line(self.line_id) {
            if let (Some(start), Some(end)) =
                (geometry.get_point(line.start), geometry.get_point(line.end))
            {
                return end.y - start.y; // Error is the y-difference
            }
        }
        0.0
    }

    fn jacobian(&self, geometry: &GeometrySystem) -> Vec<(usize, f64, f64)> {
        let mut jacobian = Vec::new();

        if let Some(line) = geometry.get_line(self.line_id) {
            // Partial derivative with respect to start point: 0 in x, -1 in y
            jacobian.push((line.start, 0.0, -1.0));
            // Partial derivative with respect to end point: 0 in x, 1 in y
            jacobian.push((line.end, 0.0, 1.0));
        }

        jacobian
    }

    fn get_dependent_points(&self) -> Vec<usize> {
        vec![self.line_id] // This would need to be resolved to actual point IDs
    }

    fn constraint_type(&self) -> &'static str {
        "Horizontal"
    }
}
