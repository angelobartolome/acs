use serde::{Deserialize, Serialize};
use crate::geometry::{Point, Circle, Line, Arc};
use crate::constraints::ConstraintType;
use crate::solver::SolverResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PrimitiveJson {
    Point {
        id: String,
        x: f64,
        y: f64,
        fixed: bool,
    },
    Circle {
        id: String,
        center: String,
        radius: f64,
        fixed: bool,
    },
    Line {
        id: String,
        start: String,
        end: String,
    },
    Arc {
        id: String,
        center: String,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        fixed: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConstraintJson {
    Vertical {
        point_a: String,
        point_b: String,
    },
    Horizontal {
        point_a: String,
        point_b: String,
    },
    Parallel {
        point_a: String,
        point_b: String,
        point_c: String,
        point_d: String,
    },
    EqualX {
        point: String,
        x: f64,
    },
    EqualY {
        point: String,
        y: f64,
    },
    Coincident {
        point_a: String,
        point_b: String,
    },
    PointOnLine {
        point: String,
        point_line_a: String,
        point_line_b: String,
    },
    EqualRadius {
        circle1: String,
        circle2: String,
    },
    FixedRadius {
        circle: String,
        radius: f64,
    },
    PointOnCircle {
        point: String,
        circle: String,
    },
    Tangent {
        entity1: String,
        entity2: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverRequest {
    pub primitives: Vec<PrimitiveJson>,
    pub constraints: Vec<ConstraintJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverResultJson {
    pub converged: bool,
    pub iterations: usize,
    pub final_error: f64,
    pub initial_error: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverResponse {
    pub primitives: Vec<PrimitiveJson>,
    pub result: SolverResultJson,
}

// Conversion implementations: From internal types to JSON types
impl From<Point> for PrimitiveJson {
    fn from(point: Point) -> Self {
        PrimitiveJson::Point {
            id: point.id,
            x: point.x,
            y: point.y,
            fixed: point.fixed,
        }
    }
}

impl From<Circle> for PrimitiveJson {
    fn from(circle: Circle) -> Self {
        PrimitiveJson::Circle {
            id: circle.id,
            center: circle.center,
            radius: circle.radius,
            fixed: circle.fixed,
        }
    }
}

impl From<Line> for PrimitiveJson {
    fn from(line: Line) -> Self {
        PrimitiveJson::Line {
            id: line.id,
            start: line.start,
            end: line.end,
        }
    }
}

impl From<Arc> for PrimitiveJson {
    fn from(arc: Arc) -> Self {
        PrimitiveJson::Arc {
            id: arc.id,
            center: arc.center,
            radius: arc.radius,
            start_angle: arc.start_angle,
            end_angle: arc.end_angle,
            fixed: arc.fixed,
        }
    }
}

impl From<ConstraintType> for ConstraintJson {
    fn from(constraint: ConstraintType) -> Self {
        match constraint {
            ConstraintType::Vertical(p1, p2) => ConstraintJson::Vertical {
                point_a: p1,
                point_b: p2,
            },
            ConstraintType::Horizontal(p1, p2) => ConstraintJson::Horizontal {
                point_a: p1,
                point_b: p2,
            },
            ConstraintType::Parallel(p1, p2, p3, p4) => ConstraintJson::Parallel {
                point_a: p1,
                point_b: p2,
                point_c: p3,
                point_d: p4,
            },
            ConstraintType::EqualX(p, x) => ConstraintJson::EqualX {
                point: p,
                x,
            },
            ConstraintType::EqualY(p, y) => ConstraintJson::EqualY {
                point: p,
                y,
            },
            ConstraintType::Coincident(p1, p2) => ConstraintJson::Coincident {
                point_a: p1,
                point_b: p2,
            },
            ConstraintType::PointOnLine(p, p_line_a, p_line_b) => ConstraintJson::PointOnLine {
                point: p,
                point_line_a: p_line_a,
                point_line_b: p_line_b,
            },
            ConstraintType::EqualRadius(c1, c2) => ConstraintJson::EqualRadius {
                circle1: c1,
                circle2: c2,
            },
            ConstraintType::FixedRadius(c, r) => ConstraintJson::FixedRadius {
                circle: c,
                radius: r,
            },
            ConstraintType::PointOnCircle(p, c) => ConstraintJson::PointOnCircle {
                point: p,
                circle: c,
            },
            ConstraintType::Tangent(e1, e2) => ConstraintJson::Tangent {
                entity1: e1,
                entity2: e2,
            },
        }
    }
}

impl From<SolverResult> for SolverResultJson {
    fn from(result: SolverResult) -> Self {
        match result {
            SolverResult::Converged {
                iterations,
                final_error,
                initial_error,
            } => SolverResultJson {
                converged: true,
                iterations,
                final_error,
                initial_error,
            },
            SolverResult::MaxIterationsReached {
                iterations,
                final_error,
                initial_error,
            } => SolverResultJson {
                converged: false,
                iterations,
                final_error,
                initial_error,
            },
        }
    }
}

// Conversion implementations: From JSON types to internal types
impl TryFrom<PrimitiveJson> for Point {
    type Error = String;

    fn try_from(primitive: PrimitiveJson) -> Result<Self, Self::Error> {
        match primitive {
            PrimitiveJson::Point { id, x, y, fixed } => Ok(Point { id, x, y, fixed }),
            _ => Err("Expected Point primitive".to_string()),
        }
    }
}

impl TryFrom<PrimitiveJson> for Circle {
    type Error = String;

    fn try_from(primitive: PrimitiveJson) -> Result<Self, Self::Error> {
        match primitive {
            PrimitiveJson::Circle {
                id,
                center,
                radius,
                fixed,
            } => Ok(Circle {
                id,
                center,
                radius,
                fixed,
            }),
            _ => Err("Expected Circle primitive".to_string()),
        }
    }
}

impl TryFrom<PrimitiveJson> for Line {
    type Error = String;

    fn try_from(primitive: PrimitiveJson) -> Result<Self, Self::Error> {
        match primitive {
            PrimitiveJson::Line { id, start, end } => Ok(Line { id, start, end }),
            _ => Err("Expected Line primitive".to_string()),
        }
    }
}

impl TryFrom<PrimitiveJson> for Arc {
    type Error = String;

    fn try_from(primitive: PrimitiveJson) -> Result<Self, Self::Error> {
        match primitive {
            PrimitiveJson::Arc {
                id,
                center,
                radius,
                start_angle,
                end_angle,
                fixed,
            } => Ok(Arc {
                id,
                center,
                radius,
                start_angle,
                end_angle,
                fixed,
            }),
            _ => Err("Expected Arc primitive".to_string()),
        }
    }
}

impl TryFrom<ConstraintJson> for ConstraintType {
    type Error = String;

    fn try_from(constraint: ConstraintJson) -> Result<Self, Self::Error> {
        match constraint {
            ConstraintJson::Vertical { point_a, point_b } => {
                Ok(ConstraintType::Vertical(point_a, point_b))
            }
            ConstraintJson::Horizontal { point_a, point_b } => {
                Ok(ConstraintType::Horizontal(point_a, point_b))
            }
            ConstraintJson::Parallel {
                point_a,
                point_b,
                point_c,
                point_d,
            } => Ok(ConstraintType::Parallel(point_a, point_b, point_c, point_d)),
            ConstraintJson::EqualX { point, x } => Ok(ConstraintType::EqualX(point, x)),
            ConstraintJson::EqualY { point, y } => Ok(ConstraintType::EqualY(point, y)),
            ConstraintJson::Coincident { point_a, point_b } => {
                Ok(ConstraintType::Coincident(point_a, point_b))
            }
            ConstraintJson::PointOnLine {
                point,
                point_line_a,
                point_line_b,
            } => Ok(ConstraintType::PointOnLine(point, point_line_a, point_line_b)),
            ConstraintJson::EqualRadius { circle1, circle2 } => {
                Ok(ConstraintType::EqualRadius(circle1, circle2))
            }
            ConstraintJson::FixedRadius { circle, radius } => {
                Ok(ConstraintType::FixedRadius(circle, radius))
            }
            ConstraintJson::PointOnCircle { point, circle } => {
                Ok(ConstraintType::PointOnCircle(point, circle))
            }
            ConstraintJson::Tangent { entity1, entity2 } => {
                Ok(ConstraintType::Tangent(entity1, entity2))
            }
        }
    }
}
