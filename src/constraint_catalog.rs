//! Every constraint type the JSON API accepts, declared once.
//!
//! ACS's constraint vocabulary ([`native`]) has one type per relationship
//! (`distance`, `on`, `tangent`, …) with role-named fields. A type may have
//! several rows (variants); the one used is inferred from the kinds of the
//! entities its fields reference, `extension: true` selects an
//! Extension variant and `internal: true` an inside variant.
//!
//! Each row ([`ConstraintSpec`]) names a JSON `type`, its fields with their
//! kinds, and how to build an internal [`ConstraintType`] from them. Field
//! resolution is shared: a Line field expands to its two endpoint Points, a
//! Circle or Arc field also resolves its center Point and an Ellipse field
//! its center and focus Points; a Scalar field takes
//! a number or a Parameter id; a Value field also takes a property reference,
//! which becomes a solver variable. The JSON API, the catalog returned by
//! [`catalog_json`] and the Jacobian test all read this table, so
//! supporting a constraint means adding a row there.
//!
//! A field name `name[i]` addresses element `i` of the array field `name`
//! (native `midpoint`'s `entities`); see [`field_path`].

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::{ArcEnd, ConstraintType, EllipseAxis, Operand};

pub mod native;

/// Every row of the catalog.
pub fn specs() -> &'static [ConstraintSpec] {
    native::SPECS
}

/// Builds the constraint a JSON constraint object of type `json_type`
/// describes. Errors name an unknown type, a missing field, an unknown
/// reference or an unsupported combination of entity kinds.
pub fn parse(json_type: &str, c: &Value, refs: &References) -> Result<ConstraintType, String> {
    native::parse(json_type, c, refs)
}

/// The catalog as JSON, one object per row:
/// `[{ "type", "fields": [{ "name", "index"?, "kind" }], "extension"?,
/// "internal"? }]`. A type with several variants has several rows;
/// `extension` is present on rows that take the flag (`false`: the segment
/// variant, `true`: the Extension variant), and `internal` likewise
/// (`false`: outside, `true`: the inside variant). A field with an `index`
/// is that element of the array field `name`.
pub fn catalog_json() -> String {
    let specs: Vec<Value> = specs()
        .iter()
        .map(|s| {
            let fields: Vec<Value> = s
                .fields
                .iter()
                .map(|&(name, kind)| match field_path(name) {
                    (name, None) => json!({ "name": name, "kind": kind.as_str() }),
                    (name, Some(i)) => {
                        json!({ "name": name, "index": i, "kind": kind.as_str() })
                    }
                })
                .collect();
            let mut row = json!({ "type": s.json_type, "fields": fields });
            match s.extension {
                ExtensionFlag::NotAccepted => {}
                ExtensionFlag::Segment => row["extension"] = json!(false),
                ExtensionFlag::Extension => row["extension"] = json!(true),
            }
            match s.internal {
                InternalFlag::NotAccepted => {}
                InternalFlag::External => row["internal"] = json!(false),
                InternalFlag::Internal => row["internal"] = json!(true),
            }
            row
        })
        .collect();
    Value::Array(specs).to_string()
}

/// A field name split into the JSON key it reads and, for `name[i]`, the
/// index of the array element: `"entities[1]"` → `("entities", Some(1))`,
/// `"line"` → `("line", None)`.
pub fn field_path(name: &str) -> (&str, Option<usize>) {
    name.strip_suffix(']')
        .and_then(|rest| rest.split_once('['))
        .and_then(|(key, i)| Some((key, Some(i.parse().ok()?))))
        .unwrap_or((name, None))
}

/// The value of field `name` of constraint `c`, reading into an array for
/// `name[i]` (see [`field_path`]).
pub fn field_value<'v>(c: &'v Value, name: &str) -> Option<&'v Value> {
    match field_path(name) {
        (key, None) => c.get(key),
        (key, Some(i)) => c.get(key)?.as_array()?.get(i),
    }
}

