#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{GeometrySystem, VarRegistry};

/// One solver variable a constraint reads, named by role: a number the solver
/// may move, such as a Point's x. Not a Parameter, which is a named, fixed
/// value in the sketch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Var<'a> {
    /// x coordinate of a Point.
    X(&'a str),
    /// y coordinate of a Point.
    Y(&'a str),
    /// Radius of a Circle or Arc.
    Radius(&'a str),
    /// Start angle of an Arc or EllipticalArc, in radians (an
    /// EllipticalArc's is parametric).
    StartAngle(&'a str),
    /// End angle of an Arc or EllipticalArc, in radians.
    EndAngle(&'a str),
    /// Minor radius (`radmin`) of an Ellipse or EllipticalArc.
    MinorRadius(&'a str),
}

impl Var<'_> {
    /// ID of the entity this variable belongs to.
    pub fn entity_id(&self) -> &str {
        match self {
            Var::X(id)
            | Var::Y(id)
            | Var::Radius(id)
            | Var::StartAngle(id)
            | Var::EndAngle(id)
            | Var::MinorRadius(id) => id,
        }
    }

    /// Global column of this variable in `pm`, if its entity is registered.
    pub fn column(&self, pm: &VarRegistry) -> Option<usize> {
        pm.get_global_index(self.entity_id(), self.index_in_entity())
    }

    /// Index of this variable within its entity's values.
    fn index_in_entity(&self) -> usize {
        match self {
            Var::X(_) | Var::Radius(_) | Var::MinorRadius(_) => 0,
            Var::Y(_) | Var::StartAngle(_) => 1,
            Var::EndAngle(_) => 2,
        }
    }
}

/// A scalar a constraint reads: either a constant (a number, or a
/// Parameter's value) or an entity property the solver may move, such as a
/// Circle's radius.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Const(f64),
    /// x coordinate of a Point.
    X(String),
    /// y coordinate of a Point.
    Y(String),
    /// Radius of a Circle or Arc.
    Radius(String),
    /// Minor radius (`radmin`) of an Ellipse.
    MinorRadius(String),
}

impl Operand {
    /// The solver variable this operand reads, or `None` for a constant.
    pub fn var(&self) -> Option<Var<'_>> {
        match self {
            Operand::Const(_) => None,
            Operand::X(id) => Some(Var::X(id)),
            Operand::Y(id) => Some(Var::Y(id)),
            Operand::Radius(id) => Some(Var::Radius(id)),
            Operand::MinorRadius(id) => Some(Var::MinorRadius(id)),
        }
    }
}

/// Values of `operands`, taking the variable ones from `x` in order (as laid
/// out by their `var()`s), and the local column of each (`None` for constants).
pub(crate) fn operand_values(operands: &[&Operand], x: &[f64]) -> Vec<(f64, Option<usize>)> {
    let mut next = 0;
    operands
        .iter()
        .map(|o| match o {
            Operand::Const(v) => (*v, None),
            _ => {
                next += 1;
                (x[next - 1], Some(next - 1))
            }
        })
        .collect()
}

/// `[X(id), Y(id)]`: both coordinates of a Point.
pub fn xy(id: &str) -> [Var<'_>; 2] {
    [Var::X(id), Var::Y(id)]
}

/// Writes `g` into row `row` of a local Jacobian.
pub(crate) fn set_row(j: &mut DMatrix<f64>, row: usize, g: &[f64]) {
    for (k, &v) in g.iter().enumerate() {
        j[(row, k)] = v;
    }
}

/// A geometric constraint, written as a kernel over its own variables.
///
/// A constraint declares the variables it drives (`vars`), optionally the
/// Guides it follows (`guides`), and evaluates residuals and their partials
/// over those local values (`eval`). Gathering values from the global vector
/// and scattering partials into the global Jacobian happen once, in the
/// provided `residual` / `jacobian` methods and `eval_into`, which accumulate
/// so a variable listed twice (two lines sharing a point) gets the sum of its
/// partials.
pub trait Constraint {
    /// Variables this constraint drives: `eval` reads them first, in this
    /// order, and writes their partials. The same variable may appear more
    /// than once.
    fn vars(&self) -> Vec<Var<'_>>;

    /// Guides: variables `eval` also reads (after `vars()`, in this order)
    /// that the constraint copies across, such as a mirror's axis or an
    /// array's center or direction. `eval` writes their partials like any
    /// other input and every solve, drags included, moves them as the
    /// constraints need (adding Horizontal to a patterned side turns the
    /// pattern about a center that moves with it). A variable may be both
    /// driven and a Guide.
    fn guides(&self) -> Vec<Var<'_>> {
        Vec::new()
    }

    fn num_residuals(&self) -> usize;

    /// Writes residuals into `r` (`num_residuals()` long) and their partials
    /// into `j` (`num_residuals() × reads(self).len()`, zeroed on entry).
    /// `x[k]` is the value of `reads(self)[k]`: the `vars()`, then the
    /// `guides()`; column `k` of `j` is the partial w.r.t. `x[k]`.
    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>);

    fn residual(&self, pm: &VarRegistry) -> DVector<f64> {
        let (r, _, _) = eval_local(self, pm);
        DVector::from(r)
    }

    /// Partials w.r.t. every global variable, Guides included.
    fn jacobian(&self, pm: &VarRegistry) -> DMatrix<f64> {
        let (r, local, cols) = eval_local(self, pm);
        let mut J = DMatrix::zeros(r.len(), pm.num_vars());
        for (k, &col) in cols.iter().enumerate() {
            for row in 0..r.len() {
                J[(row, col)] += local[(row, k)];
            }
        }
        J
    }
}

