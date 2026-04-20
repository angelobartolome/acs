use nalgebra::{DMatrix, DVector};

use crate::ParameterManager;

pub trait Constraint {
    fn num_residuals(&self) -> usize;
    fn residual(&self, _param_manager: &ParameterManager) -> DVector<f64>;
    fn jacobian(&self, _param_manager: &ParameterManager) -> DMatrix<f64>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    // ── Existing ──────────────────────────────────────────────────────────────
    Vertical(String, String),                 // p1_id, p2_id
    Horizontal(String, String),               // p1_id, p2_id
    Parallel(String, String, String, String), // L1P1, L1P2, L2P1, L2P2
    EqualX(String, f64),                      // point_id, x-value
    EqualY(String, f64),                      // point_id, y-value
    Coincident(String, String),               // p1_id, p2_id
    PointOnLine(String, String, String),      // point_id, line_pa_id, line_pb_id
    EqualRadius(String, String),              // circle1_id, circle2_id

    // ── New ───────────────────────────────────────────────────────────────────
    /// Force two lines to be perpendicular (dot product of directions = 0).
    /// (L1P1, L1P2, L2P1, L2P2)
    Perpendicular(String, String, String, String),

    /// Force a circle/arc radius to a fixed value.
    /// (circle_id, target_radius)
    FixedRadius(String, f64),

    /// Force a point to lie on the circumference of a circle.
    /// (point_id, circle_center_point_id, circle_id)
    PointOnCircle(String, String, String),

    /// External tangency between two circles (dist(centers) = r1 + r2).
    /// (c1_center_point_id, c1_id, c2_center_point_id, c2_id)
    Tangent(String, String, String, String),

    /// Tangency between a line and a circle (dist(center, line) = r).
    /// (line_pa_id, line_pb_id, circle_center_point_id, circle_id)
    TangentLineCircle(String, String, String, String),

    /// Two circles share the same center.
    /// (center1_point_id, center2_point_id)
    Concentric(String, String),

    /// Fixed Euclidean distance between two points.
    /// (p1_id, p2_id, distance)
    DistancePointPoint(String, String, f64),

    /// Fixed perpendicular distance from a point to an infinite line.
    /// (point_id, line_pa_id, line_pb_id, distance)
    DistancePointLine(String, String, String, f64),

    /// Fixed directed angle (in radians) from line L1 to line L2.
    /// (L1P1, L1P2, L2P1, L2P2, angle_radians)
    Angle(String, String, String, String, f64),

    /// Force a point to be the midpoint of a segment.
    /// (midpoint_id, endpoint_a_id, endpoint_b_id)
    Midpoint(String, String, String),

    /// Force two line segments to have equal length.
    /// (L1P1, L1P2, L2P1, L2P2)
    EqualLength(String, String, String, String),

    /// Force two points to be symmetric about a line (axis of symmetry).
    /// (p_id, q_id, axis_pa_id, axis_pb_id)
    Symmetric(String, String, String, String),

    /// Midpoint of segment (l1a, l1b) lies on infinite line (l2a, l2b).
    MidpointOfLineOnLine(String, String, String, String),
}

impl ConstraintType {
    pub fn entity_ids(&self) -> Vec<&str> {
        match self {
            Self::Vertical(a, b) => vec![a, b],
            Self::Horizontal(a, b) => vec![a, b],
            Self::Coincident(a, b) => vec![a, b],
            Self::EqualRadius(a, b) => vec![a, b],
            Self::Concentric(a, b) => vec![a, b],
            Self::EqualX(a, _) => vec![a],
            Self::EqualY(a, _) => vec![a],
            Self::FixedRadius(a, _) => vec![a],
            Self::DistancePointPoint(a, b, _) => vec![a, b],
            Self::PointOnLine(a, b, c) => vec![a, b, c],
            Self::Midpoint(a, b, c) => vec![a, b, c],
            Self::PointOnCircle(a, b, c) => vec![a, b, c],
            Self::DistancePointLine(a, b, c, _) => vec![a, b, c],
            Self::Parallel(a, b, c, d) => vec![a, b, c, d],
            Self::Perpendicular(a, b, c, d) => vec![a, b, c, d],
            Self::Tangent(a, b, c, d) => vec![a, b, c, d],
            Self::TangentLineCircle(a, b, c, d) => vec![a, b, c, d],
            Self::Angle(a, b, c, d, _) => vec![a, b, c, d],
            Self::EqualLength(a, b, c, d) => vec![a, b, c, d],
            Self::Symmetric(a, b, c, d) => vec![a, b, c, d],
            Self::MidpointOfLineOnLine(a, b, c, d) => vec![a, b, c, d],
        }
    }
}