/// Rewrites, among a sketch's constraints, each real Line–circle or Line–arc
/// tangency whose Line has an endpoint a real `on` holds on that same curve
/// into tangency at that endpoint (`TangentAtPoint`: the Line perpendicular
/// to the radius there). Through a point already on the curve, the distance
/// form changes only quadratically as the endpoint slides along the Line:
/// the endpoint drifts micrometres from the tangent point and diagnosis
/// reports spurious Redundant constraints (issue #14). Both forms have the
/// same solutions, segment or Extension. If both endpoints are held, the
/// Line's first endpoint is used. `constraints` pairs each constraint with
/// whether it is temporary; temporaries neither trigger nor get the rewrite.
pub fn tangent_at_held_endpoints(constraints: &mut [(ConstraintType, bool)]) {
    let held: std::collections::HashSet<(String, String)> = constraints
        .iter()
        .filter(|(_, temporary)| !temporary)
        .filter_map(|(ct, _)| match ct {
            ConstraintType::PointOnCircle(p, _, curve) | ConstraintType::PointOnArc(p, _, curve) => {
                Some((p.clone(), curve.clone()))
            }
            _ => None,
        })
        .collect();
    let is_held = |p: &String, curve: &String| held.contains(&(p.clone(), curve.clone()));
    for (ct, temporary) in constraints.iter_mut() {
        if *temporary {
            continue;
        }
        let (a, b, center, curve) = match &*ct {
            ConstraintType::TangentLineCircle(a, b, center, curve)
            | ConstraintType::TangentExtensionCircle(a, b, center, curve)
            | ConstraintType::TangentLineArc(a, b, center, curve) => (a, b, center, curve),
            _ => continue,
        };
        let rewritten = if is_held(a, curve) {
            ConstraintType::TangentAtPoint(a.clone(), b.clone(), center.clone())
        } else if is_held(b, curve) {
            ConstraintType::TangentAtPoint(b.clone(), a.clone(), center.clone())
        } else {
            continue;
        };
        *ct = rewritten;
    }
}

/// What a JSON constraint field refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Point,
    /// Expands to the Line's two endpoint Points.
    Line,
    /// A Circle; also resolves its center Point.
    Circle,
    /// An Arc; also resolves its center, start and end Points.
    Arc,
    /// An Ellipse; also resolves its center and focus Points.
    Ellipse,
    /// An EllipticalArc; resolves like an Ellipse (center and focus Points),
    /// plus its start and end Points.
    EllipticalArc,
    /// One of an Ellipse's axes: the string `major` or `minor`.
    Axis,
    /// A constant: a number, a numeric string or a Parameter id.
    Scalar,
    /// A Scalar, or an `{entity, property}` reference to an entity property
    /// the solver may move (see [`property_operand`]).
    Value,
}

impl FieldKind {
    /// Whether the field references an entity by ID (rather than a number).
    pub fn is_entity(self) -> bool {
        !matches!(self, FieldKind::Scalar | FieldKind::Value | FieldKind::Axis)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            FieldKind::Point => "point",
            FieldKind::Line => "line",
            FieldKind::Circle => "circle",
            FieldKind::Arc => "arc",
            FieldKind::Ellipse => "ellipse",
            FieldKind::EllipticalArc => "elliptical_arc",
            FieldKind::Axis => "axis",
            FieldKind::Scalar => "scalar",
            FieldKind::Value => "value",
        }
    }
}

/// A constraint's fields resolved to solver IDs and numbers, each list in
/// field order. A Line field contributes two entries to `points`; a Circle or
/// Arc field one entry each to `circles` and `centers`. An Arc field also
/// adds its start and end Points to `arc_ends`. An Ellipse field adds one
/// entry to `ellipses`, an Axis field one to `axes`. An EllipticalArc field
/// adds one entry to `ellipses` (it is read through the same kernels) and
/// one to `elliptical_arc_ends`.
#[derive(Debug, Default, Clone)]
pub struct Args {
    pub points: Vec<String>,
    pub circles: Vec<String>,
    pub centers: Vec<String>,
    pub scalars: Vec<f64>,
    pub operands: Vec<Operand>,
    /// (start Point ID, end Point ID) of each Arc field, in field order.
    pub arc_ends: Vec<(String, String)>,
    /// Each Ellipse or EllipticalArc field, in field order.
    pub ellipses: Vec<EllipseRef>,
    /// (start Point ID, end Point ID) of each EllipticalArc field, in field
    /// order.
    pub elliptical_arc_ends: Vec<(String, String)>,
    /// Each Axis field, in field order.
    pub axes: Vec<EllipseAxis>,
}

