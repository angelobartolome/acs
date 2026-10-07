//! [`DifferenceConstraint`]: constrains two scalars to differ by a third.

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Operand, Var, operand_values};

/// Constrains two scalars to differ by a third: param2 − param1 = difference.
/// Each may be a constant or an entity property (such as a Circle's radius);
/// a Linked Offset of an arc or circle ties the two radii to its Parameter
/// this way.
///
/// Residual: param2 − param1 − difference. Partials: −1, +1, −1 for the
/// operands that are variables.
pub struct DifferenceConstraint {
    /// The operand subtracted.
    pub param1: Operand,
    /// The operand subtracted from.
    pub param2: Operand,
    /// Their difference, `param2 − param1`.
    pub difference: Operand,
}

impl DifferenceConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(param1: Operand, param2: Operand, difference: Operand) -> Self {
        Self {
            param1,
            param2,
            difference,
        }
    }

    fn operands(&self) -> [&Operand; 3] {
        [&self.param1, &self.param2, &self.difference]
    }
}

impl Constraint for DifferenceConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        self.operands().iter().filter_map(|o| o.var()).collect()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let v = operand_values(&self.operands(), x);
        r[0] = v[1].0 - v[0].0 - v[2].0;
        for ((_, col), sign) in v.iter().zip([-1.0, 1.0, -1.0]) {
            if let Some(k) = col {
                j[(0, *k)] = sign;
            }
        }
    }
}
