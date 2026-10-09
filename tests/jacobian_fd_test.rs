//! Checks every constraint's analytical Jacobian against central finite
//! differences of its residual, at several generic configurations.
//!
//! A constraint's Jacobian has partials for every variable it reads, Guides
//! included (only a drag holds a Guide), so the differences perturb every
//! occurrence of each variable, and a variable it doesn't read must have a
//! zero column.

use acs::constraint_catalog::{
    Args, ConstraintSpec, EllipseRef, FieldKind, specs, tangent_at_held_endpoints,
};
use acs::constraints::{
    Constraint, ConstraintType, EllipseAxis, Operand, SplineAt, create_constraint, reads,
};
use nalgebra::DMatrix;
use acs::geometry::{Arc, Circle, CurveParam, Ellipse, EllipticalArc, Point, Spline};
use acs::var_registry::{EntityType, VarRegistry};

const POINTS: [&str; 6] = ["p0", "p1", "p2", "p3", "p4", "p5"];

/// Small deterministic generator so failures are reproducible.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next()
    }
}

fn build_pm(seed: u64) -> VarRegistry {
    let mut rng = Lcg(seed);
    let mut pm = VarRegistry::new();
    for id in POINTS.iter().chain(["c1_center", "c2_center"].iter()) {
        let (x, y) = (rng.range(-5.0, 5.0), rng.range(-5.0, 5.0));
        pm.register_entity(
            id.to_string(),
            EntityType::Point,
            &Point::new(id.to_string(), x, y, false),
        );
    }
    for (id, center) in [("c1", "c1_center"), ("c2", "c2_center")] {
        let r = rng.range(0.5, 3.0);
        pm.register_entity(
            id.to_string(),
            EntityType::Circle,
            &Circle::new(id.to_string(), center.to_string(), r, false),
        );
    }
    // Arcs, each with its own center and endpoint Points. Sweeps are short
    // so points and lines often fall outside the span.
    for (id, center, start, end) in ARCS {
        for p in [center, start, end] {
            let (x, y) = (rng.range(-5.0, 5.0), rng.range(-5.0, 5.0));
            pm.register_entity(
                p.to_string(),
                EntityType::Point,
                &Point::new(p.to_string(), x, y, false),
            );
        }
        let r = rng.range(0.5, 3.0);
        let a0 = rng.range(-4.0, 4.0);
        let a1 = a0 + rng.range(0.3, 1.5);
        pm.register_entity(
            id.to_string(),
            EntityType::Arc,
            &Arc::new(
                id.to_string(),
                center.to_string(),
                start.to_string(),
                end.to_string(),
                r,
                a0,
                a1,
                false,
            ),
        );
    }
    // Ellipses, each with its own center and focus Points.
    for (id, center, focus) in ELLIPSES {
        for p in [center, focus] {
            let (x, y) = (rng.range(-5.0, 5.0), rng.range(-5.0, 5.0));
            pm.register_entity(
                p.to_string(),
                EntityType::Point,
                &Point::new(p.to_string(), x, y, false),
            );
        }
        let b = rng.range(0.5, 3.0);
        pm.register_entity(
            id.to_string(),
            EntityType::Ellipse,
            &Ellipse::new(id.to_string(), center.to_string(), focus.to_string(), b, false),
        );
    }
    // Elliptical arcs, each with its own center, focus and endpoint Points.
    // Sweeps are short so points and tangency points often fall outside.
    for (id, center, focus, start, end) in ELLIPTICAL_ARCS {
        for p in [center, focus, start, end] {
            let (x, y) = (rng.range(-5.0, 5.0), rng.range(-5.0, 5.0));
            pm.register_entity(
                p.to_string(),
                EntityType::Point,
                &Point::new(p.to_string(), x, y, false),
            );
        }
        let b = rng.range(0.5, 3.0);
        let a0 = rng.range(-4.0, 4.0);
        let a1 = a0 + rng.range(0.3, 1.5);
        pm.register_entity(
            id.to_string(),
            EntityType::EllipticalArc,
            &EllipticalArc::new(
                id.to_string(),
                center.to_string(),
                focus.to_string(),
                start.to_string(),
                end.to_string(),
                b,
                a0,
                a1,
                false,
            ),
        );
    }
    // Splines' handles, and the curve parameters (`"#0"`, `"#1"`: a
    // catalog-built constraint with no JSON id names its own so) inside
    // their domains.
    for (_, handles) in SPLINES {
        for p in handles {
            let (x, y) = (rng.range(-5.0, 5.0), rng.range(-5.0, 5.0));
            pm.register_entity(p.to_string(), EntityType::Point, &Point::new(p.to_string(), x, y, false));
        }
    }
    for id in PARAMS {
        let value = rng.range(0.15, 0.85);
        pm.register_entity(id.to_string(), EntityType::CurveParam, &CurveParam { id: id.to_string(), value });
    }
    pm
}

/// (spline, handle Points) of the splines in `build_pm`.
const SPLINES: [(&str, &[&str]); 2] = [
    ("sp1", &["sp1_0", "sp1_1", "sp1_2", "sp1_3", "sp1_4"]),
    ("sp2", &["sp2_0", "sp2_1", "sp2_2", "sp2_3"]),
];

/// The curve parameters in `build_pm`.
const PARAMS: [&str; 2] = ["#0", "#1"];