/// An Ellipse field resolved: the ellipse and its center and focus Points.
#[derive(Debug, Clone)]
pub struct EllipseRef {
    pub id: String,
    pub center: String,
    pub focus: String,
}

impl Args {
    fn p(&self, i: usize) -> String {
        self.points[i].clone()
    }
    fn c(&self, i: usize) -> String {
        self.circles[i].clone()
    }
    fn center(&self, i: usize) -> String {
        self.centers[i].clone()
    }
    fn s(&self, i: usize) -> f64 {
        self.scalars[i]
    }
    fn o(&self, i: usize) -> Operand {
        self.operands[i].clone()
    }
    fn arc_start(&self, i: usize) -> String {
        self.arc_ends[i].0.clone()
    }
    fn arc_end(&self, i: usize) -> String {
        self.arc_ends[i].1.clone()
    }
    /// The Line (points 0 and 1) tangent to Arc field 0. When a line
    /// endpoint is one of the arc's endpoints, tangency is the angle at that
    /// shared point (`TangentAtPoint`); measuring it by distance there is
    /// degenerate. Otherwise, by distance (`TangentLineArc`).
    fn tangent_line_arc(&self) -> ConstraintType {
        let (a, b, center) = (self.p(0), self.p(1), self.center(0));
        let ends = [self.arc_start(0), self.arc_end(0)];
        if ends.contains(&a) {
            ConstraintType::TangentAtPoint(a, b, center)
        } else if ends.contains(&b) {
            ConstraintType::TangentAtPoint(b, a, center)
        } else {
            ConstraintType::TangentLineArc(a, b, center, self.c(0))
        }
    }
    /// Arc fields 0 and 1 tangent, inside when `internal`. When the arcs
    /// share an endpoint, tangency is the angle between their radii at that
    /// point (`TangentArcsAtPoint`); measuring it by distance there is
    /// degenerate, as for `tangent_line_arc`. Otherwise, by distance
    /// (`TangentArcs`).
    fn tangent_arcs(&self, internal: bool) -> ConstraintType {
        let (s0, e0) = (self.arc_start(0), self.arc_end(0));
        let shared = [self.arc_start(1), self.arc_end(1)]
            .into_iter()
            .find(|p| *p == s0 || *p == e0);
        match shared {
            Some(p) => ConstraintType::TangentArcsAtPoint(p, self.center(0), self.center(1), internal),
            None => ConstraintType::TangentArcs(
                self.center(0),
                self.c(0),
                self.center(1),
                self.c(1),
                internal,
            ),
        }
    }
    /// (center, focus, ellipse) IDs of Ellipse field `i`.
    fn e(&self, i: usize) -> (String, String, String) {
        let e = &self.ellipses[i];
        (e.center.clone(), e.focus.clone(), e.id.clone())
    }
    fn axis(&self, i: usize) -> EllipseAxis {
        self.axes[i]
    }
    /// The endpoint of EllipticalArc field 0 that `p` is, if it is one.
    fn elliptical_arc_end(&self, p: &str) -> Option<ArcEnd> {
        let (start, end) = &self.elliptical_arc_ends[0];
        if p == start {
            Some(ArcEnd::Start)
        } else if p == end {
            Some(ArcEnd::End)
        } else {
            None
        }
    }
    /// The Line (points 0 and 1) tangent to EllipticalArc field 0 (its
    /// ellipse is `ellipses[0]`). When a line endpoint is one of the
    /// elliptical arc's endpoints, along its tangent there
    /// (`TangentLineEllipticalArcAtPoint`), as for `tangent_line_arc`;
    /// otherwise touching the segment and the span
    /// (`TangentLineEllipticalArc`).
    fn tangent_line_elliptical_arc(&self) -> ConstraintType {
        let (a, b) = (self.p(0), self.p(1));
        let (center, focus, e) = self.e(0);
        if let Some(end) = self.elliptical_arc_end(&a) {
            ConstraintType::TangentLineEllipticalArcAtPoint(a, b, center, focus, e, end)
        } else if let Some(end) = self.elliptical_arc_end(&b) {
            ConstraintType::TangentLineEllipticalArcAtPoint(b, a, center, focus, e, end)
        } else {
            ConstraintType::TangentLineEllipticalArc(a, b, center, focus, e)
        }
    }
    /// The endpoint Arc field 0 shares with EllipticalArc field 0, and which
    /// end of the elliptical arc it is.
    fn arc_elliptical_arc_shared(&self) -> Option<(String, ArcEnd)> {
        [self.arc_start(0), self.arc_end(0)]
            .into_iter()
            .find_map(|p| self.elliptical_arc_end(&p).map(|end| (p, end)))
    }
    /// Arc field 0 tangent to EllipticalArc field 0 at the endpoint they
    /// share (`TangentArcEllipticalArcAtPoint`). Only that form exists (the
    /// row's check rejects a pair with no shared endpoint); without one,
    /// the elliptical arc's start stands in.
    fn tangent_arc_elliptical_arc(&self) -> ConstraintType {
        let (center, focus, e) = self.e(0);
        let (p, end) = self
            .arc_elliptical_arc_shared()
            .unwrap_or_else(|| (self.elliptical_arc_ends[0].0.clone(), ArcEnd::Start));
        ConstraintType::TangentArcEllipticalArcAtPoint(p, self.center(0), center, focus, e, end)
    }
}

