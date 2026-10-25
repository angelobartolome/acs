//! The JSON sketch API as Rust types: [`SolveRequest`] in, [`SolveResponse`] out, through
//! [`solve`]; a Spline's curve without solving through [`spline_curve`].
//!
//! They serialize to exactly the JSON [`solve_sketch_json`] reads and writes, so a typed
//! request means what its JSON means and goes through the same parser and catalog. With the
//! `uniffi` feature they are UniFFI records and enums, from which `uniffi-bindgen` generates
//! Swift and Kotlin; built for `wasm32` they carry TypeScript declarations (tsify) for the WASM
//! package's typed `acsSolve` and `acsCurve`.
//!
//! A constraint type whose rows read different fields has them as a nested enum flattened
//! into the constraint: [`LineOrPoints`], [`AngleOf`], [`AxisPoints`]. A type whose rows differ
//! only in the kinds of the entities they reference (`distance`, `on`, `tangent`, …) has one
//! case; the catalog picks the row from the kinds, as it does for JSON.
//!
//! [`solve_sketch_json`]: crate::sketch_solve::solve_sketch_json

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::sketch_solve::solve_value;
use crate::spline::BSpline;

/// A request: `{version: 1, primitives, maxIterations?, analyze?}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SolveRequest {
    /// Always 1.
    #[cfg_attr(feature = "uniffi", uniffi(default = 1))]
    pub version: u32,
    /// Geometry, Parameters and constraints, in any order.
    pub primitives: Vec<Primitive>,
    /// The iteration limit; 100 when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "uniffi", uniffi(default = None))]
    pub max_iterations: Option<u32>,
    /// `false` leaves `conflicting`, `redundant`, `dof` and `fullyConstrained` out of the
    /// response (a drag's frames); `true` when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "uniffi", uniffi(default = None))]
    pub analyze: Option<bool>,
}

/// A number, or the id of a `param` primitive (or a numeric string).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(untagged)]
pub enum Scalar {
    /// The value itself.
    Number(f64),
    /// A Parameter id, or a number written as a string.
    Parameter(String),
}

/// A `value` field (`equal`, `difference`): a number, an id (a Parameter, or the entity
/// `equal` compares), or a property of an entity, which the solver moves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(untagged)]
pub enum Operand {
    /// The value itself.
    Number(f64),
    /// A Parameter id, a number written as a string, or the id of a line, circle or arc.
    Id(String),
    /// A property of an entity: `{entity, property}`.
    Property(PropertyRef),
}

/// `{entity, property}`: one value of an entity, as an [`Operand`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
pub struct PropertyRef {
    /// The entity's id.
    pub entity: String,
    /// Which of its values.
    pub property: Property,
}

/// The properties a [`PropertyRef`] reads: a point's `x`/`y`, a circle's or arc's
/// `radius`, an ellipse's or elliptical arc's `radmin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "snake_case")]
pub enum Property {
    /// A point's x.
    X,
    /// A point's y.
    Y,
    /// A circle's or arc's radius.
    Radius,
    /// An ellipse's or elliptical arc's minor radius.
    Radmin,
}

/// What `horizontal`, `vertical`, `horizontal_distance`, `vertical_distance` and `direction`
/// measure: a line (`{line}`) or two points (`{a, b}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(untagged)]
pub enum LineOrPoints {
    /// A line, from `p1` to `p2`.
    Line {
        /// The line's id.
        line: String,
    },
    /// Two points, from `a` to `b`.
    Points {
        /// The first point's id.
        a: String,
        /// The second point's id.
        b: String,
    },
}

/// What `angle` measures: between two lines, or an arc's sweep.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(untagged, rename_all_fields = "camelCase")]
pub enum AngleOf {
    /// From line `a` to line `b`. Either flag present makes it the angle between rays (one
    /// root), `true` measuring its line `p2 → p1`; with neither, θ and θ + π both hold.
    Lines {
        /// The first line's id.
        a: String,
        /// The second line's id.
        b: String,
        /// Measures `a` as the ray `p2 → p1` when `true`.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        reverse_a: Option<bool>,
        /// Measures `b` as the ray `p2 → p1` when `true`.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        reverse_b: Option<bool>,
    },
    /// An arc's sweep, `0 < value < 2π`.
    Arc {
        /// The arc's id.
        arc: String,
    },
}

