//! A sketch's variables and constraints, partitioned into Components.
//!
//! Built once per solve or analysis from the geometry: the global variable
//! vector, and for each Component its constraints and the *free* variable
//! columns they read (fixed variables never become columns). Solving,
//! writing results back to the geometry and the degrees-of-freedom analysis
//! all run on this one structure, each Component on its own small system.
//!
//! Besides the sketch's constraints, every Arc brings its own *implicit*
//! rules (an [`ArcRulesConstraint`] over its center, endpoints, radius and
//! angles): an arc's endpoints always lie on it. Every EllipticalArc
//! likewise brings an [`EllipticalArcRulesConstraint`]. Implicit rules are real
//! constraints in every respect (they join the arc's Component, hold exactly
//! under soft goals, count in degrees of freedom and diagnosis) except that,
//! having no index, they are never reported Conflicting or Redundant.

use std::collections::{BTreeMap, HashMap, HashSet};

use nalgebra::{DMatrix, DVector, SymmetricEigen};

use crate::component_graph::find_components_of;
use crate::constraints::arc_rules::ArcRulesConstraint;
use crate::constraints::elliptical_arc_rules::EllipticalArcRulesConstraint;
use crate::constraints::{Var, eval_into, reads};
use crate::dogleg_solver::{System, TOLF};
use crate::{
    Constraint, EntityType, GeometrySystem, VarRegistry, ParametricDogLegSolver,
    SolverResult,
};

/// How a sketch constraint takes part in the solve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    /// Held exactly; diagnosed; counts in degrees of freedom.
    Real,
    /// A soft goal (see [`SketchSystem::solve`]), outside the diagnosed system.
    Temporary,
}

/// The sketch's constraints followed by the implicit ones (one per Arc and
/// per EllipticalArc).
/// Indices below `explicit.len()` are the sketch's constraint indices.
struct Constraints<'a> {
    explicit: &'a [Box<dyn Constraint>],
    implicit: Vec<Box<dyn Constraint>>,
}

impl Constraints<'_> {
    fn get(&self, i: usize) -> &dyn Constraint {
        match self.explicit.get(i) {
            Some(c) => c.as_ref(),
            None => self.implicit[i - self.explicit.len()].as_ref(),
        }
    }

    fn is_implicit(&self, i: usize) -> bool {
        i >= self.explicit.len()
    }
}

struct Component {
    /// Indices into [`Constraints`].
    constraints: Vec<usize>,
    /// Free global variable columns read by those constraints, in local order.
    columns: Vec<usize>,
    /// Global column → local column.
    local_of: HashMap<usize, usize>,
    /// The real constraints (non-temporary, implicit arc rules included), in
    /// index order: the diagnosed system that solving holds exactly and that
    /// degrees of freedom, full constraint and diagnosis are about.
    diagnosed: Vec<usize>,
    /// The temporary constraints (soft goals), in sketch order.
    goals: Vec<usize>,
}

/// Constraints found Conflicting or Redundant, as sorted indices into the
/// sketch's constraint list (see `ConstraintSolver::diagnose`; the rule is on
/// `SketchSystem::analyze`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Diagnosis {
    /// Constraints that can't all hold: dependent ones carrying residual in
    /// a Component that didn't solve.
    pub conflicting: Vec<usize>,
    /// Constraints that add nothing: a minimal set, latest first, whose
    /// removal changes neither the solution nor the degrees of freedom.
    pub redundant: Vec<usize>,
}

pub(crate) struct SketchSystem<'a> {
    constraints: Constraints<'a>,
    pm: VarRegistry,
    components: Vec<Component>,
    /// Global columns of the curve parameters temporary constraints own:
    /// outside the diagnosed system, like their owners.
    goal_curve_params: HashSet<usize>,
}

impl<'a> SketchSystem<'a> {
    /// `roles[i]` is how sketch constraint `i` takes part (see [`Role`]).
    pub(crate) fn new(
        geometry: &GeometrySystem,
        constraints: &'a [Box<dyn Constraint>],
        roles: &[Role],
    ) -> Self {
        let pm = build_var_registry(geometry);
        let fixed: Vec<bool> = pm.var_info().iter().map(|p| p.is_fixed).collect();
        let constraints = Constraints {
            explicit: constraints,
            implicit: implicit_arc_rules(geometry),
        };
        let n = constraints.explicit.len();

        let all = n + constraints.implicit.len();
        let components = find_components_of((0..all).map(|i| constraints.get(i)))
            .components
            .into_iter()
            .map(|indices| {
                let mut columns = Vec::new();
                let mut local_of = HashMap::new();
                // Guides are columns too: an ordinary solve moves them.
                for &i in &indices {
                    for p in reads(constraints.get(i)) {
                        let col = p
                            .column(&pm)
                            .unwrap_or_else(|| panic!("constraint references unknown variable {p:?}"));
                        if !fixed[col] && !local_of.contains_key(&col) {
                            local_of.insert(col, columns.len());
                            columns.push(col);
                        }
                    }
                }
                let (goals, diagnosed) = indices
                    .iter()
                    .partition(|&&i| i < n && roles[i] == Role::Temporary);
                Component {
                    constraints: indices,
                    columns,
                    local_of,
                    diagnosed,
                    goals,
                }
            })
            .collect();

        let goal_curve_params = (0..n)
            .filter(|&i| roles[i] == Role::Temporary)
            .flat_map(|i| owned_columns(constraints.get(i), &pm))
            .collect();

        Self {
            constraints,
            pm,
            components,
            goal_curve_params,
        }
    }

