//! ACS's native constraint vocabulary: one type per relationship, with
//! role-named fields.
//!
//! A type may have several rows, its variants. Which one a constraint uses is
//! inferred from the kinds of the entities its fields reference (`distance`
//! with `b` a Point is point–point, with `b` a Line point–segment) and from
//! two flags: `extension: true`, which selects a variant measured against
//! the Line's Extension, and `internal: true`, which selects an inside
//! variant: inside tangency between circles and arcs, or a distance measured
//! inside a circle (never inferred from the geometry, and rejected where no
//! circle or arc takes it). Rows are tried in order; for the
//! `COMMUTATIVE` types, `a` and `b` may come in either order. A
//! combination no row takes is rejected, naming the kinds given and the ones
//! the type accepts.

use serde_json::Value;

use super::{
    ConstraintSpec as S, ExtensionFlag, FieldKind, InternalFlag, References, field_path,
    field_value,
};
use crate::{ConstraintType, Operand};

use FieldKind::{Arc, Axis, Circle, Ellipse, EllipticalArc, Line, Point, Scalar, Value as Val};

type Fields = &'static [(&'static str, FieldKind)];

const AB_POINTS: Fields = &[("a", Point), ("b", Point)];
const AB_LINES: Fields = &[("a", Line), ("b", Line)];
const LINE: Fields = &[("line", Line)];
const CC: Fields = &[("a", Circle), ("b", Circle)];
const CA: Fields = &[("a", Circle), ("b", Arc)];
const AA: Fields = &[("a", Arc), ("b", Arc)];

/// Types whose `a` and `b` fields may be given in either order.
const COMMUTATIVE: &[&str] = &["distance", "tangent", "concentric", "equal", "collinear"];