/// The check of the `tangent` arc–elliptical-arc row: they must share an
/// endpoint.
pub(crate) fn arc_and_elliptical_arc_share_an_endpoint(args: &Args) -> Result<(), String> {
    match args.arc_elliptical_arc_shared() {
        Some(_) => Ok(()),
        None => Err("an arc and an elliptical arc are tangent only at an endpoint they share".into()),
    }
}

/// What constraint fields resolve against: Lines, circle/arc centers,
/// Parameters and the entities property references may address.
#[derive(Debug, Default)]
pub struct References {
    /// Line ID → (start Point ID, end Point ID).
    pub line_endpoints: HashMap<String, (String, String)>,
    /// Circle or Arc ID → center Point ID.
    pub centers: HashMap<String, String>,
    /// Parameter ID → its value.
    pub params: HashMap<String, f64>,
    /// Entity ID → its JSON geometry type (`point`, `line`, `circle`, …).
    pub entity_types: HashMap<String, String>,
    /// Arc ID → (start Point ID, end Point ID).
    pub arc_endpoints: HashMap<String, (String, String)>,
    /// Ellipse ID → (center Point ID, focus Point ID).
    pub ellipses: HashMap<String, (String, String)>,
    /// EllipticalArc ID → its Points.
    pub elliptical_arcs: HashMap<String, EllipticalArcPoints>,
}

/// The Point IDs an EllipticalArc references.
#[derive(Debug, Clone)]
pub struct EllipticalArcPoints {
    pub center: String,
    pub focus: String,
    pub start: String,
    pub end: String,
}

impl References {
    /// A Scalar field: a number, a Parameter id, or a numeric string.
    fn scalar(&self, name: &str, v: &Value) -> Result<f64, String> {
        if let Some(n) = v.as_f64() {
            return Ok(n);
        }
        let Some(s) = v.as_str() else {
            return Err(format!("field '{name}' is not a number or a Parameter id"));
        };
        if let Some(&n) = self.params.get(s) {
            return Ok(n);
        }
        s.parse()
            .map_err(|_| format!("field '{name}': unknown Parameter '{s}'"))
    }