    /// Solves every Component that isn't already solved, in place.
    /// Iterations are summed across Components and errors maxed.
    ///
    /// Real constraints are hard and temporary ones are soft goals: per
    /// Component, the real constraints hold exactly (within `TOLF`) and the
    /// goals are met as closely as they allow (see [`solve_with_goals`]).
    /// The result, errors included, is about the real constraints only, so
    /// a goal that can't be met never fails a solve. A Component is skipped
    /// (the pre-solver) only when its real constraints *and* its goals
    /// already hold.
    ///
    /// Guides (a mirror's axis, an array's center or direction) are ordinary
    /// unknowns in every solve, drags included: holding them while a drag
    /// moved a copy made the drag chase a Guide that other constraints (a
    /// polygon's tangency) moved, and the solve blew up.
    pub(crate) fn solve(&mut self, dogleg: &ParametricDogLegSolver) -> SolverResult {
        let mut iterations = 0;
        let mut initial_error = 0.0f64;
        let mut final_error = 0.0f64;
        let mut all_converged = true;

        for comp in &self.components {
            let (r, _) = evaluate(&self.pm, &self.constraints, comp, &comp.constraints);
            if r.amax() <= TOLF {
                continue;
            }

            let result = if comp.goals.is_empty() {
                let mut x = values(&self.pm, comp);
                let pm = &mut self.pm;
                let constraints = &self.constraints;
                let result = dogleg.solve_dl(&mut x, &mut |x| {
                    set_columns(pm, comp, x);
                    evaluate(pm, constraints, comp, &comp.constraints)
                });
                set_columns(&mut self.pm, comp, &x);
                result
            } else {
                solve_with_goals(dogleg, &mut self.pm, &self.constraints, comp)
            };
            let (SolverResult::Converged {
                iterations: it,
                initial_error: ie,
                ..
            }
            | SolverResult::MaxIterationsReached {
                iterations: it,
                initial_error: ie,
                ..
            }) = result;
            iterations += it;
            initial_error = initial_error.max(ie);

            let (r, _) = evaluate(&self.pm, &self.constraints, comp, &comp.diagnosed);
            all_converged &= r.amax() <= TOLF;
            final_error = final_error.max(r.norm());
        }

        if all_converged {
            SolverResult::Converged {
                iterations,
                final_error,
                initial_error,
            }
        } else {
            SolverResult::MaxIterationsReached {
                iterations,
                final_error,
                initial_error,
            }
        }
    }

    /// Copies the current variable values back into the geometry.
    pub(crate) fn write_back(&mut self, geometry: &mut GeometrySystem) -> Result<(), String> {
        for (id, point) in geometry.get_all_points_mut() {
            self.pm.write_entity_values(id, point)?;
        }
        for (id, circle) in geometry.get_all_circles_mut() {
            self.pm.write_entity_values(id, circle)?;
        }
        for (id, arc) in geometry.get_all_arcs_mut() {
            self.pm.write_entity_values(id, arc)?;
        }
        for (id, ellipse) in geometry.get_all_ellipses_mut() {
            self.pm.write_entity_values(id, ellipse)?;
        }
        for (id, arc) in geometry.get_all_elliptical_arcs_mut() {
            self.pm.write_entity_values(id, arc)?;
        }
        for (id, param) in geometry.get_all_curve_params_mut() {
            self.pm.write_entity_values(id, param)?;
        }
        Ok(())
    }

