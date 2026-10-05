//! Checks every constraint's analytical Jacobian against central finite
//! differences of its residual, at several generic configurations.
//!
//! A constraint's Jacobian has partials for every variable it reads, Guides
//! included (only a drag holds a Guide), so the differences perturb every
//! occurrence of each variable, and a variable it doesn't read must have a
//! zero column.

use acs::constraint_catalog::{Args, ConstraintSpec, EllipseRef, FieldKind, Vocabulary};
use acs::constraints::{
    Constraint, ConstraintType, EllipseAxis, Operand, create_constraint, reads,
};
use nalgebra::DMatrix;
use acs::geometry::{Arc, Circle, Ellipse, Point};
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
    pm
}

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
/// first catalog row that reaches it in either vocabulary (native first),
/// with fields filled from the entities in `build_pm`: distinct points, the
/// two circles, the two arcs and a generic scalar. Value fields take, in
/// turn, a radius, a point coordinate and a constant.
fn all_constraints() -> Vec<ConstraintType> {
    let mut seen = std::collections::HashSet::new();
    [Vocabulary::Native, Vocabulary::PlaneGcs]
        .into_iter()
        .flat_map(Vocabulary::specs)
        .flat_map(build_from_spec)
        .filter(|ct| seen.insert(variant_name(ct)))
        .collect()
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
        if kind == FieldKind::Axis {
            args.axes.push(EllipseAxis::Major);
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
    out
}

/// Exhaustive on purpose: a new `ConstraintType` variant fails to compile
/// here until it is listed, and the test below then requires a catalog row
/// (native or PlaneGCS dialect) that reaches it.
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
        ArcRules(..) => "ArcRules",
        PointOnArc(..) => "PointOnArc",
        TangentLineArc(..) => "TangentLineArc",
        TangentAtPoint(..) => "TangentAtPoint",
        PointPointAngle(..) => "PointPointAngle",
        MirrorPointExtension(..) => "MirrorPointExtension",
        CircularInstance(..) => "CircularInstance",
        LinearInstance(..) => "LinearInstance",
        PointOnEllipse(..) => "PointOnEllipse",
        TangentLineEllipse(..) => "TangentLineEllipse",
        EllipseAxisPoint(..) => "EllipseAxisPoint",
        EllipseDiameter(..) => "EllipseDiameter",
        TangentCirclesInternal(..) => "TangentCirclesInternal",
        TangentCircleArc(..) => "TangentCircleArc",
        TangentArcs(..) => "TangentArcs",
        TangentArcsAtPoint(..) => "TangentArcsAtPoint",
        Collinear(..) => "Collinear",
    }
}
const ALL_VARIANTS: [&str; 44] = [
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
    "ArcRules",
    "PointOnArc",
    "TangentLineArc",
    "TangentAtPoint",
    "PointPointAngle",
    "MirrorPointExtension",
    "CircularInstance",
    "LinearInstance",
    "PointOnEllipse",
    "TangentLineEllipse",
    "EllipseAxisPoint",
    "EllipseDiameter",
    "TangentCirclesInternal",
    "TangentCircleArc",
    "TangentArcs",
    "TangentArcsAtPoint",
    "Collinear",
];

#[test]
fn every_constraint_variant_is_reached_by_a_vocabulary() {
    let covered: std::collections::HashSet<&str> =
        all_constraints().iter().map(variant_name).collect();
    for v in ALL_VARIANTS {
        assert!(
            covered.contains(v),
            "no native or PlaneGCS catalog row builds {v}"
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
    let pm = build_pm(seed);
    let c = create_constraint(ct.clone()).unwrap();
    let jac = c.jacobian(&pm);
    let cols: Vec<usize> = reads(c.as_ref())
        .iter()
        .map(|v| v.column(&pm).unwrap())
        .collect();
    let x0: Vec<f64> = cols.iter().map(|&k| pm.values()[k]).collect();
    let h = 1e-6;
    let mut out = Vec::new();

    let residual = c.residual(&pm);
    for (r, v) in local_residual(c.as_ref(), &x0).into_iter().enumerate() {
        assert_eq!(residual[r], v, "{ct:?}: residual() disagrees with eval()");
    }

    for i in 0..pm.num_vars() {
        let shifted = |dh: f64| {
            let mut x = x0.clone();
            for k in (0..cols.len()).filter(|&k| cols[k] == i) {
                x[k] += dh;
            }
            local_residual(c.as_ref(), &x)
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
        ConstraintType::ArcRules(s("a1_center"), s("a1_start"), s("a1_start"), s("a1")),
        ConstraintType::ArcRules(s("a1_start"), s("a1_start"), s("a1_end"), s("a1")),
        ConstraintType::PointOnArc(s("a1_start"), s("a1_center"), s("a1")),
        ConstraintType::PointOnArc(s("p0"), s("a1_center"), s("a1")),
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
        ConstraintType::EllipseAxisPoint(s("p0"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Minor),
        ConstraintType::EllipseAxisPoint(s("e1_focus"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Major),
        ConstraintType::EllipseAxisPoint(s("e2_focus"), s("e2_center"), s("e2_focus"), s("e2"), EllipseAxis::Minor),
        ConstraintType::EllipseDiameter(s("p0"), s("p1"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Minor),
        ConstraintType::EllipseDiameter(s("p0"), s("p0"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Major),
        ConstraintType::EllipseDiameter(s("e1_focus"), s("p1"), s("e1_center"), s("e1_focus"), s("e1"), EllipseAxis::Major),
        ConstraintType::EllipseDiameter(s("p0"), s("e2_center"), s("e2_center"), s("e2_focus"), s("e2"), EllipseAxis::Minor),
        ConstraintType::Equal(Operand::MinorRadius(s("e1")), Operand::MinorRadius(s("e2"))),
        ConstraintType::Difference(
            Operand::MinorRadius(s("e1")),
            Operand::Radius(s("c1")),
            Operand::MinorRadius(s("e1")),
        ),
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
