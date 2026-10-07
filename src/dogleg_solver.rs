//! The numeric core: a Dog-Leg trust-region least-squares solver
//! ([`ParametricDogLegSolver`]) over a plain parameter vector, with the
//! convergence tolerance the whole crate uses (`TOLF`, 1e-10).
//!
//! It solves [non-linear least squares](https://en.wikipedia.org/wiki/Non-linear_least_squares)
//! by [Powell's dog leg method](https://en.wikipedia.org/wiki/Powell%27s_dog_leg_method): each
//! step mixes the [Gauss–Newton](https://en.wikipedia.org/wiki/Gauss%E2%80%93Newton_algorithm)
//! step and the [steepest descent](https://en.wikipedia.org/wiki/Gradient_descent) step, kept
//! within a [trust region](https://en.wikipedia.org/wiki/Trust_region) that grows or shrinks with
//! how well the linear model predicted the last step. Convergence is
//! measured by the [L-inf norm](https://en.wikipedia.org/wiki/Uniform_norm) of the residuals.

#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::SolverResult;

/// Residual tolerance (L-inf): a constraint, component or sketch is solved
/// when every residual is within it.
pub(crate) const TOLF: f64 = 1e-10;

/// Residuals and their Jacobian with respect to the parameter vector.
pub(crate) type System = (DVector<f64>, DMatrix<f64>);

/// Dog-Leg trust-region least-squares core ([Powell's dog leg
/// method](https://en.wikipedia.org/wiki/Powell%27s_dog_leg_method)). Knows nothing about sketches:
/// it moves a parameter vector `x` to drive `eval(x)`'s residuals to zero.
pub struct ParametricDogLegSolver {
    max_iterations: usize,
    /// Convergence tolerance on the L-infinity norm of the residual vector.
    tolf: f64,
    /// Convergence tolerance on the L-infinity norm of the gradient.
    tolg: f64,
    /// Convergence tolerance on the trust radius relative to the parameter norm.
    tolx: f64,
}