/// Where `ellipse_axis` puts points: one at either end of the axis, or a pair symmetric
/// about the center along it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(untagged)]
pub enum AxisPoints {
    /// One point, at either end of the axis.
    Point {
        /// The point's id.
        point: String,
    },
    /// Two points at the axis's two ends, in either order.
    Pair {
        /// One end's point id.
        a: String,
        /// The other end's point id.
        b: String,
    },
}

/// An ellipse's axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    /// Through the foci.
    Major,
    /// Perpendicular to the major axis, through the center.
    Minor,
}

/// A point `[x, y]`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Xy {
    /// x.
    pub x: f64,
    /// y.
    pub y: f64,
}

impl Serialize for Xy {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        [self.x, self.y].serialize(s)
    }
}

impl<'de> Deserialize<'de> for Xy {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let [x, y] = <[f64; 2]>::deserialize(d)?;
        Ok(Xy { x, y })
    }
}

/// A Spline's curve: a clamped B-spline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
pub struct Curve {
    /// Always 3.
    pub degree: u32,
    /// The clamped knot vector.
    pub knots: Vec<f64>,
    /// The control points, `[x, y]` each.
    #[cfg_attr(target_arch = "wasm32", tsify(type = "[number, number][]"))]
    pub control_points: Vec<Xy>,
}

impl From<BSpline> for Curve {
    fn from(c: BSpline) -> Self {
        Curve {
            degree: c.degree as u32,
            knots: c.knots,
            control_points: c.control_points.into_iter().map(|[x, y]| Xy { x, y }).collect(),
        }
    }
}