    /// The degrees-of-freedom analysis and the diagnosis at the current
    /// variables, which should be the solved position. One pass per
    /// Component over the Jacobian J of its diagnosed constraints: one
    /// [eigen-decomposition](https://en.wikipedia.org/wiki/Eigendecomposition_of_a_matrix)
    /// of JᵀJ gives its rank and null space (degrees of freedom, full
    /// constraint), and one of JJᵀ gives the left null space (diagnosis); the
    /// Redundant walk takes the rank of J with each candidate dropped.
    ///
    /// **Degrees of freedom**: free variables minus the
    /// [rank](https://en.wikipedia.org/wiki/Rank_(linear_algebra)) of each
    /// Component's [Jacobian](https://en.wikipedia.org/wiki/Jacobian_matrix_and_determinant). Free variables
    /// no constraint reads count in full. Temporary constraints don't count,
    /// here or in full constraint, nor do the curve parameters they own. A
    /// real constraint's curve parameter is a free variable its own rows
    /// take back (a point `on` a Spline: two equations, one parameter, one
    /// degree of freedom less).
    ///
    /// **Fully constrained**: IDs of the Points, Circles, Arcs and Ellipses
    /// with zero degrees of freedom, sorted; meaningful only at a solved
    /// configuration. A free variable is locked when no motion in its
    /// Component's Jacobian
    /// [null space](https://en.wikipedia.org/wiki/Kernel_(linear_algebra)) moves it; fixed variables are always locked; free
    /// variables no constraint reads are not. An entity is fully constrained
    /// when all of its variables are locked. Lines and Splines have no
    /// variables of their own; callers decide them from their Points.
    /// Constraints' curve parameters are not entities and never listed.
    ///
    /// **Diagnosis**: Conflicting and Redundant constraints. A sketch that
    /// only looks conflicting where it started isn't. Temporary constraints
    /// are outside the diagnosed system: their rows are left out and they're
    /// never reported. Rows are *dependent* when some combination of them
    /// vanishes, i.e. they have weight in a vector w with wᵀJ = 0 (the [left
    /// null space](https://en.wikipedia.org/wiki/Kernel_(linear_algebra)#Left_null_space) of J, from the
    /// zero eigenvalues of JJᵀ).
    ///
    /// - Component not Solved (some |rᵢ| > TOLF): r_N, the part of r in the
    ///   left null space, is residual that no motion reduces (to first
    ///   order; at a least-squares stall it is all of r). Every constraint
    ///   with a row where |r_N| > TOLF is Conflicting. A dependency shares
    ///   its residual across all its rows, so both sides of a conflict are
    ///   reported. Dependent constraints that carry no residual are not.
    ///   Nothing is Redundant.
    /// - Component Solved: walking the dependent constraints from last to
    ///   first in sketch order, a constraint is Redundant when dropping it
    ///   (together with those already dropped) leaves the rank of J
    ///   unchanged, less one for each curve parameter it owns that no kept
    ///   constraint reads (that unknown goes with it: a second `on` holding
    ///   a point on a Spline brings two rows and a parameter, rank 1 more,
    ///   and adds nothing). The result is a minimal set whose removal changes
    ///   neither the solution nor the degrees of freedom; of two duplicates,
    ///   the later is reported. Nothing is Conflicting.
    ///
    ///
    /// A constraint over fixed variables only has a zero row, so it is
    /// Conflicting when it doesn't hold and Redundant when it does.
    ///
    /// Implicit arc rules are diagnosed rows like any other but are never
    /// reported (they have no sketch index): a contradiction with them
    /// reports only the sketch constraints involved, and the Redundant walk
    /// never drops them, as if they came first.
    pub(crate) fn analyze(&self) -> Analysis {
        let info = self.pm.var_info();
        let mut locked: Vec<bool> = info.iter().map(|p| p.is_fixed).collect();
        let mut rank_sum = 0;
        let mut diagnosis = Diagnosis::default();

        for comp in &self.components {
            let which = &comp.diagnosed;
            if which.is_empty() {
                continue;
            }
            let (r, jac) = evaluate(&self.pm, &self.constraints, comp, which);
            let solved = r.amax() <= TOLF;
            let (r, jac) = equilibrate(r, jac);

            let rank = if comp.columns.is_empty() {
                0
            } else {
                let eig = SymmetricEigen::new(jac.transpose() * &jac);
                let mut null_motion_sq = vec![0.0f64; comp.columns.len()];
                for (k, &lambda) in eig.eigenvalues.iter().enumerate() {
                    if lambda.abs() <= ZERO_EIG {
                        let v = eig.eigenvectors.column(k);
                        for (m, sq) in null_motion_sq.iter_mut().enumerate() {
                            *sq += v[m] * v[m];
                        }
                    }
                }
                for (local, &global) in comp.columns.iter().enumerate() {
                    let touched = jac.column(local).iter().any(|&x| x != 0.0);
                    if touched && null_motion_sq[local].sqrt() <= MOTION_EPS {
                        locked[global] = true;
                    }
                }
                eig.eigenvalues.iter().filter(|l| l.abs() > ZERO_EIG).count()
            };
            rank_sum += rank;
            self.diagnose_component(which, solved, &r, &jac, rank, &mut diagnosis);
        }
        diagnosis.conflicting.sort_unstable();
        diagnosis.redundant.sort_unstable();

        let free = info
            .iter()
            .filter(|p| !p.is_fixed && !self.goal_curve_params.contains(&p.global_index))
            .count();

        let mut per_entity: BTreeMap<&str, bool> = BTreeMap::new();
        for p in info.iter().filter(|p| p.entity_type != EntityType::CurveParam) {
            *per_entity.entry(&p.entity_id).or_insert(true) &= locked[p.global_index];
        }
        let fully_constrained = per_entity
            .into_iter()
            .filter(|&(_, all_locked)| all_locked)
            .map(|(id, _)| id.to_string())
            .collect();

        Analysis {
            dof: free - rank_sum,
            fully_constrained,
            diagnosis,
        }
    }

    /// One Component's Conflicting and Redundant constraints (the rule is on
    /// [`Self::analyze`]), from its diagnosed constraints `which`, whether
    /// they all hold (`solved`), their residuals `r` and Jacobian `jac` (both
    /// [equilibrated](equilibrate)) and its rank `full_rank`.
    fn diagnose_component(
        &self,
        which: &[usize],
        solved: bool,
        r: &DVector<f64>,
        jac: &DMatrix<f64>,
        full_rank: usize,
        out: &mut Diagnosis,
    ) {
        // Row range of each diagnosed constraint.
        let mut rows = Vec::with_capacity(which.len());
        let mut start = 0;
        for &i in which {
            let n = self.constraints.get(i).num_residuals();
            rows.push(start..start + n);
            start += n;
        }

        let eig = SymmetricEigen::new(jac * jac.transpose());
        let null: Vec<usize> = (0..eig.eigenvalues.len())
            .filter(|&k| eig.eigenvalues[k].abs() <= ZERO_EIG)
            .collect();
        if null.is_empty() {
            return;
        }

        if !solved {
            let mut r_null = DVector::zeros(r.len());
            for &k in &null {
                let w = eig.eigenvectors.column(k);
                r_null += w * w.dot(r);
            }
            for (&i, range) in which.iter().zip(&rows) {
                if !self.constraints.is_implicit(i) && range.clone().any(|row| r_null[row].abs() > TOLF) {
                    out.conflicting.push(i);
                }
            }
            return;
        }

        let dependent = |range: &std::ops::Range<usize>| {
            range.clone().any(|row| {
                let weight: f64 = null.iter().map(|&k| eig.eigenvectors[(row, k)].powi(2)).sum();
                weight.sqrt() > MOTION_EPS
            })
        };
        let owned: Vec<Vec<usize>> = which.iter().map(|&i| owned_columns(self.constraints.get(i), &self.pm)).collect();
        let curve_param_reads: Vec<HashSet<usize>> = which
            .iter()
            .map(|&i| {
                reads(self.constraints.get(i))
                    .iter()
                    .filter(|v| matches!(v, Var::CurveParam(_)))
                    .filter_map(|v| v.column(&self.pm))
                    .collect()
            })
            .collect();
        let mut dropped = vec![false; which.len()];
        for c in (0..which.len()).rev() {
            // Implicit rules are never dropped: they're always kept, as if
            // first in sketch order.
            if self.constraints.is_implicit(which[c]) || !dependent(&rows[c]) {
                continue;
            }
            dropped[c] = true;
            let kept = rows
                .iter()
                .zip(&dropped)
                .filter(|&(_, &d)| !d)
                .flat_map(|(range, _)| range.clone());
            let vanished = (0..which.len())
                .filter(|&k| dropped[k])
                .flat_map(|k| &owned[k])
                .filter(|col| (0..which.len()).all(|k| dropped[k] || !curve_param_reads[k].contains(col)))
                .count();
            if rank(&jac.select_rows(kept.collect::<Vec<_>>().iter())) + vanished == full_rank {
                out.redundant.push(which[c]);
            } else {
                dropped[c] = false;
            }
        }
    }
}

