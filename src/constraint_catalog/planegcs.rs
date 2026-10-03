//! The PlaneGCS dialect: the constraint types as FreeCAD GCS names them (FreeCAD GCS's
//! names and fields), each mapped onto an internal [`ConstraintType`]. Used
//! by `P3DSketch_Solve` and `acsSolveSketchPlaneGcs`; ACS's own JSON API
//! speaks the native vocabulary (`super::native`).
//!
//! Where GCS has one type per entity combination (`horizontal_l`,
//! `horizontal_pp`, …), so does the dialect: a type is exactly one row and
//! its fields are never inferred. `arc_rules` still builds `ArcRules` until
//! arcs keep their endpoints implicitly; `p2p_symmetric_ppl` is a mirror
//! across the line's Extension and `p2p_symmetric_ppp` a midpoint.

use super::{ConstraintSpec, FieldKind};
use crate::{ConstraintType, EllipseAxis};

use FieldKind::{Arc, Circle, Ellipse, Line, Point, Scalar, Value as Val};

const PP: &[(&str, FieldKind)] = &[("p1_id", Point), ("p2_id", Point)];
const L: &[(&str, FieldKind)] = &[("l_id", Line)];
const LL: &[(&str, FieldKind)] = &[("l1_id", Line), ("l2_id", Line)];
const PPPP: &[(&str, FieldKind)] = &[
    ("l1p1_id", Point),
    ("l1p2_id", Point),
    ("l2p1_id", Point),
    ("l2p2_id", Point),
];
const CC: &[(&str, FieldKind)] = &[("c1_id", Circle), ("c2_id", Circle)];
const DIAMETER: &[(&str, FieldKind)] = &[("e_id", Ellipse), ("p1_id", Point), ("p2_id", Point)];