/// Spline `i` of `build_pm`, fit points or control points.
fn spline(i: usize, interpolated: bool) -> Spline {
    let (id, handles) = SPLINES[i];
    Spline::new(id.into(), handles.iter().map(|h| h.to_string()).collect(), interpolated, None)
}

/// (elliptical arc, center, focus, start, end) IDs of the elliptical arcs in
/// `build_pm`.
const ELLIPTICAL_ARCS: [(&str, &str, &str, &str, &str); 1] =
    [("ea1", "ea1_center", "ea1_focus", "ea1_start", "ea1_end")];

/// (ellipse, center, focus) IDs of the ellipses in `build_pm`.
const ELLIPSES: [(&str, &str, &str); 2] = [
    ("e1", "e1_center", "e1_focus"),
    ("e2", "e2_center", "e2_focus"),
];

/// (arc, center, start, end) Point IDs of the arcs in `build_pm`.
const ARCS: [(&str, &str, &str, &str); 2] = [
    ("a1", "a1_center", "a1_start", "a1_end"),
    ("a2", "a2_center", "a2_start", "a2_end"),
];

/// One constraint per internal `ConstraintType` variant, built from the
/// first catalog row that reaches it,
/// with fields filled from the entities in `build_pm`: distinct points, the
/// two circles, the two arcs and a generic scalar. Value fields take, in
/// turn, a radius, a point coordinate and a constant.
fn all_constraints() -> Vec<ConstraintType> {
    let mut seen = std::collections::HashSet::new();
    let built: Vec<ConstraintType> = specs().iter().flat_map(build_from_spec).collect();
    let rewritten: Vec<ConstraintType> = built.iter().filter_map(held_endpoint_rewrite).collect();
    // The rows build only fit-point Splines; every form, end and shared case
    // is checked in `spline_jacobians_match_finite_differences`.
    built
        .into_iter()
        .chain(rewritten)
        .filter(|ct| seen.insert(variant_name(ct)))
        .collect()
}

/// A catalog-built Line–ellipse or Line–elliptical arc tangency with its
/// first endpoint held `on` the curve, as `tangent_at_held_endpoints`
/// rewrites it in a sketch (the only way to reach the kernel it builds).
fn held_endpoint_rewrite(ct: &ConstraintType) -> Option<ConstraintType> {
    let on = match ct {
        ConstraintType::TangentLineEllipse(p, _, c, f, e) => {
            ConstraintType::PointOnEllipse(p.clone(), c.clone(), f.clone(), e.clone())
        }
        ConstraintType::TangentLineEllipticalArc(p, _, c, f, e) => {
            ConstraintType::PointOnEllipticalArc(p.clone(), c.clone(), f.clone(), e.clone())
        }
        ConstraintType::TangentLineSpline(p, _, s, _) => {
            ConstraintType::PointOnSpline(p.clone(), s.clone(), "#1".into())
        }
        _ => return None,
    };
    let mut pair = [(ct.clone(), false), (on, false)];
    tangent_at_held_endpoints(&mut pair);
    let [(rewritten, _), _] = pair;
    Some(rewritten)
}

fn args_for(spec: &ConstraintSpec) -> Args {
    let mut args = Args::default();
    let mut next_point = POINTS.iter();
    let mut next_circle = [("c1", "c1_center"), ("c2", "c2_center")].iter();
    let mut next_operand = [
        Operand::Radius("c1".into()),
        Operand::Y("p5".into()),
        Operand::Const(0.4),
    ]
    .into_iter();
    let mut next_arc = ARCS.iter();
    let mut next_ellipse = ELLIPSES.iter();
    let mut next_elliptical_arc = ELLIPTICAL_ARCS.iter();
    for &(_, kind) in spec.fields {
        let points = match kind {
            FieldKind::Point => 1,
            FieldKind::Line => 2,
            _ => 0,
        };
        for _ in 0..points {
            args.points.push(next_point.next().unwrap().to_string());
        }
        if kind == FieldKind::Circle {
            let (c, center) = next_circle.next().unwrap();
            args.circles.push(c.to_string());
            args.centers.push(center.to_string());
        }
        if kind == FieldKind::Arc {
            let (a, center, start, end) = next_arc.next().unwrap();
            args.circles.push(a.to_string());
            args.centers.push(center.to_string());
            args.arc_ends.push((start.to_string(), end.to_string()));
        }
        if kind == FieldKind::Ellipse {
            let (e, center, focus) = next_ellipse.next().unwrap();
            args.ellipses.push(EllipseRef {
                id: e.to_string(),
                center: center.to_string(),
                focus: focus.to_string(),
            });
        }
        if kind == FieldKind::EllipticalArc {
            let (e, center, focus, start, end) = next_elliptical_arc.next().unwrap();
            args.ellipses.push(EllipseRef {
                id: e.to_string(),
                center: center.to_string(),
                focus: focus.to_string(),
            });
            args.elliptical_arc_ends.push((start.to_string(), end.to_string()));
        }
        if kind == FieldKind::Axis {
            args.axes.push(EllipseAxis::Major);
        }
        if kind == FieldKind::Spline {
            args.splines.push(spline(args.splines.len(), true));
        }
        if kind == FieldKind::Scalar {
            args.scalars.push(1.3);
        }
        if kind == FieldKind::Value {
            args.operands.push(next_operand.next().unwrap());
        }
    }
    args
}