/// Geometry, Parameters and constraints: one flat array, `{type, id, …}` each.
///
/// Geometry references its Points by id. Every geometry primitive takes `isReference`
/// (Reference Geometry: held fixed), `isConstruction` and `group`, which the solver passes
/// through. Every constraint takes `temporary`: a soft goal, met as closely as the real
/// constraints allow (a drag).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum Primitive {
    /// A point.
    Point {
        /// Its id.
        id: String,
        /// x.
        x: f64,
        /// y.
        y: f64,
        /// Holds it in place.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        fixed: Option<bool>,
        /// Reference Geometry: holds it in place.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_reference: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_construction: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },
    /// A segment from `p1` to `p2` (never an infinite line).
    Line {
        /// Its id.
        id: String,
        /// The first endpoint's id.
        #[serde(rename = "p1_id")]
        p1: String,
        /// The second endpoint's id.
        #[serde(rename = "p2_id")]
        p2: String,
        /// Reference Geometry (its points aren't held: fix them).
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_reference: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_construction: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },
    /// A circle.
    Circle {
        /// Its id.
        id: String,
        /// The center point's id.
        #[serde(rename = "c_id")]
        center: String,
        /// The radius.
        radius: f64,
        /// Holds the radius.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        fixed: Option<bool>,
        /// Reference Geometry: holds the radius.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_reference: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_construction: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },
    /// An arc, sweeping counter-clockwise from `start` to `end`.
    Arc {
        /// Its id.
        id: String,
        /// The center point's id.
        #[serde(rename = "c_id")]
        center: String,
        /// The start point's id.
        #[serde(rename = "start_id")]
        start: String,
        /// The end point's id.
        #[serde(rename = "end_id")]
        end: String,
        /// The radius.
        radius: f64,
        /// The start angle, radians.
        #[serde(rename = "start_angle")]
        start_angle: f64,
        /// The end angle, radians.
        #[serde(rename = "end_angle")]
        end_angle: f64,
        /// Holds the radius and angles.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        fixed: Option<bool>,
        /// Reference Geometry: holds the radius and angles.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_reference: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_construction: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },
    /// An ellipse: center, a focus and the minor radius.
    Ellipse {
        /// Its id.
        id: String,
        /// The center point's id.
        #[serde(rename = "c_id")]
        center: String,
        /// The focus point's id.
        #[serde(rename = "focus1_id")]
        focus: String,
        /// The minor radius.
        radmin: f64,
        /// Holds the minor radius.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        fixed: Option<bool>,
        /// Reference Geometry: holds the minor radius.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_reference: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_construction: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },
    /// An ellipse's span between parametric angles, counter-clockwise.
    EllipticalArc {
        /// Its id.
        id: String,
        /// The center point's id.
        #[serde(rename = "c_id")]
        center: String,
        /// The focus point's id.
        #[serde(rename = "focus1_id")]
        focus: String,
        /// The start point's id.
        #[serde(rename = "start_id")]
        start: String,
        /// The end point's id.
        #[serde(rename = "end_id")]
        end: String,
        /// The minor radius.
        radmin: f64,
        /// The start's parametric angle, radians.
        #[serde(rename = "start_angle")]
        start_angle: f64,
        /// The end's parametric angle, radians.
        #[serde(rename = "end_angle")]
        end_angle: f64,
        /// Holds the minor radius and angles.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        fixed: Option<bool>,
        /// Reference Geometry: holds the minor radius and angles.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_reference: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_construction: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },
    /// A cubic spline through (`interpolated`) or controlled by its handle points.
    Spline {
        /// Its id.
        id: String,
        /// The handle points' ids.
        points: Vec<String>,
        /// `true`: fit points the curve passes through; `false`: control points.
        interpolated: bool,
        /// Control points only: a clamped knot vector; uniform when absent.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        knots: Option<Vec<f64>>,
        /// Response only: the solved curve. A request ignores it.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        curve: Option<Curve>,
        /// Reference Geometry (its points aren't held: fix them).
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_reference: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        is_construction: Option<bool>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },
    /// A Parameter: a constant that [`Scalar`] and [`Operand`] fields reference by id.
    Param {
        /// Its id.
        id: String,
        /// The value.
        value: f64,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        name: Option<String>,
        /// Passed through.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        group: Option<String>,
    },

    /// Two points at one place.
    Coincident {
        /// Its id.
        id: String,
        /// A point's id.
        a: String,
        /// A point's id.
        b: String,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A line, or the direction between two points, along x.
    Horizontal {
        /// Its id.
        id: String,
        /// The line, or the two points.
        #[serde(flatten)]
        of: LineOrPoints,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A line, or the direction between two points, along y.
    Vertical {
        /// Its id.
        id: String,
        /// The line, or the two points.
        #[serde(flatten)]
        of: LineOrPoints,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Two lines' directions parallel.
    Parallel {
        /// Its id.
        id: String,
        /// A line's id.
        a: String,
        /// A line's id.
        b: String,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Two lines' directions perpendicular.
    Perpendicular {
        /// Its id.
        id: String,
        /// A line's id.
        a: String,
        /// A line's id.
        b: String,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Two lines on one infinite line.
    Collinear {
        /// Its id.
        id: String,
        /// A line's id.
        a: String,
        /// A line's id.
        b: String,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A line along a circle's or arc's radius.
    Normal {
        /// Its id.
        id: String,
        /// The line's id.
        line: String,
        /// The circle's or arc's id.
        curve: String,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// The angle between two lines, or an arc's sweep (radians).
    Angle {
        /// Its id.
        id: String,
        /// The lines, or the arc.
        #[serde(flatten)]
        of: AngleOf,
        /// The angle.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// The direction of a line or from point `a` to `b`, from +x (radians, one root).
    Direction {
        /// Its id.
        id: String,
        /// The line, or the two points.
        #[serde(flatten)]
        of: LineOrPoints,
        /// The direction.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// The distance between two entities, in either order: point–point, point–line,
    /// point–circle/arc, line–circle/arc, circle/arc–circle/arc, line–line.
    Distance {
        /// Its id.
        id: String,
        /// An entity's id.
        a: String,
        /// An entity's id.
        b: String,
        /// The distance.
        value: Scalar,
        /// Measures against a line's infinite extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        extension: Option<bool>,
        /// Measures inside a circle or arc.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        internal: Option<bool>,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A point's signed distance from a line's extension, on its `side`.
    Offset {
        /// Its id.
        id: String,
        /// The point's id.
        point: String,
        /// The line's id.
        line: String,
        /// The distance.
        value: Scalar,
        /// < 0: right of `p1 → p2`; otherwise left.
        side: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A point on a line, circle, arc, ellipse, elliptical arc or spline.
    On {
        /// Its id.
        id: String,
        /// The point's id.
        point: String,
        /// The curve's id.
        curve: String,
        /// On a line's infinite extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        extension: Option<bool>,
        /// On a spline: where the contact sits along the curve. A response reports it;
        /// sending it back starts the next solve there.
        #[serde(rename = "curve_params", skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        curve_params: Option<Vec<f64>>,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A point (or line) at a line's midpoint, or a point at an arc's.
    Midpoint {
        /// Its id.
        id: String,
        /// `[point or line, line]` or `[point, arc]`.
        entities: Vec<String>,
        /// A line's midpoint on the other's infinite extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        extension: Option<bool>,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Two curves tangent, in either order.
    Tangent {
        /// Its id.
        id: String,
        /// A curve's id.
        a: String,
        /// A curve's id.
        b: String,
        /// Tangent to a line's infinite extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        extension: Option<bool>,
        /// Between circles or arcs: one inside the other.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        internal: Option<bool>,
        /// Against a spline: where the contact sits along each spline. A response reports
        /// it; sending it back starts the next solve there.
        #[serde(rename = "curve_params", skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        curve_params: Option<Vec<f64>>,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Two circles or arcs with one center.
    Concentric {
        /// Its id.
        id: String,
        /// A circle's or arc's id.
        a: String,
        /// A circle's or arc's id.
        b: String,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Two lines of one length, two circles or arcs of one radius, or two values.
    Equal {
        /// Its id.
        id: String,
        /// A line, circle or arc id, or a value.
        a: Operand,
        /// A line, circle or arc id, or a value.
        b: Operand,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A circle's or arc's radius.
    Radius {
        /// Its id.
        id: String,
        /// The circle's or arc's id.
        curve: String,
        /// The radius.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A circle's or arc's diameter.
    Diameter {
        /// Its id.
        id: String,
        /// The circle's or arc's id.
        curve: String,
        /// The diameter.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A line's or arc's length.
    Length {
        /// Its id.
        id: String,
        /// The line's or arc's id.
        curve: String,
        /// The length.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// `b − a = value`.
    Difference {
        /// Its id.
        id: String,
        /// The subtrahend.
        a: Operand,
        /// The minuend.
        b: Operand,
        /// The difference.
        value: Operand,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A point's x.
    X {
        /// Its id.
        id: String,
        /// The point's id.
        point: String,
        /// The x.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// A point's y.
    Y {
        /// Its id.
        id: String,
        /// The point's id.
        point: String,
        /// The y.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Signed `b − a` along x, of a line (`p2 − p1`) or two points.
    HorizontalDistance {
        /// Its id.
        id: String,
        /// The line, or the two points.
        #[serde(flatten)]
        of: LineOrPoints,
        /// The distance.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Signed `b − a` along y, of a line (`p2 − p1`) or two points.
    VerticalDistance {
        /// Its id.
        id: String,
        /// The line, or the two points.
        #[serde(flatten)]
        of: LineOrPoints,
        /// The distance.
        value: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// `image` is `source` mirrored across a line's extension (a Guide).
    Mirror {
        /// Its id.
        id: String,
        /// The source point's id.
        source: String,
        /// The image point's id.
        image: String,
        /// The axis line's id.
        axis: String,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// `copy` is `source` rotated about `center` (a Guide) by `angle × count`.
    Rotation {
        /// Its id.
        id: String,
        /// The source point's id.
        source: String,
        /// The copy's id.
        copy: String,
        /// The center point's id.
        center: String,
        /// The angle between neighbouring copies, radians.
        angle: Scalar,
        /// Which copy; 1 when absent.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        count: Option<Scalar>,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// `copy` is `source` moved `distance × count` along `from → to` (Guides).
    Translation {
        /// Its id.
        id: String,
        /// The source point's id.
        source: String,
        /// The copy's id.
        copy: String,
        /// The direction's start point id.
        from: String,
        /// The direction's end point id.
        to: String,
        /// The distance between neighbouring copies.
        distance: Scalar,
        /// Which copy.
        count: Scalar,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
    /// Points on an ellipse's or elliptical arc's axis.
    EllipseAxis {
        /// Its id.
        id: String,
        /// The ellipse's or elliptical arc's id.
        ellipse: String,
        /// One point at either end, or the pair at both.
        #[serde(flatten)]
        at: AxisPoints,
        /// Which axis.
        which: Axis,
        /// A soft goal.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "uniffi", uniffi(default = None))]
        temporary: Option<bool>,
    },
}

/// Whether the real constraints hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "snake_case")]
pub enum SolveStatus {
    /// Every real constraint holds within 1e-10.
    Converged,
    /// Some don't.
    Failed,
}

/// How the solve went, over all components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SolveStats {
    /// Iterations, summed.
    pub iterations: u32,
    /// The largest residual before, maxed.
    pub initial_error: f64,
    /// The largest residual after, maxed.
    pub final_error: f64,
}

/// A response: the request's primitives in order with solved values, and the analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SolveResponse {
    /// Always 1.
    pub version: u32,
    /// Whether the real constraints hold.
    pub status: SolveStatus,
    /// The request's primitives, same order, with solved values (and each spline's `curve`,
    /// each spline contact's `curve_params`).
    pub primitives: Vec<Primitive>,
    /// Ids of Conflicting constraints; absent when the request had `analyze: false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicting: Option<Vec<String>>,
    /// Ids of Redundant constraints; absent when not analysed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redundant: Option<Vec<String>>,
    /// Degrees of freedom left; absent when not analysed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dof: Option<u32>,
    /// Ids of entities with no freedom left (converged solves only); absent when not
    /// analysed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fully_constrained: Option<Vec<String>>,
    /// How the solve went.
    pub stats: SolveStats,
}

/// A request the solver can't read, or handles that make no curve.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SketchError {
    /// The JSON API's `status: "invalid"`: `message` is its `error`.
    Invalid {
        /// What's wrong: `constraint k1: 'ghost' is not a point`.
        message: String,
        /// The constraint it's about, if any.
        constraint_id: Option<String>,
    },
}

impl std::fmt::Display for SketchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SketchError::Invalid { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for SketchError {}

impl SketchError {
    fn invalid(message: impl Into<String>) -> Self {
        SketchError::Invalid {
            message: message.into(),
            constraint_id: None,
        }
    }
}

/// Solves a sketch, as [`solve_sketch_json`] solves its JSON. Stateless.
///
/// [`solve_sketch_json`]: crate::sketch_solve::solve_sketch_json
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn solve(request: SolveRequest) -> Result<SolveResponse, SketchError> {
    let json = serde_json::to_value(&request).map_err(|e| SketchError::invalid(e.to_string()))?;
    let response = solve_value(&json).map_err(|r| SketchError::Invalid {
        message: r.error,
        constraint_id: r.constraint_id,
    })?;
    serde_json::from_value(response).map_err(|e| SketchError::invalid(format!("unreadable response: {e}")))
}

/// A Spline's curve from its handles' positions, without solving: the `curve` a solve reports
/// for a `spline` with these handles (see [`spline_curve_json`]).
///
/// [`spline_curve_json`]: crate::sketch_solve::spline_curve_json
#[cfg_attr(feature = "uniffi", uniffi::export(default(knots = None)))]
pub fn spline_curve(handles: Vec<Xy>, interpolated: bool, knots: Option<Vec<f64>>) -> Result<Curve, SketchError> {
    let handles: Vec<[f64; 2]> = handles.iter().map(|h| [h.x, h.y]).collect();
    BSpline::new(&handles, interpolated, knots.as_deref())
        .map(Curve::from)
        .map_err(SketchError::invalid)
}