pub(super) static SPECS: &[ConstraintSpec] = &[
    ConstraintSpec::new("horizontal_pp", PP, |a| {
        ConstraintType::Horizontal(a.p(0), a.p(1))
    }),
    ConstraintSpec::new("vertical_pp", PP, |a| {
        ConstraintType::Vertical(a.p(0), a.p(1))
    }),
    ConstraintSpec::new("horizontal_l", L, |a| {
        ConstraintType::Horizontal(a.p(0), a.p(1))
    }),
    ConstraintSpec::new("vertical_l", L, |a| {
        ConstraintType::Vertical(a.p(0), a.p(1))
    }),
    ConstraintSpec::new("parallel", LL, |a| {
        ConstraintType::Parallel(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    ConstraintSpec::new("perpendicular_ll", LL, |a| {
        ConstraintType::Perpendicular(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    ConstraintSpec::new("perpendicular_pppp", PPPP, |a| {
        ConstraintType::Perpendicular(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    ConstraintSpec::new("p2p_coincident", PP, |a| {
        ConstraintType::Coincident(a.p(0), a.p(1))
    }),
    ConstraintSpec::new(
        "point_on_line_pl",
        &[("p_id", Point), ("l_id", Line)],
        |a| ConstraintType::PointOnLine(a.p(0), a.p(1), a.p(2)),
    ),
    ConstraintSpec::new(
        "point_on_line_ppp",
        &[("p_id", Point), ("lp1_id", Point), ("lp2_id", Point)],
        |a| ConstraintType::PointOnLine(a.p(0), a.p(1), a.p(2)),
    ),
    ConstraintSpec::new(
        "point_on_circle",
        &[("p_id", Point), ("c_id", Circle)],
        |a| ConstraintType::PointOnCircle(a.p(0), a.center(0), a.c(0)),
    ),
    ConstraintSpec::new(
        "p2p_distance",
        &[("p1_id", Point), ("p2_id", Point), ("distance", Scalar)],
        |a| ConstraintType::DistancePointPoint(a.p(0), a.p(1), a.s(0)),
    ),
    ConstraintSpec::new(
        "p2l_distance",
        &[("p_id", Point), ("l_id", Line), ("distance", Scalar)],
        |a| ConstraintType::DistancePointLine(a.p(0), a.p(1), a.p(2), a.s(0)),
    ),
    ConstraintSpec::new(
        "l2l_angle_pppp",
        &[
            ("l1p1_id", Point),
            ("l1p2_id", Point),
            ("l2p1_id", Point),
            ("l2p2_id", Point),
            ("angle", Scalar),
        ],
        |a| ConstraintType::Angle(a.p(0), a.p(1), a.p(2), a.p(3), a.s(0)),
    ),
    ConstraintSpec::new(
        "l2l_angle_ll",
        &[("l1_id", Line), ("l2_id", Line), ("angle", Scalar)],
        |a| ConstraintType::Angle(a.p(0), a.p(1), a.p(2), a.p(3), a.s(0)),
    ),
    ConstraintSpec::new("equal_length", LL, |a| {
        ConstraintType::EqualLength(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    ConstraintSpec::new("equal_radius_cc", CC, |a| {
        ConstraintType::EqualRadius(a.c(0), a.c(1))
    }),
    ConstraintSpec::new("equal_radius_aa", &[("a1_id", Arc), ("a2_id", Arc)], |a| {
        ConstraintType::EqualRadius(a.c(0), a.c(1))
    }),
    ConstraintSpec::new(
        "circle_radius",
        &[("c_id", Circle), ("radius", Scalar)],
        |a| ConstraintType::FixedRadius(a.c(0), a.s(0)),
    ),
    ConstraintSpec::new("arc_radius", &[("a_id", Arc), ("radius", Scalar)], |a| {
        ConstraintType::FixedRadius(a.c(0), a.s(0))
    }),
    ConstraintSpec::new("tangent_lc", &[("l_id", Line), ("c_id", Circle)], |a| {
        ConstraintType::TangentLineCircle(a.p(0), a.p(1), a.center(0), a.c(0))
    }),
    ConstraintSpec::new("tangent_cc", CC, |a| {
        ConstraintType::Tangent(a.center(0), a.c(0), a.center(1), a.c(1))
    }),
    ConstraintSpec::new("concentric_cc", CC, |a| {
        ConstraintType::Concentric(a.center(0), a.center(1))
    }),
    ConstraintSpec::new("midpoint_on_line_ll", LL, |a| {
        ConstraintType::MidpointOfLineOnLine(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    ConstraintSpec::new("midpoint_on_line_pppp", PPPP, |a| {
        ConstraintType::MidpointOfLineOnLine(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    ConstraintSpec::new(
        "point_on_extension_pl",
        &[("p_id", Point), ("l_id", Line)],
        |a| ConstraintType::PointOnExtension(a.p(0), a.p(1), a.p(2)),
    ),
    ConstraintSpec::new(
        "p2l_extension_distance",
        &[("p_id", Point), ("l_id", Line), ("distance", Scalar)],
        |a| ConstraintType::DistancePointExtension(a.p(0), a.p(1), a.p(2), a.s(0)),
    ),
    ConstraintSpec::new(
        "tangent_extension_lc",
        &[("l_id", Line), ("c_id", Circle)],
        |a| ConstraintType::TangentExtensionCircle(a.p(0), a.p(1), a.center(0), a.c(0)),
    ),
    ConstraintSpec::new("midpoint_on_extension_ll", LL, |a| {
        ConstraintType::MidpointOfLineOnExtension(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    ConstraintSpec::new(
        "p2p_symmetric_ppp",
        &[("p1_id", Point), ("p2_id", Point), ("p_id", Point)],
        |a| ConstraintType::Midpoint(a.p(2), a.p(0), a.p(1)),
    ),
    ConstraintSpec::new(
        "p2p_symmetric_ppl",
        &[("p1_id", Point), ("p2_id", Point), ("l_id", Line)],
        |a| ConstraintType::MirrorPointExtension(a.p(0), a.p(1), a.p(2), a.p(3)),
    ),
    ConstraintSpec::new("coordinate_x", &[("p_id", Point), ("x", Scalar)], |a| {
        ConstraintType::EqualX(a.p(0), a.s(0))
    }),
    ConstraintSpec::new("coordinate_y", &[("p_id", Point), ("y", Scalar)], |a| {
        ConstraintType::EqualY(a.p(0), a.s(0))
    }),
    ConstraintSpec::new(
        "p2l_signed_distance",
        &[
            ("p_id", Point),
            ("l_id", Line),
            ("distance", Scalar),
            ("side", Scalar),
        ],
        |a| ConstraintType::SignedDistancePointExtension(a.p(0), a.p(1), a.p(2), a.s(0), a.s(1)),
    ),
    ConstraintSpec::new(
        "difference",
        &[("param1", Val), ("param2", Val), ("difference", Val)],
        |a| ConstraintType::Difference(a.o(0), a.o(1), a.o(2)),
    ),
    ConstraintSpec::new("equal", &[("param1", Val), ("param2", Val)], |a| {
        ConstraintType::Equal(a.o(0), a.o(1))
    }),
    ConstraintSpec::new("arc_rules", &[("a_id", Arc)], |a| {
        ConstraintType::ArcRules(a.center(0), a.arc_start(0), a.arc_end(0), a.c(0))
    }),
    ConstraintSpec::new("point_on_arc", &[("p_id", Point), ("a_id", Arc)], |a| {
        ConstraintType::PointOnArc(a.p(0), a.center(0), a.c(0))
    }),
    ConstraintSpec::new("tangent_la", &[("l_id", Line), ("a_id", Arc)], |a| {
        a.tangent_line_arc()
    }),
    ConstraintSpec::new(
        "p2p_angle",
        &[("p1_id", Point), ("p2_id", Point), ("angle", Scalar)],
        |a| ConstraintType::PointPointAngle(a.p(0), a.p(1), a.s(0)),
    ),
    ConstraintSpec::new(
        "mirror_point_ppl",
        &[("pA_id", Point), ("pB_id", Point), ("axis_id", Line)],
        |a| ConstraintType::MirrorPointExtension(a.p(0), a.p(1), a.p(2), a.p(3)),
    ),
    ConstraintSpec::new(
        "circular_instance",
        &[
            ("p0_id", Point),
            ("pk_id", Point),
            ("center_id", Point),
            ("angle", Scalar),
        ],
        |a| ConstraintType::CircularInstance(a.p(0), a.p(1), a.p(2), a.s(0)),
    ),
    ConstraintSpec::new(
        "linear_instance",
        &[
            ("p0_id", Point),
            ("pk_id", Point),
            ("dirP1_id", Point),
            ("dirP2_id", Point),
            ("base_distance", Scalar),
            ("N", Scalar),
        ],
        |a| ConstraintType::LinearInstance(a.p(0), a.p(1), a.p(2), a.p(3), a.s(0), a.s(1)),
    ),
    ConstraintSpec::new(
        "point_on_ellipse",
        &[("p_id", Point), ("e_id", Ellipse)],
        |a| {
            let (center, focus, e) = a.e(0);
            ConstraintType::PointOnEllipse(a.p(0), center, focus, e)
        },
    ),
    ConstraintSpec::new("tangent_le", &[("l_id", Line), ("e_id", Ellipse)], |a| {
        let (center, focus, e) = a.e(0);
        ConstraintType::TangentLineEllipse(a.p(0), a.p(1), center, focus, e)
    }),
    ConstraintSpec::new("internal_alignment_ellipse_major_diameter", DIAMETER, |a| {
        let (center, focus, e) = a.e(0);
        ConstraintType::EllipseDiameter(a.p(0), a.p(1), center, focus, e, EllipseAxis::Major)
    }),
    ConstraintSpec::new("internal_alignment_ellipse_minor_diameter", DIAMETER, |a| {
        let (center, focus, e) = a.e(0);
        ConstraintType::EllipseDiameter(a.p(0), a.p(1), center, focus, e, EllipseAxis::Minor)
    }),
];