/// Builds `spec` with distinct entities, plus, for a row relating a Line
/// to an Arc, again with the line starting at the arc's start point, and
/// for a row relating two Arcs, again with the second starting where the
/// first starts: a shared point is what selects the angle-at-point form of
/// tangency.
fn build_from_spec(spec: &ConstraintSpec) -> Vec<ConstraintType> {
    let args = args_for(spec);
    let mut out = vec![spec.build(&args)];
    let kinds: Vec<FieldKind> = spec.fields.iter().map(|&(_, k)| k).collect();
    if kinds.contains(&FieldKind::Line) && kinds.contains(&FieldKind::Arc) {
        let mut shared = args.clone();
        shared.points[0] = shared.arc_ends[0].0.clone();
        out.push(spec.build(&shared));
    }
    if kinds.iter().filter(|&&k| k == FieldKind::Arc).count() == 2 {
        let mut shared = args.clone();
        shared.arc_ends[1].0 = shared.arc_ends[0].0.clone();
        out.push(spec.build(&shared));
    }
    // A Line or an Arc sharing the spline's start, and two splines sharing
    // an end: the at-point forms.
    if kinds.contains(&FieldKind::Spline) {
        let first = args.splines[0].points[0].clone();
        let mut shared = args.clone();
        if kinds.contains(&FieldKind::Line) {
            shared.points[0] = first;
            out.push(spec.build(&shared));
        } else if kinds.contains(&FieldKind::Arc) {
            shared.arc_ends[0].1 = first;
            out.push(spec.build(&shared));
        } else if kinds.iter().filter(|&&k| k == FieldKind::Spline).count() == 2 {
            shared.splines[1].points[0] = first;
            out.push(spec.build(&shared));
        }
    }
    // A Line or an Arc sharing the elliptical arc's start, then its end.
    if kinds.contains(&FieldKind::EllipticalArc)
        && (kinds.contains(&FieldKind::Line) || kinds.contains(&FieldKind::Arc))
    {
        let (start, end) = args.elliptical_arc_ends[0].clone();
        for p in [start, end] {
            let mut shared = args.clone();
            if kinds.contains(&FieldKind::Line) {
                shared.points[0] = p;
            } else {
                shared.arc_ends[0].0 = p;
            }
            out.push(spec.build(&shared));
        }
    }
    out
}