pub fn create_constraint(constraint_type: ConstraintType) -> Result<Box<dyn Constraint>, String> {
    match constraint_type {
        // ── Existing ──────────────────────────────────────────────────────────
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
        ConstraintType::PointOnLine(p1, pa, pb) => Ok(Box::new(
            crate::constraints::point_on_line::PointOnLineConstraint::new(p1, pa, pb),
        )),
        ConstraintType::EqualRadius(c1, c2) => Ok(Box::new(
            crate::constraints::equal_radius::EqualRadiusConstraint::new(c1, c2),
        )),

        // ── New ───────────────────────────────────────────────────────────────
        ConstraintType::Perpendicular(p1, p2, p3, p4) => Ok(Box::new(
            crate::constraints::perpendicular::PerpendicularConstraint::new(p1, p2, p3, p4),
        )),
        ConstraintType::FixedRadius(c, r) => Ok(Box::new(
            crate::constraints::fixed_radius::FixedRadiusConstraint::new(c, r),
        )),
        ConstraintType::PointOnCircle(pt, center, circle) => Ok(Box::new(
            crate::constraints::point_on_circle::PointOnCircleConstraint::new(pt, center, circle),
        )),
        ConstraintType::Tangent(c1_center, c1, c2_center, c2) => Ok(Box::new(
            crate::constraints::tangent::TangentConstraint::new(c1_center, c1, c2_center, c2),
        )),
        ConstraintType::TangentLineCircle(pa, pb, c_center, c) => Ok(Box::new(
            crate::constraints::tangent_line_circle::TangentLineCircleConstraint::new(
                pa, pb, c_center, c,
            ),
        )),
        ConstraintType::Concentric(center1, center2) => Ok(Box::new(
            crate::constraints::concentric::ConcentricConstraint::new(center1, center2),
        )),
        ConstraintType::DistancePointPoint(p1, p2, d) => Ok(Box::new(
            crate::constraints::distance_point_point::DistancePointPointConstraint::new(p1, p2, d),
        )),
        ConstraintType::DistancePointLine(pt, pa, pb, d) => Ok(Box::new(
            crate::constraints::distance_point_line::DistancePointLineConstraint::new(
                pt, pa, pb, d,
            ),
        )),
        ConstraintType::Angle(p1, p2, p3, p4, theta) => Ok(Box::new(
            crate::constraints::angle::AngleConstraint::new(p1, p2, p3, p4, theta),
        )),
        ConstraintType::Midpoint(m, a, b) => Ok(Box::new(
            crate::constraints::midpoint::MidpointConstraint::new(m, a, b),
        )),
        ConstraintType::EqualLength(p1, p2, p3, p4) => Ok(Box::new(
            crate::constraints::equal_length::EqualLengthConstraint::new(p1, p2, p3, p4),
        )),
        ConstraintType::Symmetric(p, q, pa, pb) => Ok(Box::new(
            crate::constraints::symmetric::SymmetricConstraint::new(p, q, pa, pb),
        )),
        ConstraintType::MidpointOfLineOnLine(a, b, c, d) => Ok(Box::new(
            crate::constraints::midpoint_line_on_line::MidpointOfLineOnLineConstraint::new(
                a, b, c, d,
            ),
        )),
    }
}