/// The degrees-of-freedom analysis and diagnosis of a sketch at its current
/// configuration (see `ConstraintSolver::analyze`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Analysis {
    /// Degrees of freedom the real constraints leave.
    pub dof: usize,
    /// IDs of the Points, Circles, Arcs and Ellipses with zero degrees of
    /// freedom, sorted. Lines and Splines own no variables and aren't listed.
    pub fully_constrained: Vec<String>,
    /// Conflicting and Redundant constraints.
    pub diagnosis: Diagnosis,
}

/// Threshold for a zero eigenvalue of JᵀJ (null space), consistent with the
/// 1e-10 residual tolerance.
const ZERO_EIG: f64 = 1e-9;
/// Threshold for a nonzero motion within the null space.
const MOTION_EPS: f64 = 1e-7;
/// A row or column of J with norm at most this fraction of the largest is
/// numerically zero (see [`unit_rows`]): far below any real ratio between
/// rows (an angle row in a sketch a million across against a distance
/// row is 1e-6), far above rounding.
const NUMERIC_ZERO: f64 = 1e-12;
/// Most trial steps (each shrinking the trust region) for one goal step in
/// [`solve_with_goals`].
const MAX_TRIALS: usize = 30;
/// The first trust radius in [`solve_with_goals`] is at most this many
/// times the goal error.
const INITIAL_RADIUS_FACTOR: f64 = 10.0;
/// [`solve_with_goals`] tries the Gauss–Newton step only when its model
/// removes at least this fraction of the goal error.
const GAUSS_NEWTON_REACH: f64 = 0.9;
/// Central-difference step for the Lagrangian Hessian, relative to the
/// largest variable.
const FD_STEP: f64 = 1e-5;
/// Reduced-Hessian eigenvalues within this fraction of the largest are
/// treated as zero curvature (finite-difference noise is far below it).
const CURVATURE_EPS: f64 = 1e-6;

/// `jac` with its rows, then its columns, scaled to unit length, and `r`
/// with the row scales: the system the analysis works on.
///
/// Rows come in different units (a distance's gradient is unitless, an
/// angle's goes as 1/length), so JᵀJ's eigenvalues scale with the sketch,
/// and the absolute `ZERO_EIG` would read an angle row in a sketch in the
/// hundred thousands as zero. Scaling rows and columns by nonzero factors
/// changes neither the rank nor which rows a dependency involves nor which
/// variables a null motion moves, so degrees of freedom, diagnosis and
/// full constraint mean the same at every scale.
fn equilibrate(mut r: DVector<f64>, mut jac: DMatrix<f64>) -> (DVector<f64>, DMatrix<f64>) {
    for (i, n) in unit_rows(&mut jac).into_iter().enumerate() {
        r[i] /= n;
    }
    let norms: Vec<f64> = jac.column_iter().map(|c| c.norm()).collect();
    let floor = NUMERIC_ZERO * norms.iter().copied().fold(0.0, f64::max);
    for (k, n) in norms.into_iter().enumerate() {
        if n > floor {
            jac.column_mut(k).unscale_mut(n);
        }
    }
    (r, jac)
}

/// Scales each row of `jac` to unit length, returning the scales (1 for a
/// row left alone). A row whose norm is within `NUMERIC_ZERO` of the
/// largest row's is numerically zero (rounding, or exactly 0) and is left
/// as it is: scaling it up would turn noise into a direction. Row scaling
/// keeps J's null space, its rank and which rows depend on which.
fn unit_rows(jac: &mut DMatrix<f64>) -> Vec<f64> {
    let norms: Vec<f64> = jac.row_iter().map(|r| r.norm()).collect();
    let floor = NUMERIC_ZERO * norms.iter().copied().fold(0.0, f64::max);
    norms
        .into_iter()
        .enumerate()
        .map(|(i, n)| {
            if n > floor {
                jac.row_mut(i).unscale_mut(n);
                n
            } else {
                1.0
            }
        })
        .collect()
}