/// Exhaustive on purpose: a new `ConstraintType` variant fails to compile
/// here until it is listed, and the test below then requires a catalog row
/// that reaches it.
fn variant_name(ct: &ConstraintType) -> &'static str {
    use ConstraintType::*;
    match ct {
        Vertical(..) => "Vertical",
        Horizontal(..) => "Horizontal",
        Parallel(..) => "Parallel",
        EqualX(..) => "EqualX",
        EqualY(..) => "EqualY",
        Coincident(..) => "Coincident",
        PointOnLine(..) => "PointOnLine",
        EqualRadius(..) => "EqualRadius",
        Perpendicular(..) => "Perpendicular",
        FixedRadius(..) => "FixedRadius",
        PointOnCircle(..) => "PointOnCircle",
        Tangent(..) => "Tangent",
        TangentLineCircle(..) => "TangentLineCircle",
        Concentric(..) => "Concentric",
        DistancePointPoint(..) => "DistancePointPoint",
        DistancePointLine(..) => "DistancePointLine",
        Angle(..) => "Angle",
        Midpoint(..) => "Midpoint",
        EqualLength(..) => "EqualLength",
        MidpointOfLineOnLine(..) => "MidpointOfLineOnLine",
        PointOnExtension(..) => "PointOnExtension",
        DistancePointExtension(..) => "DistancePointExtension",
        TangentExtensionCircle(..) => "TangentExtensionCircle",
        MidpointOfLineOnExtension(..) => "MidpointOfLineOnExtension",
        SignedDistancePointExtension(..) => "SignedDistancePointExtension",
        Difference(..) => "Difference",
        Equal(..) => "Equal",
        PointOnArc(..) => "PointOnArc",
        MidpointOfArc(..) => "MidpointOfArc",
        TangentLineArc(..) => "TangentLineArc",
        TangentAtPoint(..) => "TangentAtPoint",
        PointPointAngle(..) => "PointPointAngle",
        MirrorPointExtension(..) => "MirrorPointExtension",
        CircularInstance(..) => "CircularInstance",
        LinearInstance(..) => "LinearInstance",
        PointOnEllipse(..) => "PointOnEllipse",
        TangentLineEllipse(..) => "TangentLineEllipse",
        TangentLineEllipseAtPoint(..) => "TangentLineEllipseAtPoint",
        EllipseAxisPoint(..) => "EllipseAxisPoint",
        EllipseDiameter(..) => "EllipseDiameter",
        PointOnEllipticalArc(..) => "PointOnEllipticalArc",
        TangentLineEllipticalArc(..) => "TangentLineEllipticalArc",
        TangentLineEllipticalArcAtPoint(..) => "TangentLineEllipticalArcAtPoint",
        TangentArcEllipticalArcAtPoint(..) => "TangentArcEllipticalArcAtPoint",
        TangentCirclesInternal(..) => "TangentCirclesInternal",
        TangentCircleArc(..) => "TangentCircleArc",
        TangentArcs(..) => "TangentArcs",
        TangentArcsAtPoint(..) => "TangentArcsAtPoint",
        Collinear(..) => "Collinear",
        Diameter(..) => "Diameter",
        ArcLength(..) => "ArcLength",
        ArcSweep(..) => "ArcSweep",
        DistancePointCircle(..) => "DistancePointCircle",
        DistanceLineCircle(..) => "DistanceLineCircle",
        DistanceExtensionCircle(..) => "DistanceExtensionCircle",
        DistanceCircleCircle(..) => "DistanceCircleCircle",
        DistancePointArc(..) => "DistancePointArc",
        DistanceLineArc(..) => "DistanceLineArc",
        DistanceExtensionArc(..) => "DistanceExtensionArc",
        DistanceCircleArc(..) => "DistanceCircleArc",
        DistanceArcs(..) => "DistanceArcs",
        DistanceLineLine(..) => "DistanceLineLine",
        PointOnSpline(..) => "PointOnSpline",
        TangentLineSpline(..) => "TangentLineSpline",
        TangentCircleSpline(..) => "TangentCircleSpline",
        TangentArcSpline(..) => "TangentArcSpline",
        TangentEllipseSpline(..) => "TangentEllipseSpline",
        TangentSplines(..) => "TangentSplines",
        TangentLineSplineAtPoint(..) => "TangentLineSplineAtPoint",
        TangentArcSplineAtPoint(..) => "TangentArcSplineAtPoint",
        TangentSplinesAtPoint(..) => "TangentSplinesAtPoint",
    }
}
const ALL_VARIANTS: [&str; 71] = [
    "Vertical",
    "Horizontal",
    "Parallel",
    "EqualX",
    "EqualY",
    "Coincident",
    "PointOnLine",
    "EqualRadius",
    "Perpendicular",
    "FixedRadius",
    "PointOnCircle",
    "Tangent",
    "TangentLineCircle",
    "Concentric",
    "DistancePointPoint",
    "DistancePointLine",
    "Angle",
    "Midpoint",
    "EqualLength",
    "MidpointOfLineOnLine",
    "PointOnExtension",
    "DistancePointExtension",
    "TangentExtensionCircle",
    "MidpointOfLineOnExtension",
    "SignedDistancePointExtension",
    "Difference",
    "Equal",
    "PointOnArc",
    "MidpointOfArc",
    "TangentLineArc",
    "TangentAtPoint",
    "PointPointAngle",
    "MirrorPointExtension",
    "CircularInstance",
    "LinearInstance",
    "PointOnEllipse",
    "TangentLineEllipse",
    "TangentLineEllipseAtPoint",
    "EllipseAxisPoint",
    "EllipseDiameter",
    "PointOnEllipticalArc",
    "TangentLineEllipticalArc",
    "TangentLineEllipticalArcAtPoint",
    "TangentArcEllipticalArcAtPoint",
    "TangentCirclesInternal",
    "TangentCircleArc",
    "TangentArcs",
    "TangentArcsAtPoint",
    "Collinear",
    "Diameter",
    "ArcLength",
    "ArcSweep",
    "DistancePointCircle",
    "DistanceLineCircle",
    "DistanceExtensionCircle",
    "DistanceCircleCircle",
    "DistancePointArc",
    "DistanceLineArc",
    "DistanceExtensionArc",
    "DistanceCircleArc",
    "DistanceArcs",
    "DistanceLineLine",
    "PointOnSpline",
    "TangentLineSpline",
    "TangentCircleSpline",
    "TangentArcSpline",
    "TangentEllipseSpline",
    "TangentSplines",
    "TangentLineSplineAtPoint",
    "TangentArcSplineAtPoint",
    "TangentSplinesAtPoint",
];

#[test]
fn every_constraint_variant_is_reached_by_a_catalog_row() {
    let covered: std::collections::HashSet<&str> =
        all_constraints().iter().map(variant_name).collect();
    for v in ALL_VARIANTS {
        assert!(
            covered.contains(v),
            "no catalog row builds {v}"
        );
    }
}

/// Residuals of `c` at local values `x` (as `eval` receives them).
fn local_residual(c: &dyn Constraint, x: &[f64]) -> Vec<f64> {
    let mut r = vec![0.0; c.num_residuals()];
    let mut j = DMatrix::zeros(r.len(), reads(c).len());
    c.eval(x, &mut r, &mut j);
    r
}

/// Returns a description of every Jacobian entry that disagrees with central
/// finite differences over every variable the constraint reads (Guides
/// included: an ordinary solve moves them), or an empty list if all agree. A
/// column of a variable it doesn't read must be zero.
fn fd_mismatches(ct: &ConstraintType, seed: u64) -> Vec<String> {
    fd_mismatches_of(create_constraint(ct.clone()).unwrap().as_ref(), &format!("{ct:?}"), seed)
}

