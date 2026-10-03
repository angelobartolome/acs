//! Every constraint type the JSON API accepts, declared once per vocabulary.
//!
//! ACS reads constraints in two vocabularies (see [`Vocabulary`]):
//!
//! - **Native** ([`native`]): ACS's own, one type per relationship
//!   (`distance`, `on`, `tangent`, …) with role-named fields. A type may have
//!   several rows (variants); the one used is inferred from the kinds of the
//!   entities its fields reference, and `extension: true` selects an
//!   Extension variant.
//! - **PlaneGCS dialect** ([`planegcs`]): the GCS types, unchanged
//!   (`p2p_distance`, `horizontal_l`, …), one row each.
//!
//! Each row ([`ConstraintSpec`]) names a JSON `type`, its fields with their
//! kinds, and how to build an internal [`ConstraintType`] from them. Field
//! resolution is shared: a Line field expands to its two endpoint Points, a
//! Circle or Arc field also resolves its center Point and an Ellipse field
//! its center and focus Points; a Scalar field takes
//! a number or a Parameter id; a Value field also takes a property reference,
//! which becomes a solver variable. The JSON API, the catalogs returned by
//! [`Vocabulary::catalog_json`] and the Jacobian test all read these tables,
//! so supporting a constraint in a vocabulary means adding a row there.

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::{ConstraintType, EllipseAxis, Operand};

pub mod native;
pub mod planegcs;

/// Which constraint vocabulary a request speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Vocabulary {
    /// ACS's own: `acsSolveSketch`, `solve_sketch_json`.
    #[default]
    Native,
    /// The PlaneGCS dialect: `P3DSketch_Solve`,
    /// `acsSolveSketchPlaneGcs`.
    PlaneGcs,
}

impl Vocabulary {
    /// Every row of this vocabulary's catalog.
    pub fn specs(self) -> &'static [ConstraintSpec] {
        match self {
            Vocabulary::Native => native::SPECS,
            Vocabulary::PlaneGcs => planegcs::SPECS,
        }
    }

    /// The keys of a property reference: native `{ entity, property }`,
    /// dialect `{ o_id, prop }`.
    fn property_keys(self) -> (&'static str, &'static str) {
        match self {
            Vocabulary::Native => ("entity", "property"),
            Vocabulary::PlaneGcs => ("o_id", "prop"),
        }
    }

    /// Builds the constraint a JSON constraint object of type `json_type`
    /// describes. Errors name an unknown type, a missing field, an unknown
    /// reference or (native) an unsupported combination of entity kinds.
    pub fn parse(
        self,
        json_type: &str,
        c: &Value,
        refs: &References,
    ) -> Result<ConstraintType, String> {
        match self {
            Vocabulary::Native => native::parse(json_type, c, refs),
            Vocabulary::PlaneGcs => self
                .specs()
                .iter()
                .find(|s| s.json_type == json_type)
                .ok_or_else(|| format!("unknown type '{json_type}'"))?
                .parse(c, refs),
        }
    }

    /// The catalog as JSON, one object per row:
    /// `[{ "type", "fields": [{ "name", "kind" }], "extension"? }]`. A
    /// native type with several variants has several rows; `extension` is
    /// present on rows that take the flag (`false`: the segment variant,
    /// `true`: the Extension variant).
    pub fn catalog_json(self) -> String {
        let specs: Vec<Value> = self
            .specs()
            .iter()
            .map(|s| {
                let fields: Vec<Value> = s
                    .fields
                    .iter()
                    .map(|&(name, kind)| json!({ "name": name, "kind": kind.as_str() }))
                    .collect();
                let mut row = json!({ "type": s.json_type, "fields": fields });
                match s.extension {
                    ExtensionFlag::NotAccepted => {}
                    ExtensionFlag::Segment => row["extension"] = json!(false),
                    ExtensionFlag::Extension => row["extension"] = json!(true),
                }
                row
            })
            .collect();
        Value::Array(specs).to_string()
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
    /// One of an Ellipse's axes: the string `major` or `minor`.
    Axis,
    /// A constant: a number, a numeric string or a Parameter id.
    Scalar,
    /// A Scalar, or an `{o_id, prop}` reference to an entity property the
    /// solver may move (see [`property_operand`]).
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
/// entry to `ellipses`, an Axis field one to `axes`.
#[derive(Debug, Default, Clone)]
pub struct Args {
    pub points: Vec<String>,
    pub circles: Vec<String>,
    pub centers: Vec<String>,
    pub scalars: Vec<f64>,
    pub operands: Vec<Operand>,
    /// (start Point ID, end Point ID) of each Arc field, in field order.
    pub arc_ends: Vec<(String, String)>,
    /// Each Ellipse field, in field order.
    pub ellipses: Vec<EllipseRef>,
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
    /// (center, focus, ellipse) IDs of Ellipse field `i`.
    fn e(&self, i: usize) -> (String, String, String) {
        let e = &self.ellipses[i];
        (e.center.clone(), e.focus.clone(), e.id.clone())
    }
    fn axis(&self, i: usize) -> EllipseAxis {
        self.axes[i]
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
    /// The vocabulary the request speaks, which names the keys of property
    /// references.
    pub vocabulary: Vocabulary,
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

    /// A Value field: a Scalar, or a property reference (native
    /// `{entity, property}`, dialect `{o_id, prop}`).
    fn operand(&self, name: &str, v: &Value) -> Result<Operand, String> {
        let Some(obj) = v.as_object() else {
            return self.scalar(name, v).map(Operand::Const);
        };
        let field = |key: &str| {
            obj.get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("field '{name}': property reference has no '{key}'"))
        };
        let (entity_key, property_key) = self.vocabulary.property_keys();
        let (o_id, prop) = (field(entity_key)?, field(property_key)?);
        let kind = self
            .entity_types
            .get(o_id)
            .ok_or_else(|| format!("field '{name}': '{o_id}' is not an entity"))?;
        property_operand(kind, o_id, prop)
            .ok_or_else(|| format!("field '{name}': {kind} '{o_id}' has no property '{prop}'"))
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
        ("ellipse", "radmin") => Some(Operand::MinorRadius(id)),
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

/// One row of a catalog: a JSON `type` with one set of field kinds.
pub struct ConstraintSpec {
    pub json_type: &'static str,
    /// (JSON field name, kind), in order.
    pub fields: &'static [(&'static str, FieldKind)],
    pub extension: ExtensionFlag,
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
            build,
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

    pub fn build(&self, args: &Args) -> ConstraintType {
        (self.build)(args)
    }

    /// Resolves a JSON constraint object's fields against `refs` and builds
    /// the constraint. An explicit `center_id` overrides the center of the
    /// `c_id` circle. Errors name the missing field or unknown reference.
    pub fn parse(&self, c: &Value, refs: &References) -> Result<ConstraintType, String> {
        let mut args = Args::default();
        for &(name, kind) in self.fields {
            let field = c
                .get(name)
                .ok_or_else(|| format!("missing field '{name}'"))?;
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
                    let explicit = (name == "c_id")
                        .then(|| c.get("center_id").and_then(Value::as_str))
                        .flatten();
                    let center = match explicit {
                        Some(center) => center.to_string(),
                        None => refs
                            .centers
                            .get(&id)
                            .cloned()
                            .ok_or_else(|| format!("'{id}' is not a circle or arc"))?,
                    };
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
                FieldKind::Scalar | FieldKind::Value | FieldKind::Axis => unreachable!(),
            }
        }
        Ok(self.build(&args))
    }
}