    /// A Value field: a Scalar, or a property reference
    /// `{entity, property}`.
    fn operand(&self, name: &str, v: &Value) -> Result<Operand, String> {
        let Some(obj) = v.as_object() else {
            return self.scalar(name, v).map(Operand::Const);
        };
        let field = |key: &str| {
            obj.get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("field '{name}': property reference has no '{key}'"))
        };
        let (entity, prop) = (field("entity")?, field("property")?);
        let kind = self
            .entity_types
            .get(entity)
            .ok_or_else(|| format!("field '{name}': '{entity}' is not an entity"))?;
        property_operand(kind, entity, prop)
            .ok_or_else(|| format!("field '{name}': {kind} '{entity}' has no property '{prop}'"))
    }
}

/// The solver variable behind property `prop` of the `kind` entity `id`:
/// a Point's `x` and `y`, a Circle's or Arc's `radius`, an Ellipse's
/// `radmin`.
pub fn property_operand(kind: &str, id: &str, prop: &str) -> Option<Operand> {
    let id = id.to_string();
    match (kind, prop) {
        ("point", "x") => Some(Operand::X(id)),
        ("point", "y") => Some(Operand::Y(id)),
        ("circle" | "arc", "radius") => Some(Operand::Radius(id)),
        ("ellipse" | "elliptical_arc", "radmin") => Some(Operand::MinorRadius(id)),
        _ => None,
    }
}

/// Whether a row takes the native `extension` flag, and which value selects it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionFlag {
    /// The row has no Extension variant; `extension: true` doesn't select it.
    NotAccepted,
    /// The segment variant: `extension` absent or `false`.
    Segment,
    /// The Extension variant: `extension: true`.
    Extension,
}

impl ExtensionFlag {
    /// Whether a constraint with `extension` set as given can select this row.
    pub fn accepts(self, extension: bool) -> bool {
        match self {
            ExtensionFlag::NotAccepted | ExtensionFlag::Segment => !extension,
            ExtensionFlag::Extension => extension,
        }
    }
}

/// Whether a row takes the native `internal` flag (tangency between circles
/// and arcs; `distance` point–circle and circle–circle), and which value
/// selects it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalFlag {
    /// The row has no inside variant; `internal: true` doesn't select it.
    NotAccepted,
    /// The outside variant (external tangency, a gap outside the circle):
    /// `internal` absent or `false`.
    External,
    /// The inside variant: `internal: true`.
    Internal,
}

impl InternalFlag {
    /// Whether a constraint with `internal` set as given can select this row.
    pub fn accepts(self, internal: bool) -> bool {
        match self {
            InternalFlag::NotAccepted | InternalFlag::External => !internal,
            InternalFlag::Internal => internal,
        }
    }
}

/// A row's test of its resolved fields (see [`ConstraintSpec::parse`]).
pub type Check = fn(&Args) -> Result<(), String>;

/// One row of a catalog: a JSON `type` with one set of field kinds.
pub struct ConstraintSpec {
    pub json_type: &'static str,
    /// (JSON field name, kind), in order.
    pub fields: &'static [(&'static str, FieldKind)],
    pub extension: ExtensionFlag,
    pub internal: InternalFlag,
    /// A test of the resolved fields a row can only pass or fail once it
    /// sees which entities they are (e.g. a shared endpoint), run before
    /// `build`.
    check: Option<Check>,
    build: fn(&Args) -> ConstraintType,
}

