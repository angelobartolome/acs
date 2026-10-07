//! [`PointOnCircleConstraint`]: constrains a point to lie on the
//! circumference of a circle.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a point to lie on the circumference of a circle.
///
/// Entities:
///   - `point_id`        – the free point (x, y)
///   - `circle_center_id`– the circle's center point (cx, cy)
///   - `circle_id`       – the circle entity (radius r at param index 0)
///
/// Residual: (px − cx)² + (py − cy)² − r² = 0
pub struct PointOnCircleConstraint {
    /// The free point (x, y).
    pub point_id: String,
    /// The circle's center point (cx, cy).
    pub circle_center_id: String,
    /// The circle entity (radius r at param index 0).
    pub circle_id: String,
}

impl PointOnCircleConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(point_id: String, circle_center_id: String, circle_id: String) -> Self {
        Self {
            point_id,
            circle_center_id,
            circle_id,
        }
    }
}

impl Constraint for PointOnCircleConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [&xy(&self.point_id)[..], &xy(&self.circle_center_id), &[Var::Radius(&self.circle_id)]].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, cx, cy, r]
        let (dx, dy, rad) = (x[0] - x[2], x[1] - x[3], x[4]);
        r[0] = dx * dx + dy * dy - rad * rad;
        set_row(j, 0, &[2.0 * dx, 2.0 * dy, -2.0 * dx, -2.0 * dy, -2.0 * rad]);
    }
}