impl Default for ParametricDogLegSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ParametricDogLegSolver {
    /// A solver with the default limits: 100 iterations, residual tolerance
    /// 1e-10.
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            tolf: TOLF,
            tolg: 1e-80,
            tolx: 1e-80,
        }
    }

    /// Caps the iterations of one solve (at least 1).
    pub fn set_max_iterations(&mut self, max_iterations: usize) {
        self.max_iterations = max_iterations.max(1);
    }

    pub(crate) fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    /// Minimizes ‖eval(x)‖² starting from `x`, leaving the result in `x`.
    /// `eval` returns the residuals and their Jacobian with respect to `x`.
    pub(crate) fn solve_dl(
        &self,
        x: &mut DVector<f64>,
        eval: &mut dyn FnMut(&DVector<f64>) -> System,
    ) -> SolverResult {
        let (mut fx, mut jac) = eval(x);
        if x.is_empty() {
            // Nothing to move: solved only if it already is.
            let error = fx.norm();
            return if fx.amax() <= self.tolf {
                SolverResult::Converged {
                    iterations: 0,
                    final_error: error,
                    initial_error: error,
                }
            } else {
                SolverResult::MaxIterationsReached {
                    iterations: 0,
                    final_error: error,
                    initial_error: error,
                }
            };
        }

        let mut err = 0.5 * fx.norm_squared();
        let initial_error = fx.norm();

        // Divergence limit is fixed from the initial error (matches GCS.cpp).
        let diverging_lim = 1e6 * err + 1e12;

        let mut g = jac.transpose() * (-&fx);
        let mut g_inf = g.amax();
        let mut fx_inf = fx.amax();

        let mut delta = 0.1_f64;
        let mut nu = 2.0_f64;
        let mut reduce = 0i32;
        let mut iter = 0usize;
        let mut stop = 0u8;

        while stop == 0 {
            // --- Convergence / divergence checks (matches GCS::solve_DL order) ---
            if fx_inf <= self.tolf {
                stop = 1; // exact solution found
                break;
            }
            if g_inf <= self.tolg {
                stop = 2; // gradient vanished
                break;
            }
            let x_norm = x.norm();
            if delta <= self.tolx * (self.tolx + x_norm) {
                stop = 2; // trust radius collapsed
                break;
            }
            if iter >= self.max_iterations {
                stop = 4;
                break;
            }
            if err > diverging_lim || err.is_nan() {
                stop = 6; // diverging or NaN
                break;
            }

            // --- Cauchy (steepest-descent) step: h_sd = alpha * g ---
            // When J*g underflows (near-zero gradient at near-solution), fall back to h_sd = 0
            // so the dogleg path degrades to a scaled GN step instead of stopping early.
            let jg_norm_sq = (&jac * &g).norm_squared();
            let (alpha, h_sd) = if jg_norm_sq > 1e-300 {
                let a = g.norm_squared() / jg_norm_sq;
                (a, a * &g)
            } else {
                (0.0, DVector::zeros(g.len()))
            };

            // --- Gauss-Newton step: solve J h = -f ---
            let h_gn = match self.compute_gn_step(&jac, &fx) {
                Some(h) => h,
                None => {
                    stop = 6;
                    break;
                }
            };

            // Bail out if the linear solve produced garbage.
            let rel_error = (&jac * &h_gn + &fx).norm() / fx.norm();
            if rel_error > 1e15 {
                break;
            }

            // --- Dogleg step selection ---
            let h_dl = if h_gn.norm() < delta {
                // GN step fits inside the trust region.
                if h_gn.norm() <= self.tolx * (self.tolx + x_norm) {
                    stop = 5; // step negligible relative to parameters
                    break;
                }
                h_gn.clone()
            } else if alpha * g.norm() >= delta {
                // Cauchy step already reaches the trust region boundary.
                (delta / (alpha * g.norm())) * &h_sd
            } else {
                // Interpolate along the dogleg path between h_sd and h_gn.
                let b = &h_gn - &h_sd;
                let bb = b.norm_squared();
                let gb = h_sd.dot(&b).abs();
                // c = delta^2 - ||h_sd||^2, rewritten for numerical stability
                let c = (delta + h_sd.norm()) * (delta - h_sd.norm());

                // Two-case formula avoids catastrophic cancellation when gb ≈ 0.
                let beta = if gb > 0.0 {
                    c / (gb + (gb * gb + c * bb).sqrt())
                } else if bb > 1e-300 {
                    ((gb * gb + c * bb).sqrt() - gb) / bb
                } else {
                    1.0 // h_gn ≈ h_sd; the whole segment collapses to a point
                };
                &h_sd + beta * &b
            };

            // --- Apply trial step ---
            let x_new = &*x + &h_dl;
            let (fx_new, jac_new) = eval(&x_new);
            let err_new = 0.5 * fx_new.norm_squared();

            // Predicted reduction in the linear model vs actual reduction.
            let dl_model = err - 0.5 * (&fx + &jac * &h_dl).norm_squared();
            let df_actual = err - err_new;

            // Accept step only if both actual and predicted reductions are positive.
            let rho = if df_actual > 0.0 && dl_model > 0.0 {
                *x = x_new;
                fx = fx_new;
                jac = jac_new;
                err = err_new;
                g = jac.transpose() * (-&fx);
                g_inf = g.amax();
                fx_inf = fx.amax();
                dl_model / df_actual
            } else {
                // Reject: x, fx and jac stay at the previous point.
                -1.0
            };

            // --- Trust-region update (matches GCS::solve_DL nu/reduce logic) ---
            if (rho - 1.0).abs() < 0.2 && h_dl.norm() > delta / 3.0 && reduce <= 0 {
                delta *= 3.0;
                nu = 2.0;
                reduce = 0;
            } else if rho < 0.25 {
                delta /= nu;
                nu *= 2.0; // exponential backoff on repeated failures
                reduce = 2;
            } else {
                reduce -= 1;
            }

            iter += 1;
        }

        let final_error = fx.norm();
        // Only a residual within tolerance counts as solved. Stops 2 and 5
        // (vanished gradient, collapsed trust region, negligible step) also
        // occur when stuck in a local minimum away from any solution.
        let solved = stop == 1 || (matches!(stop, 2 | 5) && fx.amax() <= self.tolf);
        if solved {
            SolverResult::Converged {
                iterations: iter,
                final_error,
                initial_error,
            }
        } else {
            SolverResult::MaxIterationsReached {
                iterations: iter,
                final_error,
                initial_error,
            }
        }
    }

    /// Computes the Gauss-Newton step.
    /// Returns `None` only if every fallback fails.
    fn compute_gn_step(&self, jac: &DMatrix<f64>, fx: &DVector<f64>) -> Option<DVector<f64>> {
        let neg_fx = -fx;
        let svd_fallback = || jac.clone().svd(true, true).solve(&neg_fx, 1e-12).ok();

        // Direct solve J h = -f; avoids forming the normal equations J^T J.
        // nalgebra's full_piv_lu panics on non-square matrices, so fall back to SVD
        // for rectangular systems (under/overdetermined).
        if jac.nrows() == jac.ncols() {
            jac.clone().full_piv_lu().solve(&neg_fx).or_else(svd_fallback)
        } else {
            svd_fallback()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Intersect the unit circle with the line y = x: no sketch involved.
    #[test]
    fn solves_a_plain_nonlinear_system() {
        let mut x = DVector::from_vec(vec![2.0, 0.5]);
        let result = ParametricDogLegSolver::new().solve_dl(&mut x, &mut |x| {
            let (a, b) = (x[0], x[1]);
            let r = DVector::from_vec(vec![a * a + b * b - 1.0, a - b]);
            let j = DMatrix::from_row_slice(2, 2, &[2.0 * a, 2.0 * b, 1.0, -1.0]);
            (r, j)
        });
        assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
        let h = std::f64::consts::FRAC_1_SQRT_2;
        assert!((x[0] - h).abs() < 1e-9 && (x[1] - h).abs() < 1e-9, "{x}");
    }

    #[test]
    fn nothing_to_move_is_solved_only_if_already_satisfied() {
        let dl = ParametricDogLegSolver::new();
        let mut empty = DVector::zeros(0);
        let zero = |_: &DVector<f64>| (DVector::from_vec(vec![0.0]), DMatrix::zeros(1, 0));
        let off = |_: &DVector<f64>| (DVector::from_vec(vec![1.0]), DMatrix::zeros(1, 0));
        assert!(matches!(dl.solve_dl(&mut empty, &mut { zero }), SolverResult::Converged { .. }));
        assert!(matches!(
            dl.solve_dl(&mut empty, &mut { off }),
            SolverResult::MaxIterationsReached { .. }
        ));
    }
}
