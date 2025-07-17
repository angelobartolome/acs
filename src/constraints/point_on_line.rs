use crate::{constraints::Constraint, geometry::GeometrySystem};

pub struct PointOnLineConstraint {
    point_id: usize,
    line_id: usize,
}

impl PointOnLineConstraint {
    pub fn new(point_id: usize, line_id: usize) -> Self {
        Self { point_id, line_id }
    }
}

impl Constraint for PointOnLineConstraint {
    fn error(&self, geometry: &GeometrySystem) -> f64 {
        if let Some(point) = geometry.get_point(self.point_id) {
            if let Some(line) = geometry.get_line(self.line_id) {
                let line_start = geometry.get_point(line.start).unwrap();
                let line_end = geometry.get_point(line.end).unwrap();
                // Calculate the distance from the point to the line
                let dx = line_end.x - line_start.x;
                let dy = line_end.y - line_start.y;
                let line_length = (dx * dx + dy * dy).sqrt();
                if line_length > 1e-12 {
                    // Line equation: Ax + By + C = 0
                    let a = dy;
                    let b = -dx;
                    let c = dx * line_start.y - dy * line_start.x;
                    return (a * point.x + b * point.y + c).abs() / line_length;
                }
            }
        }
        0.0
    }

    fn jacobian(&self, geometry: &GeometrySystem) -> Vec<(usize, f64, f64)> {
        let mut jacobian = Vec::new();

        if let Some(point) = geometry.get_point(self.point_id) {
            if let Some(line) = geometry.get_line(self.line_id) {
                let line_start = geometry.get_point(line.start).unwrap();
                let line_end = geometry.get_point(line.end).unwrap();

                let dx = line_end.x - line_start.x;
                let dy = line_end.y - line_start.y;
                let line_length = (dx * dx + dy * dy).sqrt();

                if line_length > 1e-12 {
                    // Line equation: Ax + By + C = 0
                    let a = dy;
                    let b = -dx;
                    let c = dx * line_start.y - dy * line_start.x;

                    let distance_value = a * point.x + b * point.y + c;
                    let distance = distance_value.abs() / line_length;

                    if distance > 1e-12 {
                        let sign = if distance_value >= 0.0 { 1.0 } else { -1.0 };

                        // Partial derivatives with respect to the point coordinates
                        let dx_point = sign * a / line_length;
                        let dy_point = sign * b / line_length;
                        jacobian.push((self.point_id, dx_point, dy_point));

                        // Partial derivatives with respect to line start point
                        let dx_start = sign * (point.y - line_end.y) / line_length;
                        let dy_start = sign * (line_end.x - point.x) / line_length;
                        jacobian.push((line.start, dx_start, dy_start));

                        // Partial derivatives with respect to line end point
                        let dx_end = sign * (line_start.y - point.y) / line_length;
                        let dy_end = sign * (point.x - line_start.x) / line_length;
                        jacobian.push((line.end, dx_end, dy_end));
                    } else {
                        // When the point is very close to the line, use simplified gradients
                        jacobian.push((self.point_id, a / line_length, b / line_length));
                        jacobian.push((line.start, -a / line_length, -b / line_length));
                        jacobian.push((line.end, 0.0, 0.0));
                    }
                }
            }
        }
        jacobian
    }

    fn get_dependent_points(&self) -> Vec<usize> {
        // Since we can't access geometry here, we'll return just the point
        // The solver will need to determine line dependencies from the jacobian
        vec![self.point_id]
    }

    fn constraint_type(&self) -> &'static str {
        "PointOnLine"
    }
}