/// Numerical rank of `jac`: the eigenvalues of JᵀJ above `ZERO_EIG`, as in
/// the degrees-of-freedom analysis.
fn rank(jac: &DMatrix<f64>) -> usize {
    if jac.ncols() == 0 || jac.nrows() == 0 {
        return 0;
    }
    (jac.transpose() * jac)
        .symmetric_eigenvalues()
        .iter()
        .filter(|l| l.abs() > ZERO_EIG)
        .count()
}

/// Residuals and Jacobian of some of a Component's constraints (`which`, rows
/// in that order), over the Component's local columns.
fn evaluate(
    pm: &VarRegistry,
    constraints: &Constraints,
    comp: &Component,
    which: &[usize],
) -> System {
    let num_residuals = which.iter().map(|&i| constraints.get(i).num_residuals()).sum();
    let mut r = DVector::zeros(num_residuals);
    let mut jac = DMatrix::zeros(num_residuals, comp.columns.len());
    let column_of = |global: usize| comp.local_of.get(&global).copied();
    let mut row = 0;
    for &i in which {
        let c = constraints.get(i);
        eval_into(c, pm, &column_of, row, &mut r, &mut jac);
        row += c.num_residuals();
    }
    (r, jac)
}

/// The Component's current local values.
fn values(pm: &VarRegistry, comp: &Component) -> DVector<f64> {
    DVector::from_iterator(comp.columns.len(), comp.columns.iter().map(|&c| pm.values()[c]))
}

