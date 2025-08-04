use nalgebra::{DMatrix, DVector};

use crate::ParameterManager;

pub trait Constraint {
    fn num_residuals(&self) -> usize;
    fn residual(&self, _param_manager: &ParameterManager) -> DVector<f64>;
    fn jacobian(&self, _param_manager: &ParameterManager) -> DMatrix<f64>;
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
    EqualRadius(String, String),              // Circle/Arc IDs
    FixedRadius(String, f64),                 // Circle/Arc ID, radius value
    PointOnCircle(String, String),            // Point ID, Circle ID
    Tangent(String, String),                  // Circle IDs (or Circle-Line)
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
        ConstraintType::EqualRadius(_c1, _c2) => Ok(Box::new(
            crate::constraints::equal_radius::EqualRadiusConstraint::new(_c1, _c2),
        )),
        ConstraintType::FixedRadius(_c1, _radius) => {
            // TODO: Implement FixedRadiusConstraint
            Err("FixedRadius constraint not yet implemented".to_string())
        }
        ConstraintType::PointOnCircle(_p1, _c1) => {
            // TODO: Implement PointOnCircleConstraint
            Err("PointOnCircle constraint not yet implemented".to_string())
        }
        ConstraintType::Tangent(_e1, _e2) => {
            // TODO: Implement TangentConstraint
            Err("Tangent constraint not yet implemented".to_string())
        }
    }
}
