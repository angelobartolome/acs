use crate::geometry::GeometrySystem;

pub trait Constraint {
    fn error(&self, geometry: &GeometrySystem) -> f64;
    fn jacobian(&self, geometry: &GeometrySystem) -> Vec<(usize, f64, f64)>; // (point_id, dx, dy)
    fn get_dependent_points(&self) -> Vec<usize>;
    fn constraint_type(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Vertical(usize),          // Line ID
    Horizontal(usize),        // Line ID
    Parallel(usize, usize),   // Line IDs
    Coincident(usize, usize), // Point IDs
    EqualX(usize, f64),       // Point ID and X value
    EqualY(usize, f64),       // Point ID and Y value
}

pub fn create_constraint(
    constraint_type: ConstraintType,
    geometry: &GeometrySystem,
) -> Result<Box<dyn Constraint>, String> {
    match constraint_type {
        ConstraintType::Vertical(line_id) => {
            if !geometry.get_all_lines().contains_key(&line_id) {
                return Err("Invalid line ID".to_string());
            }
            Ok(Box::new(
                crate::constraints::vertical::VerticalConstraint::new(line_id),
            ))
        }
        ConstraintType::Horizontal(line_id) => {
            if !geometry.get_all_lines().contains_key(&line_id) {
                return Err("Invalid line ID".to_string());
            }
            Ok(Box::new(
                crate::constraints::horizontal::HorizontalConstraint::new(line_id),
            ))
        }
        ConstraintType::Parallel(line1_id, line2_id) => {
            if !geometry.get_all_lines().contains_key(&line1_id)
                || !geometry.get_all_lines().contains_key(&line2_id)
            {
                return Err("Invalid line IDs".to_string());
            }
            Ok(Box::new(
                crate::constraints::parallel::ParallelConstraint::new(line1_id, line2_id),
            ))
        }
        ConstraintType::Coincident(point_a_id, point_b_id) => {
            if !geometry.get_all_points().contains_key(&point_a_id)
                || !geometry.get_all_points().contains_key(&point_b_id)
            {
                return Err("Invalid point IDs".to_string());
            }
            Ok(Box::new(
                crate::constraints::coincident::CoincidentConstraint::new(point_a_id, point_b_id),
            ))
        }
        ConstraintType::EqualX(point_id, x_value) => {
            if !geometry.get_all_points().contains_key(&point_id) {
                return Err("Invalid point ID".to_string());
            }
            Ok(Box::new(
                crate::constraints::equal_x::EqualXConstraint::new(point_id, x_value),
            ))
        }
        ConstraintType::EqualY(point_id, y_value) => {
            if !geometry.get_all_points().contains_key(&point_id) {
                return Err("Invalid point ID".to_string());
            }
            Ok(Box::new(
                crate::constraints::equal_y::EqualYConstraint::new(point_id, y_value),
            ))
        }
    }
}
