use crate::constraints::base::Constraint;
use crate::geometry::GeometrySystem;

pub struct ParallelConstraint {
    line_a_id: usize,
    line_b_id: usize,
}

impl ParallelConstraint {
    pub fn new(line_a_id: usize, line_b_id: usize) -> Self {
        Self {
            line_a_id,
            line_b_id,
        }
    }
}

impl Constraint for ParallelConstraint {
    fn error(&self, geometry: &GeometrySystem) -> f64 {
        if let (Some(line_a), Some(line_b)) = (
            geometry.get_line(self.line_a_id),
            geometry.get_line(self.line_b_id),
        ) {
            if let (Some(start_a), Some(end_a), Some(start_b), Some(end_b)) = (
                geometry.get_point(line_a.start),
                geometry.get_point(line_a.end),
                geometry.get_point(line_b.start),
                geometry.get_point(line_b.end),
            ) {
                // Check if the lines are parallel
                let slope_a = (end_a.y - start_a.y) / (end_a.x - start_a.x);
                let slope_b = (end_b.y - start_b.y) / (end_b.x - start_b.x);
                return if (slope_a - slope_b).abs() < 1e-6 {
                    0.0 // No error if parallel
                } else {
                    (slope_a - slope_b).abs() // Error is the difference in slopes
                };
            }
        }
        0.0
    }

    fn jacobian(&self, geometry: &GeometrySystem) -> Vec<(usize, f64, f64)> {
        let mut jacobian = Vec::new();

        if let Some(line_a) = geometry.get_line(self.line_a_id) {
            // Partial derivative with respect to start point of line A: 0 in x, -1 in y
            jacobian.push((line_a.start, 0.0, -1.0));
            // Partial derivative with respect to end point of line A: 0 in x, 1 in y
            jacobian.push((line_a.end, 0.0, 1.0));
        }

        if let Some(line_b) = geometry.get_line(self.line_b_id) {
            // Partial derivative with respect to start point of line B: 0 in x, -1 in y
            jacobian.push((line_b.start, 0.0, -1.0));
            // Partial derivative with respect to end point of line B: 0 in x, 1 in y
            jacobian.push((line_b.end, 0.0, 1.0));
        }

        jacobian
    }

    fn get_dependent_points(&self) -> Vec<usize> {
        vec![self.line_a_id, self.line_b_id]
    }

    fn constraint_type(&self) -> &'static str {
        "Parallel"
    }
}