/// Solves a Component that has soft goals (temporary constraints): minimizes
/// ‖goal residuals‖² subject to the real constraints' residuals being zero.
/// GCS does this with an SQP (`System::solve(subsysA, subsysB)`); here it is
/// a feasible trust-region [SQP](https://en.wikipedia.org/wiki/Sequential_quadratic_programming)
/// whose feasibility step is the existing Dog-Leg:
///
/// 1. *Restore*: Dog-Leg on the real constraints alone, from the current
///    values. If it doesn't converge the real constraints can't be solved
///    from here and the Component fails; goals play no part in that.
/// 2. *Model*: at a feasible x, with J_A the real constraints' Jacobian and
///    Z an orthonormal basis of its null space (the motions they allow, to
///    first order), the goal error f = ½‖r_B‖² along x + Z·u is modelled by
///    m(u) = gᵀu + ½uᵀHu: g = Zᵀ·J_Bᵀ·r_B and H = Zᵀ·∇²L·Z, the
///    [Hessian](https://en.wikipedia.org/wiki/Hessian_matrix) of the Lagrangian L = f + λᵀr_A
///    (least-squares [multipliers](https://en.wikipedia.org/wiki/Lagrange_multiplier) λ), by
///    [central differences](https://en.wikipedia.org/wiki/Finite_difference#Basic_types) of
///    ∇L = J_Bᵀr_B + J_Aᵀλ along each column of Z (the analytical
///    Jacobians; two evaluations per column).
/// 3. *Step*: first the Gauss–Newton model (H = ZᵀJ_BᵀJ_BZ, λ = 0), one
///    trial, when its Newton step removes most of the goal error
///    (`GAUSS_NEWTON_REACH`); it is accepted only if m predicted the drop
///    well (ρ ≥ ¼). Its pseudo-inverse step is the smallest motion that
///    meets the goals to first order, so free geometry the goals don't
///    need stays put. Otherwise the second-order model: the
///    [Newton step](https://en.wikipedia.org/wiki/Newton%27s_method_in_optimization)
///    when it fits the trust radius Δ and earns at
///    least half the predicted decrease, else argmin m(u) over ‖u‖ ≤ Δ
///    (exact, by H's eigen-decomposition, `ReducedModel::trust_region_step`).
///    Each model keeps its own Δ.
/// 4. *Re-project*: Dog-Leg on the real constraints from x + Z·u, which
///    lands back on their solution set. Accept when f drops; grow or shrink
///    Δ by how well m predicted the drop.
/// 5. Repeat 2–4 until the goals hold, Z is empty (nothing may move, so a
///    fully constrained drag moves nothing), the step is negligible or
///    predicts no decrease, no trial step lowers f, or the budget
///    (`max_iterations` goal steps) is spent.
///
/// Why second order: at a *fold* the real constraints change only
/// quadratically in the direction a goal needs (a fixed-length edge that is
/// exactly horizontal must tilt for its corner to move sideways). There Z
/// has no such direction and the tilt has a zero column in J_A, so a
/// first-order method (projected Gauss–Newton, or Dog-Leg on the real
/// constraints and low-weighted goals together: its linear model has the
/// same zero column) only creeps on incidental tilt; ticket 23's repro sat
/// unmoved after ~1700 iterations, reported Converged. ∇²L sees the fold
/// as negative curvature along the tilt: with g = 0 there (the *hard
/// case*) the step goes along it to the trust boundary, and once tilted,
/// first-order motion exists again. Near a goal that can be met, λ → 0 and H tends to
/// the Gauss–Newton ZᵀJ_BᵀJ_BZ: Newton steps, fast local convergence.
///
/// Why Gauss–Newton first: away from a fold the λ-terms of ∇L still curve
/// the model along motions the goals don't need, and so choose how free
/// geometry moves, by how the constraints happen to be written. A corner
/// dragged toward a circle it is tangent to (TangentAtPoint, radius free)
/// scaled the whole sketch about the center: the angle row's curvature
/// left scaling the only free motion (`tests/sketch_json_test.rs`
/// `a_drag_toward_a_circle_moves_the_lines_before_the_radius`).
///
/// Every accepted x satisfies the real constraints, so after a successful
/// restore the result is Converged whatever the goals ask; unreachable goals
/// end at a (second-order) local minimum of f on the real constraints.
/// A goal met exactly *at* a fold (the edge of what the real constraints
/// allow) is a singular solution and converges only linearly. A Component
/// of goals only is plain least squares on the goals, always Converged.
///
/// Iterations count Dog-Leg iterations plus accepted goal steps; errors are
/// the real constraints' residual norms (0 when there are none).
fn solve_with_goals(
    dogleg: &ParametricDogLegSolver,
    pm: &mut VarRegistry,
    constraints: &Constraints,
    comp: &Component,
) -> SolverResult {
    let real = &comp.diagnosed;
    let goals = &comp.goals;
    let mut x = values(pm, comp);

    // Dog-Leg on `which` from `x`, leaving the result in `x` and `pm`.
    let run = |pm: &mut VarRegistry, x: &mut DVector<f64>, which: &[usize], iterations: &mut usize| {
        let result = dogleg.solve_dl(x, &mut |x| {
            set_columns(pm, comp, x);
            evaluate(pm, constraints, comp, which)
        });
        set_columns(pm, comp, x);
        let (SolverResult::Converged { iterations: it, .. }
        | SolverResult::MaxIterationsReached { iterations: it, .. }) = result;
        *iterations += it;
        matches!(result, SolverResult::Converged { .. })
    };

    let mut iterations = 0;
    if real.is_empty() {
        run(pm, &mut x, goals, &mut iterations);
        return SolverResult::Converged {
            iterations,
            final_error: 0.0,
            initial_error: 0.0,
        };
    }

    let initial_error = evaluate(pm, constraints, comp, real).0.norm();
    if !run(pm, &mut x, real, &mut iterations) {
        return SolverResult::MaxIterationsReached {
            iterations,
            final_error: evaluate(pm, constraints, comp, real).0.norm(),
            initial_error,
        };
    }

    let mut r_goal = evaluate(pm, constraints, comp, goals).0;
    // One trust radius per model: [Gauss–Newton, second order].
    let mut radius: [Option<f64>; 2] = [None, None];
    'outer: for _ in 0..dogleg.max_iterations() {
        if r_goal.amax() <= TOLF {
            break;
        }
        // The Gauss–Newton step first, one trial, when its model meets most
        // of the goal; the second-order model when it doesn't (at a fold)
        // or the step gets nowhere.
        for second_order in [false, true] {
            let Some(model) = ReducedModel::at(pm, constraints, comp, &x, second_order) else {
                break 'outer;
            };
            let negligible = 1e-12 * (1.0 + x.norm());
            let f = 0.5 * r_goal.norm_squared();
            if !second_order && -model.change(&model.newton_step()) < GAUSS_NEWTON_REACH * f {
                continue;
            }
            // First radius: the Newton step, unless it is far longer than the
            // goal error (near a fold, where the tangent model is poor), or
            // vanishes (at a fold, where only the curvature leads anywhere).
            let delta = radius[usize::from(second_order)].get_or_insert_with(|| {
                let newton = model.newton_step().norm();
                let goal = r_goal.norm();
                if newton > 0.0 { newton.min(INITIAL_RADIUS_FACTOR * goal) } else { goal }
            });

            for _ in 0..if second_order { MAX_TRIALS } else { 1 } {
                let u = model.step(*delta);
                let step = u.norm();
                let predicted = -model.change(&u);
                if step <= negligible || predicted <= 0.0 {
                    break;
                }
                let mut x_try = &x + &model.z * &u;
                if run(pm, &mut x_try, real, &mut iterations) {
                    let r = evaluate(pm, constraints, comp, goals).0;
                    let actual = f - 0.5 * r.norm_squared();
                    let rho = actual / predicted;
                    // A Gauss–Newton step the model mispredicts is left to
                    // the second-order model: accepting it would creep.
                    if actual > 0.0 && (second_order || rho >= 0.25) {
                        if rho < 0.25 {
                            *delta = step / 4.0;
                        } else if rho > 0.75 && step >= 0.99 * *delta {
                            *delta *= 4.0;
                        }
                        iterations += 1;
                        (x, r_goal) = (x_try, r);
                        continue 'outer;
                    }
                }
                *delta = step / 4.0;
            }
        }
        break;
    }

    set_columns(pm, comp, &x);
    SolverResult::Converged {
        iterations,
        final_error: evaluate(pm, constraints, comp, real).0.norm(),
        initial_error,
    }
}

/// The quadratic model of the goal error ½‖r_B‖² on the real constraints'
/// solution set near a feasible x, in the coordinates u of their Jacobian's
/// null space (x + Z·u): m(u) = gᵀu + ½uᵀHu (see [`solve_with_goals`]).
struct ReducedModel {
    /// Orthonormal basis (columns) of the null space of J_A.
    z: DMatrix<f64>,
    /// Reduced gradient Zᵀ·J_Bᵀ·r_B.
    g: DVector<f64>,
    /// Reduced Hessian of the Lagrangian, Zᵀ·∇²L·Z (or its Gauss–Newton
    /// part ZᵀJ_BᵀJ_BZ), as eigenvalues and eigenvectors.
    eig: SymmetricEigen<f64, nalgebra::Dyn>,
}