/// Every variable `c` reads, as `eval` receives them: `vars()`, then
/// `guides()`. Components are grouped by these, so a Guide joins the
/// Component of the constraints that move it.
pub fn reads<C: Constraint + ?Sized>(c: &C) -> Vec<Var<'_>> {
    let mut all = c.vars();
    all.extend(c.guides());
    all
}

/// Evaluates `c` at the current values: its residuals, its local Jacobian and
/// the global column of each of its columns.
fn eval_local<C: Constraint + ?Sized>(c: &C, pm: &VarRegistry) -> (Vec<f64>, DMatrix<f64>, Vec<usize>) {
    let (cols, x) = gather(&reads(c), pm);
    let mut r = vec![0.0; c.num_residuals()];
    let mut j = DMatrix::zeros(r.len(), cols.len());
    c.eval(&x, &mut r, &mut j);
    (r, j, cols)
}

/// Evaluates `c` at the current values, writing its residuals into
/// `r[row..]` and adding its partials into `J[row.., column_of(global)]`.
/// Variables whose global column maps to `None` (fixed ones) are skipped.
pub(crate) fn eval_into(
    c: &dyn Constraint,
    pm: &VarRegistry,
    column_of: &dyn Fn(usize) -> Option<usize>,
    row: usize,
    r: &mut DVector<f64>,
    J: &mut DMatrix<f64>,
) {
    let (local_r, local_j, cols) = eval_local(c, pm);
    let n = local_r.len();
    for (i, v) in local_r.into_iter().enumerate() {
        r[row + i] = v;
    }
    for (k, &global) in cols.iter().enumerate() {
        if let Some(col) = column_of(global) {
            for i in 0..n {
                J[(row + i, col)] += local_j[(i, k)];
            }
        }
    }
}

