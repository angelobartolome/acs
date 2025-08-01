use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::Point;

pub trait Constraint {
    fn num_residuals(&self) -> usize;
    fn residual(&self, points: &HashMap<String, Point>) -> DVector<f64>;
    fn jacobian(
        &self,
        points: &HashMap<String, Point>,
        id_to_index: &HashMap<String, usize>,
    ) -> DMatrix<f64>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Vertical(String, String),                 // Points IDs
    Horizontal(String, String),               // Points IDs
    Parallel(String, String, String, String), // L1P1, L1P2, L2P1, L2P2
    EqualX(String, f64),                      // Point ID, x-coordinate
    EqualY(String, f64),                      // Point ID, y-coordinate
    Coincident(String, String),               // Point IDs
    PointOnLine(String, String, String),      // Point ID, Line Point A ID, Line Point B ID
}

pub fn create_constraint(constraint_type: ConstraintType) -> Result<Box<dyn Constraint>, String> {
    match constraint_type {
        ConstraintType::Vertical(p1, p2) => Ok(Box::new(
            crate::constraints::vertical::VerticalConstraint::new(p1, p2),
        )),
        ConstraintType::Horizontal(p1, p2) => Ok(Box::new(
            crate::constraints::horizontal::HorizontalConstraint::new(p1, p2),
        )),
        ConstraintType::Parallel(p1, p2, p3, p4) => Ok(Box::new(
            crate::constraints::parallel::ParallelConstraint::new(p1, p2, p3, p4),
        )),
        ConstraintType::EqualX(p1, x) => Ok(Box::new(
            crate::constraints::equal_x::EqualXConstraint::new(p1, x),
        )),
        ConstraintType::EqualY(p1, y) => Ok(Box::new(
            crate::constraints::equal_y::EqualYConstraint::new(p1, y),
        )),
        ConstraintType::Coincident(p1, p2) => Ok(Box::new(
            crate::constraints::coincident::CoincidentConstraint::new(p1, p2),
        )),
        ConstraintType::PointOnLine(p1, p_line_a, p_line_b) => Ok(Box::new(
            crate::constraints::point_on_line::PointOnLineConstraint::new(p1, p_line_a, p_line_b),
        )),
    }
}