impl ConstraintSpec {
    pub const fn new(
        json_type: &'static str,
        fields: &'static [(&'static str, FieldKind)],
        build: fn(&Args) -> ConstraintType,
    ) -> Self {
        ConstraintSpec {
            json_type,
            fields,
            extension: ExtensionFlag::NotAccepted,
            internal: InternalFlag::NotAccepted,
            check: None,
            build,
        }
    }

    /// This row with a check of its resolved fields (see [`Self::parse`]).
    pub const fn check(self, check: Check) -> Self {
        ConstraintSpec {
            check: Some(check),
            ..self
        }
    }

    /// This row as the segment variant of a type that has an Extension one.
    pub const fn segment(self) -> Self {
        ConstraintSpec {
            extension: ExtensionFlag::Segment,
            ..self
        }
    }

    /// This row as the Extension variant (`extension: true`).
    pub const fn extension(self) -> Self {
        ConstraintSpec {
            extension: ExtensionFlag::Extension,
            ..self
        }
    }

    /// This row as the external variant of a tangency that has an inside
    /// one (`internal` absent or `false`).
    pub const fn external(self) -> Self {
        ConstraintSpec {
            internal: InternalFlag::External,
            ..self
        }
    }

    /// This row as the inside variant (`internal: true`).
    pub const fn internal(self) -> Self {
        ConstraintSpec {
            internal: InternalFlag::Internal,
            ..self
        }
    }

    pub fn build(&self, args: &Args) -> ConstraintType {
        (self.build)(args)
    }

    /// Resolves a JSON constraint object's fields against `refs`, runs the
    /// row's check, if any, and builds the constraint. Errors name the
    /// missing field, unknown reference or failed check.
    pub fn parse(&self, c: &Value, refs: &References) -> Result<ConstraintType, String> {
        let mut args = Args::default();
        for &(name, kind) in self.fields {
            let field = field_value(c, name).ok_or_else(|| format!("missing field '{name}'"))?;
            match kind {
                FieldKind::Scalar => {
                    args.scalars.push(refs.scalar(name, field)?);
                    continue;
                }
                FieldKind::Value => {
                    args.operands.push(refs.operand(name, field)?);
                    continue;
                }
                FieldKind::Axis => {
                    args.axes.push(match field.as_str() {
                        Some("major") => EllipseAxis::Major,
                        Some("minor") => EllipseAxis::Minor,
                        _ => return Err(format!("field '{name}' is not \"major\" or \"minor\"")),
                    });
                    continue;
                }
                _ => {}
            }
            let id = field
                .as_str()
                .ok_or_else(|| format!("field '{name}' is not an ID"))?
                .to_string();
            match kind {
                FieldKind::Point => args.points.push(id),
                FieldKind::Line => {
                    let (a, b) = refs
                        .line_endpoints
                        .get(&id)
                        .ok_or_else(|| format!("'{id}' is not a line"))?;
                    args.points.push(a.clone());
                    args.points.push(b.clone());
                }
                FieldKind::Circle | FieldKind::Arc => {
                    let center = refs
                        .centers
                        .get(&id)
                        .cloned()
                        .ok_or_else(|| format!("'{id}' is not a circle or arc"))?;
                    if kind == FieldKind::Arc {
                        let ends = refs
                            .arc_endpoints
                            .get(&id)
                            .cloned()
                            .ok_or_else(|| format!("'{id}' is not an arc"))?;
                        args.arc_ends.push(ends);
                    }
                    args.circles.push(id);
                    args.centers.push(center);
                }
                FieldKind::Ellipse => {
                    let (center, focus) = refs
                        .ellipses
                        .get(&id)
                        .cloned()
                        .ok_or_else(|| format!("'{id}' is not an ellipse"))?;
                    args.ellipses.push(EllipseRef { id, center, focus });
                }
                FieldKind::EllipticalArc => {
                    let points = refs
                        .elliptical_arcs
                        .get(&id)
                        .cloned()
                        .ok_or_else(|| format!("'{id}' is not an elliptical arc"))?;
                    args.ellipses.push(EllipseRef {
                        id,
                        center: points.center,
                        focus: points.focus,
                    });
                    args.elliptical_arc_ends.push((points.start, points.end));
                }
                FieldKind::Scalar | FieldKind::Value | FieldKind::Axis => unreachable!(),
            }
        }
        if let Some(check) = self.check {
            check(&args)?;
        }
        Ok(self.build(&args))
    }
}