/// Global column and current value of each variable.
///
/// Panics on an unknown entity; `check_vars` rejects those when the
/// constraint is added.
fn gather(vars: &[Var<'_>], pm: &VarRegistry) -> (Vec<usize>, Vec<f64>) {
    let cols: Vec<usize> = vars
        .iter()
        .map(|p| {
            p.column(pm)
                .unwrap_or_else(|| panic!("constraint references unknown variable {p:?}"))
        })
        .collect();
    let values = cols.iter().map(|&c| pm.values()[c]).collect();
    (cols, values)
}

/// Checks that every variable a constraint reads exists in `geometry` and
/// belongs to the right kind of entity.
pub fn check_vars(constraint: &dyn Constraint, geometry: &GeometrySystem) -> Result<(), String> {
    for p in reads(constraint) {
        let ok = match p {
            Var::X(id) | Var::Y(id) => geometry.get_point(id).is_some(),
            Var::Radius(id) => {
                geometry.get_circle(id).is_some() || geometry.get_arc(id).is_some()
            }
            Var::StartAngle(id) | Var::EndAngle(id) => {
                geometry.get_arc(id).is_some() || geometry.get_elliptical_arc(id).is_some()
            }
            Var::MinorRadius(id) => {
                geometry.get_ellipse(id).is_some() || geometry.get_elliptical_arc(id).is_some()
            }
        };
        if !ok {
            let kind = match p {
                Var::X(_) | Var::Y(_) => "a point",
                Var::Radius(_) => "a circle or arc",
                Var::StartAngle(_) | Var::EndAngle(_) => "an arc or elliptical arc",
                Var::MinorRadius(_) => "an ellipse or elliptical arc",
            };
            return Err(format!("'{}' is not {kind}", p.entity_id()));
        }
    }
    Ok(())
}

/// One end of an arc: where it starts, or where it ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcEnd {
    Start,
    End,
}

/// One of an Ellipse's two axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EllipseAxis {
    /// Through the center and the focus; its endpoints are the major radius
    /// `a` from the center.
    Major,
    /// Perpendicular to the major axis at the center; its endpoints are the
    /// minor radius `radmin` from the center.
    Minor,
}