pub(super) static SPECS: &[S] = &[
    S::new("coincident", AB_POINTS, |a| {
        ConstraintType::Coincident(a.p(0), a.p(1))
    }),
    S::new("horizontal", LINE, |a| {
        ConstraintType::Horizontal(a.p(0), a.p(1))
    }),
    S::new("horizontal", AB_POINTS, |a| {
        ConstraintType::Horizontal(a.p(0), a.p(1))
    }),
    S::new("vertical", LINE, |a| {
        ConstraintType::Vertical(a.p(0), a.p(1))
    }),
    S::new("vertical", AB_POINTS, |a| {
        ConstraintType::Vertical(a.p(0), a.p(1))
    }),
    S::new("parallel", AB_LINES, |a| {
        ConstraintType::Parallel(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    S::new("perpendicular", AB_LINES, |a| {
        ConstraintType::Perpendicular(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    S::new("collinear", AB_LINES, |a| {
        ConstraintType::Collinear(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    // Normal to a curve: the line's Extension passes through its center.
    S::new("normal", &[("line", Line), ("curve", Circle)], |a| {
        ConstraintType::PointOnExtension(a.center(0), a.p(0), a.p(1))
    }),
    S::new("normal", &[("line", Line), ("curve", Arc)], |a| {
        ConstraintType::PointOnExtension(a.center(0), a.p(0), a.p(1))
    }),
    S::new(
        "angle",
        &[("a", Line), ("b", Line), ("value", Scalar)],
        |a| ConstraintType::Angle(a.p(0), a.p(1), a.p(2), a.p(3), a.s(0)),
    ),
    S::new("angle", &[("arc", Arc), ("value", Scalar)], |a| {
        ConstraintType::ArcSweep(a.c(0), a.s(0))
    }),
    S::new("direction", &[("line", Line), ("value", Scalar)], |a| {
        ConstraintType::PointPointAngle(a.p(0), a.p(1), a.s(0))
    }),
    S::new(
        "direction",
        &[("a", Point), ("b", Point), ("value", Scalar)],
        |a| ConstraintType::PointPointAngle(a.p(0), a.p(1), a.s(0)),
    ),
    S::new(
        "distance",
        &[("a", Point), ("b", Point), ("value", Scalar)],
        |a| ConstraintType::DistancePointPoint(a.p(0), a.p(1), a.s(0)),
    ),
    S::new(
        "distance",
        &[("a", Point), ("b", Line), ("value", Scalar)],
        |a| ConstraintType::DistancePointLine(a.p(0), a.p(1), a.p(2), a.s(0)),
    )
    .segment(),
    S::new(
        "distance",
        &[("a", Point), ("b", Line), ("value", Scalar)],
        |a| ConstraintType::DistancePointExtension(a.p(0), a.p(1), a.p(2), a.s(0)),
    )
    .extension(),
    S::new(
        "distance",
        &[("a", Point), ("b", Circle), ("value", Scalar)],
        |a| ConstraintType::DistancePointCircle(a.p(0), a.center(0), a.c(0), a.s(0), false),
    )
    .external(),
    S::new(
        "distance",
        &[("a", Point), ("b", Circle), ("value", Scalar)],
        |a| ConstraintType::DistancePointCircle(a.p(0), a.center(0), a.c(0), a.s(0), true),
    )
    .internal(),
    S::new(
        "distance",
        &[("a", Point), ("b", Arc), ("value", Scalar)],
        |a| ConstraintType::DistancePointArc(a.p(0), a.center(0), a.c(0), a.s(0), false),
    )
    .external(),
    S::new(
        "distance",
        &[("a", Point), ("b", Arc), ("value", Scalar)],
        |a| ConstraintType::DistancePointArc(a.p(0), a.center(0), a.c(0), a.s(0), true),
    )
    .internal(),
    S::new(
        "distance",
        &[("a", Line), ("b", Circle), ("value", Scalar)],
        |a| ConstraintType::DistanceLineCircle(a.p(0), a.p(1), a.center(0), a.c(0), a.s(0)),
    )
    .segment(),
    S::new(
        "distance",
        &[("a", Line), ("b", Circle), ("value", Scalar)],
        |a| ConstraintType::DistanceExtensionCircle(a.p(0), a.p(1), a.center(0), a.c(0), a.s(0)),
    )
    .extension(),
    S::new(
        "distance",
        &[("a", Line), ("b", Arc), ("value", Scalar)],
        |a| ConstraintType::DistanceLineArc(a.p(0), a.p(1), a.center(0), a.c(0), a.s(0)),
    )
    .segment(),
    S::new(
        "distance",
        &[("a", Line), ("b", Arc), ("value", Scalar)],
        |a| ConstraintType::DistanceExtensionArc(a.p(0), a.p(1), a.center(0), a.c(0), a.s(0)),
    )
    .extension(),
    S::new(
        "distance",
        &[("a", Circle), ("b", Circle), ("value", Scalar)],
        |a| {
            ConstraintType::DistanceCircleCircle(
                a.center(0),
                a.c(0),
                a.center(1),
                a.c(1),
                a.s(0),
                false,
            )
        },
    )
    .external(),
    S::new(
        "distance",
        &[("a", Circle), ("b", Circle), ("value", Scalar)],
        |a| {
            ConstraintType::DistanceCircleCircle(
                a.center(0),
                a.c(0),
                a.center(1),
                a.c(1),
                a.s(0),
                true,
            )
        },
    )
    .internal(),
    S::new(
        "distance",
        &[("a", Circle), ("b", Arc), ("value", Scalar)],
        |a| {
            ConstraintType::DistanceCircleArc(a.center(0), a.c(0), a.center(1), a.c(1), a.s(0), false)
        },
    )
    .external(),
    S::new(
        "distance",
        &[("a", Circle), ("b", Arc), ("value", Scalar)],
        |a| {
            ConstraintType::DistanceCircleArc(a.center(0), a.c(0), a.center(1), a.c(1), a.s(0), true)
        },
    )
    .internal(),
    S::new(
        "distance",
        &[("a", Arc), ("b", Arc), ("value", Scalar)],
        |a| ConstraintType::DistanceArcs(a.center(0), a.c(0), a.center(1), a.c(1), a.s(0), false),
    )
    .external(),
    S::new(
        "distance",
        &[("a", Arc), ("b", Arc), ("value", Scalar)],
        |a| ConstraintType::DistanceArcs(a.center(0), a.c(0), a.center(1), a.c(1), a.s(0), true),
    )
    .internal(),
    S::new(
        "distance",
        &[("a", Line), ("b", Line), ("value", Scalar)],
        |a| ConstraintType::DistanceLineLine(a.p(0), a.p(1), a.p(2), a.p(3), a.s(0)),
    ),
    S::new(
        "offset",
        &[
            ("point", Point),
            ("line", Line),
            ("value", Scalar),
            ("side", Scalar),
        ],
        |a| ConstraintType::SignedDistancePointExtension(a.p(0), a.p(1), a.p(2), a.s(0), a.s(1)),
    ),
    S::new("on", &[("point", Point), ("curve", Line)], |a| {
        ConstraintType::PointOnLine(a.p(0), a.p(1), a.p(2))
    })
    .segment(),
    S::new("on", &[("point", Point), ("curve", Line)], |a| {
        ConstraintType::PointOnExtension(a.p(0), a.p(1), a.p(2))
    })
    .extension(),
    S::new("on", &[("point", Point), ("curve", Circle)], |a| {
        ConstraintType::PointOnCircle(a.p(0), a.center(0), a.c(0))
    }),
    S::new("on", &[("point", Point), ("curve", Arc)], |a| {
        ConstraintType::PointOnArc(a.p(0), a.center(0), a.c(0))
    }),
    S::new("on", &[("point", Point), ("curve", Ellipse)], |a| {
        let (center, focus, e) = a.e(0);
        ConstraintType::PointOnEllipse(a.p(0), center, focus, e)
    }),
    S::new("on", &[("point", Point), ("curve", EllipticalArc)], |a| {
        let (center, focus, e) = a.e(0);
        ConstraintType::PointOnEllipticalArc(a.p(0), center, focus, e)
    }),
    S::new(
        "midpoint",
        &[("entities[0]", Point), ("entities[1]", Line)],
        |a| ConstraintType::Midpoint(a.p(0), a.p(1), a.p(2)),
    ),
    S::new(
        "midpoint",
        &[("entities[0]", Line), ("entities[1]", Line)],
        |a| ConstraintType::MidpointOfLineOnLine(a.p(0), a.p(1), a.p(2), a.p(3)),
    )
    .segment(),
    S::new(
        "midpoint",
        &[("entities[0]", Line), ("entities[1]", Line)],
        |a| ConstraintType::MidpointOfLineOnExtension(a.p(0), a.p(1), a.p(2), a.p(3)),
    )
    .extension(),
    S::new(
        "midpoint",
        &[("entities[0]", Point), ("entities[1]", Arc)],
        |a| ConstraintType::MidpointOfArc(a.p(0), a.center(0), a.c(0)),
    ),
    S::new("tangent", &[("a", Line), ("b", Circle)], |a| {
        ConstraintType::TangentLineCircle(a.p(0), a.p(1), a.center(0), a.c(0))
    })
    .segment(),
    S::new("tangent", &[("a", Line), ("b", Circle)], |a| {
        ConstraintType::TangentExtensionCircle(a.p(0), a.p(1), a.center(0), a.c(0))
    })
    .extension(),
    S::new("tangent", &[("a", Line), ("b", Arc)], |a| {
        a.tangent_line_arc()
    }),
    S::new("tangent", &[("a", Line), ("b", Ellipse)], |a| {
        let (center, focus, e) = a.e(0);
        ConstraintType::TangentLineEllipse(a.p(0), a.p(1), center, focus, e)
    }),
    S::new("tangent", &[("a", Line), ("b", EllipticalArc)], |a| {
        a.tangent_line_elliptical_arc()
    }),
    S::new("tangent", &[("a", Arc), ("b", EllipticalArc)], |a| {
        a.tangent_arc_elliptical_arc()
    })
    .check(super::arc_and_elliptical_arc_share_an_endpoint),
    S::new("tangent", CC, |a| {
        ConstraintType::Tangent(a.center(0), a.c(0), a.center(1), a.c(1))
    })
    .external(),
    S::new("tangent", CC, |a| {
        ConstraintType::TangentCirclesInternal(a.center(0), a.c(0), a.center(1), a.c(1))
    })
    .internal(),
    S::new("tangent", CA, |a| {
        ConstraintType::TangentCircleArc(a.center(0), a.c(0), a.center(1), a.c(1), false)
    })
    .external(),
    S::new("tangent", CA, |a| {
        ConstraintType::TangentCircleArc(a.center(0), a.c(0), a.center(1), a.c(1), true)
    })
    .internal(),
    S::new("tangent", AA, |a| a.tangent_arcs(false)).external(),
    S::new("tangent", AA, |a| a.tangent_arcs(true)).internal(),
    S::new("concentric", CC, |a| {
        ConstraintType::Concentric(a.center(0), a.center(1))
    }),
    S::new("concentric", CA, |a| {
        ConstraintType::Concentric(a.center(0), a.center(1))
    }),
    S::new("concentric", AA, |a| {
        ConstraintType::Concentric(a.center(0), a.center(1))
    }),
    S::new("equal", AB_LINES, |a| {
        ConstraintType::EqualLength(a.p(0), a.p(1), a.p(2), a.p(3))
    }),
    S::new("equal", CC, |a| ConstraintType::EqualRadius(a.c(0), a.c(1))),
    S::new("equal", CA, |a| ConstraintType::EqualRadius(a.c(0), a.c(1))),
    S::new("equal", AA, |a| ConstraintType::EqualRadius(a.c(0), a.c(1))),
    S::new("equal", &[("a", Val), ("b", Val)], |a| {
        ConstraintType::Equal(a.o(0), a.o(1))
    }),
    S::new("radius", &[("curve", Circle), ("value", Scalar)], |a| {
        ConstraintType::FixedRadius(a.c(0), a.s(0))
    }),
    S::new("radius", &[("curve", Arc), ("value", Scalar)], |a| {
        ConstraintType::FixedRadius(a.c(0), a.s(0))
    }),
    S::new("diameter", &[("curve", Circle), ("value", Scalar)], |a| {
        ConstraintType::Diameter(a.c(0), a.s(0))
    }),
    S::new("diameter", &[("curve", Arc), ("value", Scalar)], |a| {
        ConstraintType::Diameter(a.c(0), a.s(0))
    }),
    S::new("length", &[("curve", Line), ("value", Scalar)], |a| {
        ConstraintType::DistancePointPoint(a.p(0), a.p(1), a.s(0))
    }),
    S::new("length", &[("curve", Arc), ("value", Scalar)], |a| {
        ConstraintType::ArcLength(a.c(0), a.s(0))
    }),
    S::new(
        "difference",
        &[("a", Val), ("b", Val), ("value", Val)],
        |a| ConstraintType::Difference(a.o(0), a.o(1), a.o(2)),
    ),
    S::new("x", &[("point", Point), ("value", Scalar)], |a| {
        ConstraintType::EqualX(a.p(0), a.s(0))
    }),
    S::new("y", &[("point", Point), ("value", Scalar)], |a| {
        ConstraintType::EqualY(a.p(0), a.s(0))
    }),
    // Horizontal/vertical distance dimensions, signed: b.x − a.x = value
    // (a Line's p2.x − p1.x), so a negative value puts `b` left of `a`;
    // `y` for vertical. Not commutative. Built as `Difference` over the
    // coordinates: its residual is exactly that, linear.
    S::new(
        "horizontal_distance",
        &[("a", Point), ("b", Point), ("value", Scalar)],
        |a| ConstraintType::Difference(
            Operand::X(a.p(0)),
            Operand::X(a.p(1)),
            Operand::Const(a.s(0)),
        ),
    ),
    S::new(
        "horizontal_distance",
        &[("line", Line), ("value", Scalar)],
        |a| ConstraintType::Difference(
            Operand::X(a.p(0)),
            Operand::X(a.p(1)),
            Operand::Const(a.s(0)),
        ),
    ),
    S::new(
        "vertical_distance",
        &[("a", Point), ("b", Point), ("value", Scalar)],
        |a| ConstraintType::Difference(
            Operand::Y(a.p(0)),
            Operand::Y(a.p(1)),
            Operand::Const(a.s(0)),
        ),
    ),
    S::new(
        "vertical_distance",
        &[("line", Line), ("value", Scalar)],
        |a| ConstraintType::Difference(
            Operand::Y(a.p(0)),
            Operand::Y(a.p(1)),
            Operand::Const(a.s(0)),
        ),
    ),
    S::new(
        "mirror",
        &[("source", Point), ("image", Point), ("axis", Line)],
        |a| ConstraintType::MirrorPointExtension(a.p(0), a.p(1), a.p(2), a.p(3)),
    ),
    S::new(
        "rotation",
        &[
            ("source", Point),
            ("copy", Point),
            ("center", Point),
            ("angle", Scalar),
        ],
        |a| ConstraintType::CircularInstance(a.p(0), a.p(1), a.p(2), a.s(0)),
    ),
    S::new(
        "translation",
        &[
            ("source", Point),
            ("copy", Point),
            ("from", Point),
            ("to", Point),
            ("distance", Scalar),
            ("count", Scalar),
        ],
        |a| ConstraintType::LinearInstance(a.p(0), a.p(1), a.p(2), a.p(3), a.s(0), a.s(1)),
    ),
    S::new(
        "ellipse_axis",
        &[("ellipse", Ellipse), ("point", Point), ("which", Axis)],
        |a| {
            let (center, focus, e) = a.e(0);
            ConstraintType::EllipseAxisPoint(a.p(0), center, focus, e, a.axis(0))
        },
    ),
    S::new(
        "ellipse_axis",
        &[
            ("ellipse", Ellipse),
            ("a", Point),
            ("b", Point),
            ("which", Axis),
        ],
        |a| {
            let (center, focus, e) = a.e(0);
            ConstraintType::EllipseDiameter(a.p(0), a.p(1), center, focus, e, a.axis(0))
        },
    ),
    S::new(
        "ellipse_axis",
        &[("ellipse", EllipticalArc), ("point", Point), ("which", Axis)],
        |a| {
            let (center, focus, e) = a.e(0);
            ConstraintType::EllipseAxisPoint(a.p(0), center, focus, e, a.axis(0))
        },
    ),
    S::new(
        "ellipse_axis",
        &[
            ("ellipse", EllipticalArc),
            ("a", Point),
            ("b", Point),
            ("which", Axis),
        ],
        |a| {
            let (center, focus, e) = a.e(0);
            ConstraintType::EllipseDiameter(a.p(0), a.p(1), center, focus, e, a.axis(0))
        },
    ),
];

/// Builds the native constraint `c` of type `json_type`, choosing the row
/// whose entity kinds and `extension` and `internal` flags match.
pub(super) fn parse(
    json_type: &str,
    c: &Value,
    refs: &References,
) -> Result<ConstraintType, String> {
    let variants: Vec<&S> = SPECS.iter().filter(|s| s.json_type == json_type).collect();
    if variants.is_empty() {
        return Err(format!("unknown type '{json_type}'"));
    }
    let flag = |name: &str| match c.get(name) {
        None => Ok(false),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| format!("field '{name}' is not true or false")),
    };
    let (extension, internal) = (flag("extension")?, flag("internal")?);
    let swapped = COMMUTATIVE.contains(&json_type).then(|| swap_a_b(c));
    let matching = |spec: &S| {
        std::iter::once(c)
            .chain(swapped.as_ref())
            .find(|obj| matches_kinds(spec, obj, refs))
    };
    let accepts = |s: &S| s.extension.accepts(extension) && s.internal.accepts(internal);
    for spec in variants.iter().filter(|s| accepts(s)) {
        if let Some(obj) = matching(spec) {
            return spec.parse(obj, refs);
        }
    }
    // A type with one row reports exactly what's wrong with its fields.
    if let [only] = variants[..]
        && accepts(only)
    {
        return only.parse(c, refs);
    }
    // `internal` on entities whose row has no inside variant (a Line).
    if internal
        && variants.iter().any(|s| s.internal == InternalFlag::Internal)
        && variants.iter().any(|s| {
            s.internal == InternalFlag::NotAccepted
                && s.extension.accepts(extension)
                && matching(s).is_some()
        })
    {
        return Err(format!(
            "field 'internal' applies only between a circle or arc and a point, circle or arc; got {}",
            got_kinds(&variants, c, refs, false, false)
        ));
    }
    Err(unsupported(json_type, &variants, c, refs, extension, internal))
}

/// `c` with its `a` and `b` fields exchanged.
fn swap_a_b(c: &Value) -> Value {
    let mut out = c.clone();
    if let Some(obj) = out.as_object_mut() {
        let (a, b) = (obj.remove("a"), obj.remove("b"));
        if let Some(b) = b {
            obj.insert("a".into(), b);
        }
        if let Some(a) = a {
            obj.insert("b".into(), a);
        }
    }
    out
}

/// The kind of entity `id` names, if it names one.
fn entity_kind<'r>(refs: &'r References, id: &str) -> Option<&'r str> {
    refs.entity_types.get(id).map(String::as_str)
}

/// Whether every entity field of `spec` references an entity of its kind,
/// no number field references an entity, and each array field has exactly
/// the elements `spec` reads.
fn matches_kinds(spec: &S, c: &Value, refs: &References) -> bool {
    let array_lengths_match = spec.fields.iter().all(|&(name, _)| match field_path(name) {
        (_, None) => true,
        (key, Some(_)) => {
            let read = spec.fields.iter().filter(|&&(n, _)| field_path(n).0 == key);
            c.get(key).and_then(Value::as_array).map(Vec::len) == Some(read.count())
        }
    });
    array_lengths_match
        && spec.fields.iter().all(|&(name, kind)| {
            let field = field_value(c, name);
            if kind.is_entity() {
                field
                    .and_then(Value::as_str)
                    .and_then(|id| entity_kind(refs, id))
                    == Some(kind.as_str())
            } else {
                !field
                    .and_then(Value::as_str)
                    .is_some_and(|s| refs.entity_types.contains_key(s))
            }
        })
}

/// The error for a combination of kinds no row of `json_type` takes:
/// `unsupported combination for 'distance': got a: point, b: circle;
/// expected (a: point, b: point) or (a: point, b: line) or …`.
fn unsupported(
    json_type: &str,
    variants: &[&S],
    c: &Value,
    refs: &References,
    extension: bool,
    internal: bool,
) -> String {
    let got = got_kinds(variants, c, refs, extension, internal);
    let expected: Vec<String> = variants
        .iter()
        .map(|s| {
            let mut fields: Vec<String> = s
                .fields
                .iter()
                .filter(|(_, kind)| !matches!(kind, Scalar | Axis))
                .map(|(name, kind)| format!("{name}: {}", kind.as_str()))
                .collect();
            if s.extension == ExtensionFlag::Extension {
                fields.push("extension".into());
            }
            if s.internal == InternalFlag::Internal {
                fields.push("internal".into());
            }
            format!("({})", fields.join(", "))
        })
        .collect();
    format!(
        "unsupported combination for '{json_type}': got {got}; expected {}",
        expected.join(" or ")
    )
}

/// The kinds of the entities `c` references in the entity fields of
/// `variants`, and the flags it sets: `a: point, b: circle, extension`.
fn got_kinds(
    variants: &[&S],
    c: &Value,
    refs: &References,
    extension: bool,
    internal: bool,
) -> String {
    let mut names: Vec<&str> = Vec::new();
    for &(name, kind) in variants.iter().flat_map(|s| s.fields) {
        if !matches!(kind, Scalar | Axis) && !names.contains(&name) {
            names.push(name);
        }
    }
    let mut got: Vec<String> = names
        .iter()
        .filter_map(|&name| {
            let v = field_value(c, name)?;
            let kind = match v.as_str() {
                Some(id) => match entity_kind(refs, id) {
                    Some(kind) => kind.to_string(),
                    None => format!("'{id}' (not an entity)"),
                },
                None => "value".to_string(),
            };
            Some(format!("{name}: {kind}"))
        })
        .collect();
    if extension {
        got.push("extension".into());
    }
    if internal {
        got.push("internal".into());
    }
    if got.is_empty() {
        "nothing".to_string()
    } else {
        got.join(", ")
    }
}