/// [`fd_mismatches`] for any constraint `c`, described by `what`.
fn fd_mismatches_of(c: &dyn Constraint, what: &str, seed: u64) -> Vec<String> {
    let pm = build_pm(seed);
    let jac = c.jacobian(&pm);
    let cols: Vec<usize> = reads(c)
        .iter()
        .map(|v| v.column(&pm).unwrap())
        .collect();
    let x0: Vec<f64> = cols.iter().map(|&k| pm.values()[k]).collect();
    let h = 1e-6;
    let mut out = Vec::new();

    let residual = c.residual(&pm);
    for (r, v) in local_residual(c, &x0).into_iter().enumerate() {
        assert_eq!(residual[r], v, "{what}: residual() disagrees with eval()");
    }

    for i in 0..pm.num_vars() {
        let shifted = |dh: f64| {
            let mut x = x0.clone();
            for k in (0..cols.len()).filter(|&k| cols[k] == i) {
                x[k] += dh;
            }
            local_residual(c, &x)
        };
        let (plus, minus) = (shifted(h), shifted(-h));
        let is_read = cols.contains(&i);
        let name = &pm.var_info()[i].name;

        for r in 0..c.num_residuals() {
            let fd = (plus[r] - minus[r]) / (2.0 * h);
            let an = jac[(r, i)];
            if !is_read && an != 0.0 {
                out.push(format!(
                    "seed {seed} residual {r} d/d{name}: {an:.6} in a column it doesn't read"
                ));
            } else if (an - fd).abs() > 1e-5 * (1.0 + fd.abs()) {
                out.push(format!(
                    "seed {seed} residual {r} d/d{name}: analytic {an:.6} vs fd {fd:.6}"
                ));
            }
        }
    }
    out
}

/// The mirror's axis, the rotation's center and the translation's direction
/// are Guides (a drag never moves them), and they have partials like any
/// other input, so an ordinary solve can move them (their exactness is
/// checked with every other column by the finite-difference test).
#[test]
fn array_and_mirror_guides_are_declared_and_have_partials() {
    let s = |x: &str| x.to_string();
    for (ct, guides) in [
        (
            ConstraintType::MirrorPointExtension(s("p0"), s("p1"), s("p2"), s("p3")),
            vec!["p2", "p3"],
        ),
        (
            ConstraintType::CircularInstance(s("p0"), s("p1"), s("p2"), 0.5),
            vec!["p2"],
        ),
        (
            ConstraintType::LinearInstance(s("p0"), s("p1"), s("p2"), s("p3"), 1.0, 2.0),
            vec!["p2", "p3"],
        ),
    ] {
        let c = create_constraint(ct.clone()).unwrap();
        let declared = c.guides();
        let mut ids: Vec<&str> = declared.iter().map(|v| v.entity_id()).collect();
        ids.dedup();
        assert_eq!(ids, guides, "{ct:?}");

        let mut pm = build_pm(7);
        let jac = c.jacobian(&pm);
        for v in c.guides() {
            let col = v.column(&pm).unwrap();
            assert!(jac.column(col).iter().any(|&d| d != 0.0), "{ct:?}: {v:?} has no partials");
            let before = c.residual(&pm);
            pm.values_mut()[col] += 0.1;
            assert_ne!(c.residual(&pm), before, "{ct:?}: {v:?} doesn't affect the residual");
        }
    }
}

