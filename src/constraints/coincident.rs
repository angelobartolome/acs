use crate::{constraints::Constraint, geometry::GeometrySystem};

pub struct CoincidentConstraint {
    point_a_id: usize,
    point_b_id: usize,
}

impl CoincidentConstraint {
    pub fn new(point_a_id: usize, point_b_id: usize) -> Self {
        Self {
            point_a_id,
            point_b_id,
        }
    }
}

impl Constraint for CoincidentConstraint {
    fn error(&self, geometry: &GeometrySystem) -> f64 {
        if let Some(point_a) = geometry.get_point(self.point_a_id) {
            if let Some(point_b) = geometry.get_point(self.point_b_id) {
                // Calculate the distance between the two points
                let dx = point_a.x - point_b.x;
                let dy = point_a.y - point_b.y;
                return (dx * dx + dy * dy).sqrt();
            }
        }
        0.0
    }

    fn jacobian(&self, geometry: &GeometrySystem) -> Vec<(usize, f64, f64)> {
        let mut jacobian = Vec::new();

        if let Some(point_a) = geometry.get_point(self.point_a_id) {
            if let Some(point_b) = geometry.get_point(self.point_b_id) {
                let dx = point_a.x - point_b.x;
                let dy = point_a.y - point_b.y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > 1e-12 {
                    // For error = sqrt((xa - xb)² + (ya - yb)²)
                    // d/dxa = (xa - xb) / distance, d/dya = (ya - yb) / distance
                    // d/dxb = -(xa - xb) / distance, d/dyb = -(ya - yb) / distance
                    let dx_norm = dx / distance;
                    let dy_norm = dy / distance;

                    jacobian.push((self.point_a_id, dx_norm, dy_norm));
                    jacobian.push((self.point_b_id, -dx_norm, -dy_norm));
                } else {
                    // When points are very close, the gradient is undefined,
                    // but we can use a small perturbation approach
                    jacobian.push((self.point_a_id, 1.0, 0.0));
                    jacobian.push((self.point_b_id, -1.0, 0.0));
                }
            }
        }
        jacobian
    }

    fn get_dependent_points(&self) -> Vec<usize> {
        vec![self.point_a_id, self.point_b_id]
    }

    fn constraint_type(&self) -> &'static str {
        "Coincident"
    }
}
