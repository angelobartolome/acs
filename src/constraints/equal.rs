use nalgebra::DMatrix;

use crate::constraints::{Constraint, Operand, Var, operand_values};

/// Constrains two scalars to be equal: param1 = param2. Each may be a
/// constant or an entity property (such as a Circle's radius or a Point's x).
///
/// Residual: param1 − param2. Partials: +1, −1 for the operands that are
/// variables.
pub struct EqualConstraint {
    pub param1: Operand,
    pub param2: Operand,
}

impl EqualConstraint {
    pub fn new(param1: Operand, param2: Operand) -> Self {
        Self { param1, param2 }
    }

    fn operands(&self) -> [&Operand; 2] {
        [&self.param1, &self.param2]
    }
}

impl Constraint for EqualConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        self.operands().iter().filter_map(|o| o.var()).collect()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let v = operand_values(&self.operands(), x);
        r[0] = v[0].0 - v[1].0;
        for ((_, col), sign) in v.iter().zip([1.0, -1.0]) {
            if let Some(k) = col {
                j[(0, *k)] = sign;
            }
        }
    }
}