/// Constraints whose arguments share a point, as in two lines meeting at a
/// corner. A Jacobian written with `=` instead of `+=` is wrong only here.
fn shared_point_constraints() -> Vec<ConstraintType> {
    let s = |x: &str| x.to_string();
    vec![
        ConstraintType::Parallel(s("p0"), s("p1"), s("p1"), s("p2")),
        ConstraintType::Perpendicular(s("p0"), s("p1"), s("p1"), s("p2")),
        ConstraintType::Angle(s("p0"), s("p1"), s("p1"), s("p2"), 0.7),
        ConstraintType::EqualLength(s("p0"), s("p1"), s("p1"), s("p2")),
        ConstraintType::MidpointOfLineOnLine(s("p0"), s("p1"), s("p1"), s("p2")),
        ConstraintType::DistancePointLine(s("p0"), s("p1"), s("p0"), 1.0),
        ConstraintType::PointOnLine(s("p2"), s("p1"), s("p2")),
        ConstraintType::MidpointOfLineOnExtension(s("p0"), s("p1"), s("p1"), s("p2")),
        ConstraintType::DistancePointExtension(s("p0"), s("p1"), s("p0"), 1.0),
        ConstraintType::DistancePointExtension(s("p0"), s("p1"), s("p0"), 0.0),
        ConstraintType::PointOnExtension(s("p2"), s("p1"), s("p2")),
        ConstraintType::Collinear(s("p0"), s("p1"), s("p1"), s("p2")),
        ConstraintType::Collinear(s("p0"), s("p1"), s("p2"), s("p0")),
        ConstraintType::PointOnExtension(s("c1_center"), s("c1_center"), s("p1")),
        ConstraintType::TangentExtensionCircle(s("c1_center"), s("p1"), s("c1_center"), s("c1")),
        ConstraintType::Midpoint(s("p0"), s("p0"), s("p1")),
        ConstraintType::TangentLineCircle(s("c1_center"), s("p1"), s("c1_center"), s("c1")),
        ConstraintType::Tangent(s("c1_center"), s("c1"), s("c1_center"), s("c2")),
        ConstraintType::PointOnCircle(s("c1_center"), s("c1_center"), s("c1")),
        ConstraintType::EqualRadius(s("c1"), s("c1")),
        ConstraintType::Coincident(s("p0"), s("p0")),
        ConstraintType::DistancePointPoint(s("p0"), s("p0"), 2.0),
        ConstraintType::Horizontal(s("p0"), s("p0")),
        ConstraintType::Vertical(s("p0"), s("p0")),
        ConstraintType::SignedDistancePointExtension(s("p0"), s("p1"), s("p0"), 1.0, 1.0),
        ConstraintType::SignedDistancePointExtension(s("p2"), s("p1"), s("p2"), 0.5, -1.0),
        ConstraintType::Difference(
            Operand::Radius(s("c1")),
            Operand::Radius(s("c1")),
            Operand::Radius(s("c1")),
        ),
        ConstraintType::Difference(
            Operand::X(s("p0")),
            Operand::Const(2.0),
            Operand::X(s("p0")),
        ),
        ConstraintType::Equal(Operand::Radius(s("c1")), Operand::Radius(s("c1"))),
        ConstraintType::Equal(Operand::Const(1.5), Operand::Y(s("p0"))),
        ConstraintType::PointOnArc(s("a1_start"), s("a1_center"), s("a1")),
        ConstraintType::PointOnArc(s("p0"), s("a1_center"), s("a1")),
        // Midpoint of an arc: the point being the arc's own start, end or
        // center, and the other arc.
        ConstraintType::MidpointOfArc(s("a1_start"), s("a1_center"), s("a1")),
        ConstraintType::MidpointOfArc(s("a1_end"), s("a1_center"), s("a1")),
        ConstraintType::MidpointOfArc(s("a1_center"), s("a1_center"), s("a1")),
        ConstraintType::MidpointOfArc(s("p0"), s("a2_center"), s("a2")),
        ConstraintType::TangentLineArc(s("a1_center"), s("p1"), s("a1_center"), s("a1")),
        ConstraintType::TangentLineArc(s("a1_start"), s("p1"), s("a1_center"), s("a1")),
        ConstraintType::TangentLineArc(s("p0"), s("p1"), s("a2_center"), s("a2")),
        ConstraintType::TangentAtPoint(s("a1_start"), s("p1"), s("a1_center")),
        ConstraintType::TangentAtPoint(s("p0"), s("p0"), s("a1_center")),
        ConstraintType::TangentAtPoint(s("p0"), s("p1"), s("p0")),
        // Curve–curve tangency: inside as well as external (the catalog
        // reaches each variant first with `internal: false`), a circle and
        // an arc on one center, and joined arcs sharing a center.
        ConstraintType::TangentCircleArc(s("c1_center"), s("c1"), s("a1_center"), s("a1"), true),
        ConstraintType::TangentCircleArc(s("a1_center"), s("c1"), s("a1_center"), s("a1"), false),
        ConstraintType::TangentArcs(s("a1_center"), s("a1"), s("a2_center"), s("a2"), true),
        ConstraintType::TangentArcs(s("a2_center"), s("a1"), s("a1_center"), s("a2"), false),
        ConstraintType::TangentArcsAtPoint(s("a1_start"), s("a1_center"), s("a2_center"), true),
        ConstraintType::TangentArcsAtPoint(s("a1_end"), s("a1_center"), s("a2_center"), false),
        ConstraintType::TangentArcsAtPoint(s("p0"), s("p0"), s("a2_center"), false),
        ConstraintType::TangentArcsAtPoint(s("p0"), s("a1_center"), s("a1_center"), true),
        ConstraintType::PointPointAngle(s("p0"), s("p0"), 0.7),
        ConstraintType::PointPointAngle(s("p0"), s("p1"), -2.5),
        ConstraintType::MirrorPointExtension(s("p0"), s("p0"), s("p1"), s("p2")),
        ConstraintType::MirrorPointExtension(s("p0"), s("p1"), s("p0"), s("p2")),
        ConstraintType::MirrorPointExtension(s("p0"), s("p1"), s("p2"), s("p1")),
        ConstraintType::CircularInstance(s("p0"), s("p1"), s("p0"), 2.1),
        ConstraintType::CircularInstance(s("p0"), s("p0"), s("p1"), 2.1),
        ConstraintType::CircularInstance(s("p0"), s("p1"), s("p1"), -0.9),
        ConstraintType::LinearInstance(s("p0"), s("p1"), s("p0"), s("p2"), 1.5, 2.0),
        ConstraintType::LinearInstance(s("p0"), s("p1"), s("p2"), s("p1"), 1.5, 3.0),
        ConstraintType::LinearInstance(s("p0"), s("p0"), s("p1"), s("p2"), -0.8, 1.0),
        // Ellipses: the other axis and focus/center sharing. Lines through
        // an ellipse's center or focus, points at its center or focus.
        ConstraintType::PointOnEllipse(s("e1_focus"), s("e1_center"), s("e1_focus"), s("e1")),
        ConstraintType::PointOnEllipse(s("e1_center"), s("e1_center"), s("e1_focus"), s("e1")),
        ConstraintType::PointOnEllipse(s("p0"), s("e1_center"), s("e1_center"), s("e1")),
        ConstraintType::TangentLineEllipse(s("e1_center"), s("p1"), s("e1_center"), s("e1_focus"), s("e1")),
        ConstraintType::TangentLineEllipse(s("p0"), s("e1_focus"), s("e1_center"), s("e1_focus"), s("e1")),
        ConstraintType::TangentLineEllipse(s("p0"), s("p1"), s("e2_center"), s("e2_focus"), s("e2")),
        ConstraintType::TangentLineEllipse(s("p2"), s("p3"), s("e1_center"), s("e1_focus"), s("e1")),
        ConstraintType::TangentLineEllipseAtPoint(s("e1_focus"), s("p1"), s("e1_center"), s("e1_focus"), s("e1")),
        ConstraintType::TangentLineEllipseAtPoint(s("p0"), s("e1_center"), s("e1_center"), s("e1_focus"), s("e1")),
        ConstraintType::TangentLineEllipseAtPoint(s("p0"), s("p0"), s("e2_center"), s("e2_focus"), s("e2")),
        ConstraintType::TangentLineEllipseAtPoint(s("ea1_start"), s("p1"), s("ea1_center"), s("ea1_focus"), s("ea1")),
        ConstraintType::EllipseAxisPoint(s("p0"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Minor),
        ConstraintType::EllipseAxisPoint(s("e1_focus"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Major),
        ConstraintType::EllipseAxisPoint(s("e2_focus"), s("e2_center"), s("e2_focus"), s("e2"), EllipseAxis::Minor),
        ConstraintType::EllipseDiameter(s("p0"), s("p1"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Minor),
        ConstraintType::EllipseDiameter(s("p0"), s("p0"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Major),
        ConstraintType::EllipseDiameter(s("e1_focus"), s("p1"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Major),
        ConstraintType::EllipseDiameter(s("p0"), s("e2_center"), s("e2_center"), s("e2_focus"), s("e2"), EllipseAxis::Minor),
        ConstraintType::Equal(Operand::MinorRadius(s("e1")), Operand::MinorRadius(s("e2"))),
        // Dimensions and distances: the internal variants (the catalog
        // builds the outside ones first), shared points and both circles.
        ConstraintType::DistancePointCircle(s("p0"), s("c1_center"), s("c1"), 0.5, true),
        ConstraintType::DistancePointCircle(s("c2_center"), s("c1_center"), s("c1"), 0.5, false),
        ConstraintType::DistanceCircleCircle(s("c1_center"), s("c1"), s("c2_center"), s("c2"), 0.5, true),
        ConstraintType::DistanceCircleCircle(s("c2_center"), s("c2"), s("c1_center"), s("c1"), 0.5, true),
        ConstraintType::DistanceCircleCircle(s("c1_center"), s("c1"), s("c1_center"), s("c2"), 0.5, false),
        ConstraintType::DistanceCircleArc(s("c1_center"), s("c1"), s("a1_center"), s("a1"), 0.5, true),
        ConstraintType::DistanceArcs(s("a1_center"), s("a1"), s("a2_center"), s("a2"), 0.5, true),
        ConstraintType::DistancePointArc(s("p0"), s("a1_center"), s("a1"), 0.5, true),
        ConstraintType::DistancePointArc(s("a1_start"), s("a1_center"), s("a1"), 0.5, false),
        ConstraintType::DistancePointArc(s("a1_end"), s("a1_center"), s("a1"), 0.5, true),
        ConstraintType::DistancePointArc(s("a1_center"), s("a1_center"), s("a1"), 0.5, false),
        ConstraintType::DistancePointArc(s("a2_center"), s("a1_center"), s("a1"), 0.5, false),
        ConstraintType::DistanceLineArc(s("a1_center"), s("p1"), s("a1_center"), s("a1"), 1.0),
        ConstraintType::DistanceLineArc(s("a1_end"), s("a1_start"), s("a1_center"), s("a1"), 1.0),
        ConstraintType::DistanceLineArc(s("p0"), s("p1"), s("a2_center"), s("a2"), 1.0),
        ConstraintType::DistanceExtensionArc(s("a1_start"), s("p1"), s("a1_center"), s("a1"), 1.0),
        ConstraintType::DistanceExtensionArc(s("p0"), s("p1"), s("a2_center"), s("a2"), 1.0),
        ConstraintType::DistanceLineCircle(s("c1_center"), s("p1"), s("c1_center"), s("c1"), 1.0),
        ConstraintType::DistanceLineCircle(s("p0"), s("p1"), s("c1_center"), s("c1"), 1.0),
        ConstraintType::DistanceExtensionCircle(s("c1_center"), s("p1"), s("c1_center"), s("c1"), 1.0),
        ConstraintType::DistanceLineLine(s("p0"), s("p1"), s("p1"), s("p2"), 1.0),
        ConstraintType::DistanceLineLine(s("p0"), s("p1"), s("p2"), s("p0"), 1.0),
        ConstraintType::ArcLength(s("a2"), 2.0),
        ConstraintType::ArcSweep(s("a2"), 5.9),
        ConstraintType::Difference(
            Operand::MinorRadius(s("e1")),
            Operand::Radius(s("c1")),
            Operand::MinorRadius(s("e1")),
        ),
        // `horizontal_distance`/`vertical_distance`: a signed coordinate
        // difference, also between a point and itself.
        ConstraintType::Difference(Operand::X(s("p0")), Operand::X(s("p1")), Operand::Const(-1.5)),
        ConstraintType::Difference(Operand::Y(s("p0")), Operand::Y(s("p0")), Operand::Const(0.0)),
    ]
}

#[test]
fn every_constraint_jacobian_matches_finite_differences() {
    let mut failures = Vec::new();
    for ct in all_constraints()
        .into_iter()
        .chain(shared_point_constraints())
    {
        for seed in [1, 2, 3, 42, 1234] {
            for m in fd_mismatches(&ct, seed) {
                failures.push(format!("{ct:?}: {m}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} Jacobian mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// An Arc's implicit rules (no sketch constraint builds them), including
/// the shared-point cases.
#[test]
fn arc_rules_jacobian_matches_finite_differences() {
    use acs::constraints::arc_rules::ArcRulesConstraint;
    let s = |x: &str| x.to_string();
    let mut failures = Vec::new();
    for (center, start, end) in [
        ("a1_center", "a1_start", "a1_end"),
        ("a1_center", "a1_start", "a1_start"),
        ("a1_start", "a1_start", "a1_end"),
    ] {
        let c = ArcRulesConstraint::new(s(center), s(start), s(end), s("a1"));
        let what = format!("ArcRules({center}, {start}, {end})");
        for seed in [1, 2, 3, 42, 1234] {
            failures.extend(fd_mismatches_of(&c, &what, seed).into_iter().map(|m| format!("{what}: {m}")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn elliptical_arc_rules_jacobian_matches_finite_differences() {
    use acs::constraints::elliptical_arc_rules::EllipticalArcRulesConstraint;
    let s = |x: &str| x.to_string();
    let mut failures = Vec::new();
    for (center, focus, start, end) in [
        ("ea1_center", "ea1_focus", "ea1_start", "ea1_end"),
        ("ea1_center", "ea1_focus", "ea1_start", "ea1_start"),
        ("ea1_center", "ea1_center", "ea1_start", "ea1_end"),
    ] {
        let c = EllipticalArcRulesConstraint::new(s(center), s(focus), s(start), s(end), s("ea1"));
        let what = format!("EllipticalArcRules({center}, {focus}, {start}, {end})");
        for seed in [1, 2, 3, 42, 1234] {
            failures.extend(fd_mismatches_of(&c, &what, seed).into_iter().map(|m| format!("{what}: {m}")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every Spline kernel in both handle forms (fit points, whose control
/// points come from a linear solve at centripetal parameters, and control
/// points), at either end and at a curve parameter, and with shared Points.
#[test]
fn spline_jacobians_match_finite_differences() {
    let s = |x: &str| x.to_string();
    let mut cases = Vec::new();
    for form in [true, false] {
        let (a, b) = (spline(0, form), spline(1, form));
        let a_start = a.points[0].clone();
        cases.extend([
            ConstraintType::PointOnSpline(s("p0"), a.clone(), s("#0")),
            ConstraintType::PointOnSpline(a.points[2].clone(), a.clone(), s("#0")),
            ConstraintType::TangentLineSpline(s("p0"), s("p1"), a.clone(), s("#0")),
            ConstraintType::TangentLineSpline(a.points[1].clone(), s("p1"), a.clone(), s("#0")),
            ConstraintType::TangentCircleSpline(s("c1_center"), s("c1"), a.clone(), s("#0")),
            ConstraintType::TangentArcSpline(s("a1_center"), s("a1"), a.clone(), s("#0")),
            ConstraintType::TangentEllipseSpline(s("e1_center"), s("e1_focus"), s("e1"), a.clone(), s("#0")),
            ConstraintType::TangentSplines(a.clone(), s("#0"), b.clone(), s("#1")),
            ConstraintType::TangentSplines(a.clone(), s("#0"), a.clone(), s("#1")),
        ]);
        for at in [SplineAt::Start, SplineAt::End, SplineAt::CurveParam(s("#1"))] {
            cases.extend([
                ConstraintType::TangentLineSplineAtPoint(a_start.clone(), s("p1"), a.clone(), at.clone()),
                ConstraintType::TangentArcSplineAtPoint(a_start.clone(), s("a1_center"), a.clone(), at.clone()),
                ConstraintType::TangentSplinesAtPoint(a.clone(), at.clone(), b.clone(), SplineAt::Start),
                ConstraintType::TangentSplinesAtPoint(b.clone(), SplineAt::End, a.clone(), at.clone()),
            ]);
        }
    }
    // A two-point fit spline (a straight cubic) and a three-point one.
    let short = |n: usize| Spline::new(s("sp1"), SPLINES[0].1[..n].iter().map(|h| h.to_string()).collect(), true, None);
    for n in [2, 3] {
        cases.push(ConstraintType::PointOnSpline(s("p0"), short(n), s("#0")));
        cases.push(ConstraintType::TangentCircleSpline(s("c1_center"), s("c1"), short(n), s("#0")));
    }
    // Control points with a given (non-uniform, clamped) knot vector, and
    // degree 2 (three control points).
    let knotted = Spline::new(s("sp1"), SPLINES[0].1.iter().map(|h| h.to_string()).collect(), false,
        Some(vec![0.0, 0.0, 0.0, 0.0, 0.3, 2.0, 2.0, 2.0, 2.0]));
    cases.push(ConstraintType::PointOnSpline(s("p0"), knotted.clone(), s("#0")));
    cases.push(ConstraintType::TangentLineSpline(s("p0"), s("p1"), knotted, s("#0")));
    let quadratic = Spline::new(s("sp1"), SPLINES[0].1[..3].iter().map(|h| h.to_string()).collect(), false, None);
    cases.push(ConstraintType::TangentEllipseSpline(s("e1_center"), s("e1_focus"), s("e1"), quadratic, s("#0")));

    let mut failures = Vec::new();
    for ct in cases {
        for seed in [1, 2, 3, 42, 1234] {
            for m in fd_mismatches(&ct, seed) {
                failures.push(format!("{ct:?}: {m}"));
            }
        }
    }
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}