impl ReducedModel {
    /// The model at the feasible `x` (also left in `pm`), with the
    /// Lagrangian's curvature when `second_order`, else Gauss–Newton;
    /// `None` when the real constraints allow no motion.
    fn at(
        pm: &mut VarRegistry,
        constraints: &Constraints,
        comp: &Component,
        x: &DVector<f64>,
        second_order: bool,
    ) -> Option<Self> {
        let (real, goals) = (&comp.diagnosed, &comp.goals);
        set_columns(pm, comp, x);
        let (_, j_real) = evaluate(pm, constraints, comp, real);
        let (r_goal, j_goal) = evaluate(pm, constraints, comp, goals);
        let z = null_space_basis(&j_real);
        if z.ncols() == 0 {
            return None;
        }
        let grad = j_goal.transpose() * &r_goal;
        if !second_order {
            let jz = &j_goal * &z;
            let eig = SymmetricEigen::new(jz.transpose() * jz);
            return Some(Self {
                g: z.transpose() * grad,
                z,
                eig,
            });
        }
        // Least-squares multipliers: ∇L = J_Bᵀr_B + J_Aᵀλ is as small as it
        // gets (zero along J_A's row space).
        let lambda = j_real
            .transpose()
            .svd(true, true)
            .solve(&(-&grad), 1e-12)
            .unwrap_or_else(|_| DVector::zeros(j_real.nrows()));

        // ∇²L·z_k by central differences of ∇L (the analytical Jacobians),
        // with λ held: the Gauss–Newton part J_BᵀJ_B and the curvature of
        // every residual, which is what sees a fold.
        let grad_l = |pm: &mut VarRegistry, x: &DVector<f64>| {
            set_columns(pm, comp, x);
            let (r_b, j_b) = evaluate(pm, constraints, comp, goals);
            let (_, j_a) = evaluate(pm, constraints, comp, real);
            j_b.transpose() * r_b + j_a.transpose() * &lambda
        };
        let h = FD_STEP * (1.0 + x.amax());
        let mut hz = DMatrix::zeros(x.len(), z.ncols());
        for k in 0..z.ncols() {
            let dz = h * z.column(k);
            let diff = grad_l(pm, &(x + &dz)) - grad_l(pm, &(x - &dz));
            hz.set_column(k, &(diff / (2.0 * h)));
        }
        set_columns(pm, comp, x);
        let reduced = z.transpose() * hz;
        let eig = SymmetricEigen::new(0.5 * (&reduced + reduced.transpose()));
        Some(Self {
            g: z.transpose() * grad,
            z,
            eig,
        })
    }

    /// Eigenvalues within this of zero (relative to the largest) are zero.
    fn curvature_tol(&self) -> f64 {
        CURVATURE_EPS * self.eig.eigenvalues.amax().max(1.0)
    }

    /// m(u) − m(0).
    fn change(&self, u: &DVector<f64>) -> f64 {
        let v = self.eig.eigenvectors.transpose() * u;
        self.g.dot(u) + 0.5 * v.iter().zip(self.eig.eigenvalues.iter()).map(|(v, l)| l * v * v).sum::<f64>()
    }

    /// −Σ (vᵢᵀg)/(lᵢ + μ)·vᵢ over the eigenpairs `keep` accepts.
    fn shifted_step(&self, mu: f64, keep: impl Fn(f64) -> bool) -> DVector<f64> {
        let gt = self.eig.eigenvectors.transpose() * &self.g;
        let mut u = DVector::zeros(self.g.len());
        for (i, &l) in self.eig.eigenvalues.iter().enumerate() {
            if keep(l + mu) {
                u -= self.eig.eigenvectors.column(i) * (gt[i] / (l + mu));
            }
        }
        u
    }

    /// The (pseudo-inverse) Newton step over the positive curvature.
    fn newton_step(&self) -> DVector<f64> {
        let tol = self.curvature_tol();
        self.shifted_step(0.0, |l| l > tol)
    }

    /// The step to try within radius Δ: the Newton step when it fits and
    /// gets at least half the trust-region step's predicted decrease (near
    /// a goal that can be met, the curvature left is the vanishing
    /// multipliers' and only slows the trust-region step down), else the
    /// trust-region step.
    fn step(&self, delta: f64) -> DVector<f64> {
        let tr = self.trust_region_step(delta);
        let newton = self.newton_step();
        if newton.norm() <= delta && -self.change(&newton) >= -0.5 * self.change(&tr) {
            newton
        } else {
            tr
        }
    }