impl EllipseAxis {
    /// `major` or `minor`, as the JSON `which` field names it.
    pub fn as_str(self) -> &'static str {
        match self {
            EllipseAxis::Major => "major",
            EllipseAxis::Minor => "minor",
        }
    }
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

    /// Fixed distance from a point to a Line (the segment).
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

    /// Midpoint of Line (l1a, l1b) lies on Line (l2a, l2b) (the segment).
    MidpointOfLineOnLine(String, String, String, String),

    // ── Extension variants (measure against the infinite line through a
    //    Line's endpoints; native `extension: true`) ──
    /// A point lies on a Line's Extension.
    /// (point_id, line_pa_id, line_pb_id)
    PointOnExtension(String, String, String),

    /// Fixed perpendicular distance from a point to a Line's Extension.
    /// (point_id, line_pa_id, line_pb_id, distance)
    DistancePointExtension(String, String, String, f64),

    /// A circle is tangent to a Line's Extension (dist(center, line) = r).
    /// (line_pa_id, line_pb_id, circle_center_point_id, circle_id)
    TangentExtensionCircle(String, String, String, String),

    /// Midpoint of Line (l1a, l1b) lies on the Extension of Line (l2a, l2b).
    MidpointOfLineOnExtension(String, String, String, String),

    // ── Linked Offsets and property relations ──
    /// Signed perpendicular distance from a point to a Line's Extension:
    /// side · cross(b − a, p − a) / |b − a| = distance, so the point is held
    /// on the `side` (+1 left of a→b, −1 right) it was placed on.
    /// (point_id, line_pa_id, line_pb_id, distance, side)
    SignedDistancePointExtension(String, String, String, f64, f64),

    /// param2 − param1 = difference.
    /// (param1, param2, difference)
    Difference(Operand, Operand, Operand),

    /// param1 = param2.
    /// (param1, param2)
    Equal(Operand, Operand),

    // ── Arcs ──────────────────────────────────────────────────────────────────
    /// A point lies on an arc's span (on its circle, between its start and
    /// end angles counter-clockwise).
    /// (point_id, arc_center_point_id, arc_id)
    PointOnArc(String, String, String),

    /// A point is the midpoint of an arc's span: on its circle, halfway by
    /// angle from its start to its end, counter-clockwise.
    /// (point_id, arc_center_point_id, arc_id)
    MidpointOfArc(String, String, String),

    /// A Line (the segment) is tangent to an arc, touching it on both the
    /// segment and the arc's span.
    /// (line_pa_id, line_pb_id, arc_center_point_id, arc_id)
    TangentLineArc(String, String, String, String),

    // ── Arrays and mirror ──
    /// Direction of p1 → p2 is `angle` radians CCW from +x.
    /// (p1_id, p2_id, angle)
    PointPointAngle(String, String, f64),

    /// pB is pA mirrored across the Extension of the axis Line.
    /// (pA_id, pB_id, axis_pa_id, axis_pb_id)
    MirrorPointExtension(String, String, String, String),

    /// pk is p0 rotated about center by `angle` radians CCW.
    /// (p0_id, pk_id, center_id, angle)
    CircularInstance(String, String, String, f64),

    /// pk is p0 translated base_distance·n along dir_p1 → dir_p2.
    /// (p0_id, pk_id, dir_p1_id, dir_p2_id, base_distance, n)
    LinearInstance(String, String, String, String, f64, f64),

    // ── Ellipses (center Point, focus Point, `radmin` Var) ──
    /// A point lies on an ellipse.
    /// (point_id, ellipse_center_id, ellipse_focus1_id, ellipse_id)
    PointOnEllipse(String, String, String, String),

    /// A Line (the segment) is tangent to an ellipse, touching it on the
    /// segment.
    /// (line_pa_id, line_pb_id, ellipse_center_id, ellipse_focus1_id, ellipse_id)
    TangentLineEllipse(String, String, String, String, String),

    /// A Line runs along an ellipse's tangent at its endpoint `point_id`,
    /// held on the ellipse (or elliptical arc) by another constraint; built
    /// only by `constraint_catalog::tangent_at_held_endpoints`.
    /// (point_id, other_line_end_id, ellipse_center_id, ellipse_focus1_id,
    /// ellipse_or_elliptical_arc_id)
    TangentLineEllipseAtPoint(String, String, String, String, String),

    /// A point is an endpoint (either one) of an ellipse's major or minor
    /// axis.
    /// (point_id, ellipse_center_id, ellipse_focus1_id, ellipse_id, axis)
    EllipseAxisPoint(String, String, String, String, EllipseAxis),

    /// Two points are the two endpoints of an ellipse's major or minor axis
    /// (native two-point `ellipse_axis`).
    /// (p1_id, p2_id, ellipse_center_id, ellipse_focus1_id, ellipse_id, axis)
    EllipseDiameter(String, String, String, String, String, EllipseAxis),

    // ── Elliptical arcs (an ellipse's center, focus and `radmin`, plus
    //    parametric start and end angles) ──
    /// A point lies on an elliptical arc's span: on its ellipse, between its
    /// start and end angles (counter-clockwise, parametric).
    /// (point_id, center_id, focus1_id, elliptical_arc_id)
    PointOnEllipticalArc(String, String, String, String),

    /// A Line (the segment) is tangent to an elliptical arc, touching it on
    /// the segment and on the span.
    /// (line_pa_id, line_pb_id, center_id, focus1_id, elliptical_arc_id)
    TangentLineEllipticalArc(String, String, String, String, String),

    /// A Line runs along an elliptical arc's tangent at one of its endpoints
    /// (the Line's endpoint `point_id` is that endpoint).
    /// (point_id, other_line_end_id, center_id, focus1_id, elliptical_arc_id, which end)
    TangentLineEllipticalArcAtPoint(String, String, String, String, String, ArcEnd),

    /// An Arc and an elliptical arc are tangent at an endpoint they share:
    /// the Arc's radius there is normal to the ellipse.
    /// (point_id, arc_center_id, center_id, focus1_id, elliptical_arc_id, which end)
    TangentArcEllipticalArcAtPoint(String, String, String, String, String, ArcEnd),

    /// A Line tangent to an arc at a Point they share (a line endpoint that
    /// is an arc endpoint): the line is perpendicular to the radius there.
    /// (point_id, other_line_end_id, arc_center_point_id)
    TangentAtPoint(String, String, String),

    // ── Curve–curve tangency ──
    /// Two circles touching inside: dist(centers) = |r1 − r2|.
    /// (c1_center_point_id, c1_id, c2_center_point_id, c2_id)
    TangentCirclesInternal(String, String, String, String),

    /// A circle and an arc touching on the arc's span, externally
    /// (dist(centers) = r1 + r2) or, when `internal`, inside (|r1 − r2|).
    /// (circle_center_point_id, circle_id, arc_center_point_id, arc_id, internal)
    TangentCircleArc(String, String, String, String, bool),

    /// Two arcs touching on both spans, externally or, when `internal`,
    /// inside.
    /// (arc1_center_point_id, arc1_id, arc2_center_point_id, arc2_id, internal)
    TangentArcs(String, String, String, String, bool),

    /// Two arcs tangent at an endpoint they share: radii collinear there,
    /// centers on opposite sides of it (external) or, when `internal`, on
    /// the same side.
    /// (point_id, arc1_center_point_id, arc2_center_point_id, internal)
    TangentArcsAtPoint(String, String, String, bool),

    /// Two Lines lie on one infinite line: both endpoints of Line B lie on
    /// Line A's Extension (the segments need not overlap).
    /// (a_p1_id, a_p2_id, b_p1_id, b_p2_id)
    Collinear(String, String, String, String),

    // ── Dimensions and distances to circles and lines ──
    /// A circle's or arc's diameter: 2r = diameter.
    /// (circle_id, diameter)
    Diameter(String, f64),

    /// An arc's length: r · sweep = length, sweep (end − start) mod 2π in
    /// (0, 2π].
    /// (arc_id, length)
    ArcLength(String, f64),

    /// An arc's sweep, 0 < sweep < 2π (anything else is rejected).
    /// (arc_id, sweep_radians)
    ArcSweep(String, f64),

    /// Gap between a point and a circle: |p − c| − r = distance, or with
    /// `internal`, r − |p − c| = distance.
    /// (point_id, circle_center_point_id, circle_id, distance, internal)
    DistancePointCircle(String, String, String, f64, bool),

    /// Gap between a Line (the segment) and a circle: the distance from the
    /// center to the segment, minus r.
    /// (line_pa_id, line_pb_id, circle_center_point_id, circle_id, distance)
    DistanceLineCircle(String, String, String, String, f64),

    /// Gap between a Line's Extension and a circle: the distance from the
    /// center to the Extension, minus r.
    /// (line_pa_id, line_pb_id, circle_center_point_id, circle_id, distance)
    DistanceExtensionCircle(String, String, String, String, f64),

    /// Gap between two circles: |c1 − c2| − r1 − r2 = distance, or with
    /// `internal`, |r1 − r2| − |c1 − c2| = distance (one inside the other).
    /// (c1_center_point_id, c1_id, c2_center_point_id, c2_id, distance, internal)
    DistanceCircleCircle(String, String, String, String, f64, bool),

    /// Gap between a circle and an arc, measured as between two circles,
    /// with the nearest points on the arc's span (`TangentCircleArc` held
    /// `distance` apart).
    /// (circle_center_point_id, circle_id, arc_center_point_id, arc_id, distance, internal)
    DistanceCircleArc(String, String, String, String, f64, bool),

    /// Gap between two arcs, measured as between two circles, with the
    /// nearest points on both spans (`TangentArcs` held `distance` apart).
    /// (arc1_center_point_id, arc1_id, arc2_center_point_id, arc2_id, distance, internal)
    DistanceArcs(String, String, String, String, f64, bool),

    /// Gap between a point and an arc: `DistancePointCircle`'s gap, with the
    /// nearest point (on the ray from the center through the point) on the
    /// arc's span.
    /// (point_id, arc_center_point_id, arc_id, distance, internal)
    DistancePointArc(String, String, String, f64, bool),

    /// Gap between a Line (the segment) and an arc: `DistanceLineCircle`'s
    /// gap, with the nearest point (on the ray from the center through the
    /// segment's nearest point) on the arc's span.
    /// (line_pa_id, line_pb_id, arc_center_point_id, arc_id, distance)
    DistanceLineArc(String, String, String, String, f64),

    /// Gap between a Line's Extension and an arc: `DistanceExtensionCircle`'s
    /// gap, with the nearest point (on the ray from the center through the
    /// foot of the perpendicular) on the arc's span.
    /// (line_pa_id, line_pb_id, arc_center_point_id, arc_id, distance)
    DistanceExtensionArc(String, String, String, String, f64),

    /// Both endpoints of line b are `distance` from line a's Extension, on
    /// one side: b is parallel to a, `distance` away.
    /// (a_p1_id, a_p2_id, b_p1_id, b_p2_id, distance)
    DistanceLineLine(String, String, String, String, f64),
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
        ConstraintType::MidpointOfLineOnLine(a, b, c, d) => Ok(Box::new(
            crate::constraints::midpoint_line_on_line::MidpointOfLineOnLineConstraint::new(
                a, b, c, d,
            ),
        )),
        ConstraintType::PointOnExtension(p, pa, pb) => Ok(Box::new(
            crate::constraints::point_on_extension::PointOnExtensionConstraint::new(p, pa, pb),
        )),
        ConstraintType::DistancePointExtension(pt, pa, pb, d) => Ok(Box::new(
            crate::constraints::distance_point_extension::DistancePointExtensionConstraint::new(
                pt, pa, pb, d,
            ),
        )),
        ConstraintType::TangentExtensionCircle(pa, pb, c_center, c) => Ok(Box::new(
            crate::constraints::tangent_extension_circle::TangentExtensionCircleConstraint::new(
                pa, pb, c_center, c,
            ),
        )),
        ConstraintType::MidpointOfLineOnExtension(a, b, c, d) => Ok(Box::new(
            crate::constraints::midpoint_line_on_extension::MidpointOfLineOnExtensionConstraint::new(
                a, b, c, d,
            ),
        )),
        ConstraintType::SignedDistancePointExtension(pt, pa, pb, d, side) => Ok(Box::new(
            crate::constraints::signed_distance_point_extension::SignedDistancePointExtensionConstraint::new(
                pt, pa, pb, d, side,
            ),
        )),
        ConstraintType::Difference(a, b, d) => Ok(Box::new(
            crate::constraints::difference::DifferenceConstraint::new(a, b, d),
        )),
        ConstraintType::Equal(a, b) => Ok(Box::new(
            crate::constraints::equal::EqualConstraint::new(a, b),
        )),
        ConstraintType::PointOnArc(p, center, arc) => Ok(Box::new(
            crate::constraints::point_on_arc::PointOnArcConstraint::new(p, center, arc),
        )),
        ConstraintType::MidpointOfArc(p, center, arc) => Ok(Box::new(
            crate::constraints::midpoint_of_arc::MidpointOfArcConstraint::new(p, center, arc),
        )),
        ConstraintType::TangentLineArc(pa, pb, center, arc) => Ok(Box::new(
            crate::constraints::tangent_line_arc::TangentLineArcConstraint::new(
                pa, pb, center, arc,
            ),
        )),
        ConstraintType::TangentAtPoint(p, other, center) => Ok(Box::new(
            crate::constraints::tangent_at_point::TangentAtPointConstraint::new(p, other, center),
        )),
        ConstraintType::TangentCirclesInternal(c1_center, c1, c2_center, c2) => {
            use crate::constraints::tangent_curves::{TangentCurve, TangentCurvesConstraint};
            Ok(Box::new(TangentCurvesConstraint::new(
                TangentCurve::circle(c1_center, c1),
                TangentCurve::circle(c2_center, c2),
                true,
            )))
        }
        ConstraintType::TangentCircleArc(c_center, c, a_center, a, internal) => {
            use crate::constraints::tangent_curves::{TangentCurve, TangentCurvesConstraint};
            Ok(Box::new(TangentCurvesConstraint::new(
                TangentCurve::circle(c_center, c),
                TangentCurve::arc(a_center, a),
                internal,
            )))
        }
        ConstraintType::TangentArcs(a1_center, a1, a2_center, a2, internal) => {
            use crate::constraints::tangent_curves::{TangentCurve, TangentCurvesConstraint};
            Ok(Box::new(TangentCurvesConstraint::new(
                TangentCurve::arc(a1_center, a1),
                TangentCurve::arc(a2_center, a2),
                internal,
            )))
        }
        ConstraintType::TangentArcsAtPoint(p, c1, c2, internal) => Ok(Box::new(
            crate::constraints::tangent_arcs_at_point::TangentArcsAtPointConstraint::new(
                p, c1, c2, internal,
            ),
        )),
        ConstraintType::PointPointAngle(p1, p2, angle) => Ok(Box::new(
            crate::constraints::point_point_angle::PointPointAngleConstraint::new(p1, p2, angle),
        )),
        ConstraintType::MirrorPointExtension(a, b, pa, pb) => Ok(Box::new(
            crate::constraints::mirror_point_extension::MirrorPointExtensionConstraint::new(
                a, b, pa, pb,
            ),
        )),
        ConstraintType::CircularInstance(p0, pk, c, angle) => Ok(Box::new(
            crate::constraints::circular_instance::CircularInstanceConstraint::new(
                p0, pk, c, angle,
            ),
        )),
        ConstraintType::LinearInstance(p0, pk, d1, d2, base, n) => Ok(Box::new(
            crate::constraints::linear_instance::LinearInstanceConstraint::new(
                p0, pk, d1, d2, base, n,
            ),
        )),
        ConstraintType::PointOnEllipse(p, center, focus, e) => Ok(Box::new(
            crate::constraints::point_on_ellipse::PointOnEllipseConstraint::new(
                p, center, focus, e,
            ),
        )),
        ConstraintType::TangentLineEllipse(pa, pb, center, focus, e) => Ok(Box::new(
            crate::constraints::tangent_line_ellipse::TangentLineEllipseConstraint::new(
                pa, pb, center, focus, e,
            ),
        )),
        ConstraintType::TangentLineEllipseAtPoint(p, other, center, focus, e) => Ok(Box::new(
            crate::constraints::tangent_line_ellipse_at_point::TangentLineEllipseAtPointConstraint::new(
                p, other, center, focus, e,
            ),
        )),
        ConstraintType::EllipseAxisPoint(p, center, focus, e, axis) => Ok(Box::new(
            crate::constraints::ellipse_axis_point::EllipseAxisPointConstraint::new(
                p, center, focus, e, axis,
            ),
        )),
        ConstraintType::EllipseDiameter(p1, p2, center, focus, e, axis) => Ok(Box::new(
            crate::constraints::ellipse_diameter::EllipseDiameterConstraint::new(
                p1, p2, center, focus, e, axis,
            ),
        )),
        ConstraintType::PointOnEllipticalArc(p, center, focus, e) => Ok(Box::new(
            crate::constraints::point_on_elliptical_arc::PointOnEllipticalArcConstraint::new(
                p, center, focus, e,
            ),
        )),
        ConstraintType::TangentLineEllipticalArc(pa, pb, center, focus, e) => Ok(Box::new(
            crate::constraints::tangent_line_elliptical_arc::TangentLineEllipticalArcConstraint::new(
                pa, pb, center, focus, e,
            ),
        )),
        ConstraintType::TangentLineEllipticalArcAtPoint(p, other, center, focus, e, end) => {
            Ok(Box::new(
                crate::constraints::tangent_line_elliptical_arc_at_point::TangentLineEllipticalArcAtPointConstraint::new(
                    p, other, center, focus, e, end,
                ),
            ))
        }
        ConstraintType::TangentArcEllipticalArcAtPoint(p, arc_center, center, focus, e, end) => {
            Ok(Box::new(
                crate::constraints::tangent_arc_elliptical_arc_at_point::TangentArcEllipticalArcAtPointConstraint::new(
                    p, arc_center, center, focus, e, end,
                ),
            ))
        }
        ConstraintType::Collinear(a1, a2, b1, b2) => Ok(Box::new(
            crate::constraints::collinear::CollinearConstraint::new(a1, a2, b1, b2),
        )),
        ConstraintType::Diameter(c, d) => Ok(Box::new(
            crate::constraints::diameter::DiameterConstraint::new(c, d),
        )),
        ConstraintType::ArcLength(arc, length) => Ok(Box::new(
            crate::constraints::arc_length::ArcLengthConstraint::new(arc, length),
        )),
        ConstraintType::ArcSweep(arc, sweep) => {
            if !crate::constraints::arc_sweep::ArcSweepConstraint::accepts(sweep) {
                return Err(format!("arc sweep {sweep} is not between 0 and 2π"));
            }
            Ok(Box::new(
                crate::constraints::arc_sweep::ArcSweepConstraint::new(arc, sweep),
            ))
        }
        ConstraintType::DistancePointCircle(p, center, c, d, internal) => Ok(Box::new(
            crate::constraints::distance_point_circle::DistancePointCircleConstraint::new(
                p, center, c, d, internal,
            ),
        )),
        ConstraintType::DistanceLineCircle(pa, pb, center, c, d) => Ok(Box::new(
            crate::constraints::distance_line_circle::DistanceLineCircleConstraint::new(
                pa, pb, center, c, d,
            ),
        )),
        ConstraintType::DistanceExtensionCircle(pa, pb, center, c, d) => Ok(Box::new(
            crate::constraints::distance_extension_circle::DistanceExtensionCircleConstraint::new(
                pa, pb, center, c, d,
            ),
        )),
        ConstraintType::DistanceCircleCircle(c1_center, c1, c2_center, c2, d, internal) => {
            Ok(Box::new(
                crate::constraints::distance_circle_circle::DistanceCircleCircleConstraint::new(
                    c1_center, c1, c2_center, c2, d, internal,
                ),
            ))
        }
        ConstraintType::DistanceCircleArc(c_center, c, a_center, a, d, internal) => {
            use crate::constraints::tangent_curves::{TangentCurve, TangentCurvesConstraint};
            Ok(Box::new(
                TangentCurvesConstraint::new(
                    TangentCurve::circle(c_center, c),
                    TangentCurve::arc(a_center, a),
                    internal,
                )
                .with_gap(d),
            ))
        }
        ConstraintType::DistanceArcs(a1_center, a1, a2_center, a2, d, internal) => {
            use crate::constraints::tangent_curves::{TangentCurve, TangentCurvesConstraint};
            Ok(Box::new(
                TangentCurvesConstraint::new(
                    TangentCurve::arc(a1_center, a1),
                    TangentCurve::arc(a2_center, a2),
                    internal,
                )
                .with_gap(d),
            ))
        }
        ConstraintType::DistancePointArc(p, center, arc, d, internal) => Ok(Box::new(
            crate::constraints::distance_point_arc::DistancePointArcConstraint::new(
                p, center, arc, d, internal,
            ),
        )),
        ConstraintType::DistanceLineArc(pa, pb, center, arc, d) => Ok(Box::new(
            crate::constraints::distance_line_arc::DistanceLineArcConstraint::new(
                pa, pb, center, arc, d, false,
            ),
        )),
        ConstraintType::DistanceExtensionArc(pa, pb, center, arc, d) => Ok(Box::new(
            crate::constraints::distance_line_arc::DistanceLineArcConstraint::new(
                pa, pb, center, arc, d, true,
            ),
        )),
        ConstraintType::DistanceLineLine(a1, a2, b1, b2, d) => {
            // At 0 the side-free residual |d| − value only touches zero:
            // two Lines at distance 0 are `collinear`.
            if d <= 0.0 {
                return Err(format!(
                    "distance {d} between lines is not positive (use collinear for 0)"
                ));
            }
            Ok(Box::new(
                crate::constraints::distance_line_line::DistanceLineLineConstraint::new(
                    a1, a2, b1, b2, d,
                ),
            ))
        }
    }
}