    /// argmin m(u) over ‖u‖ ≤ Δ, by the eigen-decomposition (Moré–Sorensen):
    /// u(μ) = −(H + μI)⁻¹g with μ ≥ max(0, −λ_min) and ‖u‖ = Δ unless the
    /// Newton step fits; in the *hard case* (g has no part along the most
    /// negative curvature, as at a symmetric fold) the rest of the way to
    /// the boundary is along that curvature.
    fn trust_region_step(&self, delta: f64) -> DVector<f64> {
        let tol = self.curvature_tol();
        let l_min = self.eig.eigenvalues.min();
        if l_min >= -tol {
            let u = self.newton_step();
            if u.norm() <= delta {
                return u;
            }
        }
        let lo = (-l_min).max(0.0);
        if l_min < -tol {
            let gt = self.eig.eigenvectors.transpose() * &self.g;
            let along_min: f64 = (0..gt.len())
                .filter(|&i| self.eig.eigenvalues[i] <= l_min + tol)
                .map(|i| gt[i] * gt[i])
                .sum::<f64>()
                .sqrt();
            let u = self.shifted_step(lo, |l| l > tol);
            if along_min <= 1e-12 * self.g.norm() && u.norm() <= delta {
                let k = self.eig.eigenvalues.imin();
                let v = self.eig.eigenvectors.column(k).into_owned();
                // Either way along a fold; deterministically, the way the
                // gradient doesn't oppose, then the largest entry's sign.
                let toward = self.g.dot(&v);
                let sign = if toward > 0.0 || (toward == 0.0 && v[v.iamax()] < 0.0) { -1.0 } else { 1.0 };
                let tau = sign * (delta * delta - u.norm_squared()).max(0.0).sqrt();
                return u + v * tau;
            }
        }
        // ‖u(μ)‖ falls from ∞ at `lo` to ≤ Δ at `hi`: bisect.
        let mut lo = lo;
        let mut hi = lo + self.g.norm() / delta + tol;
        for _ in 0..100 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if self.shifted_step(mid, |l| l > 0.0).norm() > delta {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.shifted_step(hi, |l| l > 0.0)
    }
}

/// An orthonormal basis (as columns) of the null space of `jac`: the
/// eigenvectors of JᵀJ with eigenvalues within `ZERO_EIG`, as in the
/// degrees-of-freedom analysis, with J's rows at unit length first (the
/// null space is the same; `ZERO_EIG` then means the same at every scale,
/// where an angle row in a large sketch would fall under it).
fn null_space_basis(jac: &DMatrix<f64>) -> DMatrix<f64> {
    let n = jac.ncols();
    let mut jac = jac.clone();
    unit_rows(&mut jac);
    let eig = SymmetricEigen::new(jac.transpose() * &jac);
    let null: Vec<usize> = (0..n).filter(|&k| eig.eigenvalues[k].abs() <= ZERO_EIG).collect();
    eig.eigenvectors.select_columns(null.iter())
}

/// Writes a Component's local values into the global variable vector.
fn set_columns(pm: &mut VarRegistry, comp: &Component, x: &DVector<f64>) {
    let params = pm.values_mut();
    for (&global, &v) in comp.columns.iter().zip(x.iter()) {
        params[global] = v;
    }
}

/// Each Arc's implicit rules, in sorted arc order: its start and end Points
/// at its radius from its center, at its start and end angles; then each
/// EllipticalArc's, in sorted order: its start and end Points on its ellipse
/// at their parametric angles. Arcs whose center, focus or endpoints aren't
/// Points get none (they can't be solved).
fn implicit_arc_rules(geometry: &GeometrySystem) -> Vec<Box<dyn Constraint>> {
    let has_points = |ids: &[&String]| ids.iter().all(|p| geometry.get_point(p).is_some());
    let mut arcs: Vec<_> = geometry.get_all_arcs().values().collect();
    arcs.sort_by(|a, b| a.id.cmp(&b.id));
    let mut elliptical: Vec<_> = geometry.get_all_elliptical_arcs().values().collect();
    elliptical.sort_by(|a, b| a.id.cmp(&b.id));
    let circular = arcs
        .into_iter()
        .filter(|a| has_points(&[&a.center, &a.start, &a.end]))
        .map(|a| {
            Box::new(ArcRulesConstraint::new(a.center.clone(), a.start.clone(), a.end.clone(), a.id.clone()))
                as Box<dyn Constraint>
        });
    let elliptical = elliptical
        .into_iter()
        .filter(|a| has_points(&[&a.center, &a.focus1, &a.start, &a.end]))
        .map(|a| {
            Box::new(EllipticalArcRulesConstraint::new(
                a.center.clone(),
                a.focus1.clone(),
                a.start.clone(),
                a.end.clone(),
                a.id.clone(),
            )) as Box<dyn Constraint>
        });
    circular.chain(elliptical).collect()
}

/// Global columns of the curve parameters `c` owns.
fn owned_columns(c: &dyn Constraint, pm: &VarRegistry) -> Vec<usize> {
    c.curve_params().into_iter().filter_map(|id| Var::CurveParam(id).column(pm)).collect()
}

/// Registers every Point, Circle, Arc, Ellipse and EllipticalArc, then the
/// constraints' curve parameters, in a deterministic (sorted) order.
fn build_var_registry(geometry: &GeometrySystem) -> VarRegistry {
    let mut pm = VarRegistry::new();

    let mut point_ids: Vec<&String> = geometry.get_all_points().keys().collect();
    point_ids.sort();
    for id in point_ids {
        pm.register_entity(id.clone(), EntityType::Point, &geometry.get_all_points()[id]);
    }

    let mut circle_ids: Vec<&String> = geometry.get_all_circles().keys().collect();
    circle_ids.sort();
    for id in circle_ids {
        pm.register_entity(id.clone(), EntityType::Circle, &geometry.get_all_circles()[id]);
    }

    let mut arc_ids: Vec<&String> = geometry.get_all_arcs().keys().collect();
    arc_ids.sort();
    for id in arc_ids {
        pm.register_entity(id.clone(), EntityType::Arc, &geometry.get_all_arcs()[id]);
    }

    let mut ellipse_ids: Vec<&String> = geometry.get_all_ellipses().keys().collect();
    ellipse_ids.sort();
    for id in ellipse_ids {
        pm.register_entity(id.clone(), EntityType::Ellipse, &geometry.get_all_ellipses()[id]);
    }

    let mut elliptical_arc_ids: Vec<&String> = geometry.get_all_elliptical_arcs().keys().collect();
    elliptical_arc_ids.sort();
    for id in elliptical_arc_ids {
        pm.register_entity(
            id.clone(),
            EntityType::EllipticalArc,
            &geometry.get_all_elliptical_arcs()[id],
        );
    }

    let mut param_ids: Vec<&String> = geometry.get_all_curve_params().keys().collect();
    param_ids.sort();
    for id in param_ids {
        pm.register_entity(id.clone(), EntityType::CurveParam, &geometry.get_all_curve_params()[id]);
    }

    pm
}
