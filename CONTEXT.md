# ACS

A 2-D geometric constraint solver: given a sketch of geometric entities and the constraints between them, it moves the entities until every constraint holds.

## Language

### Sketch

**Sketch**:
The complete set of entities and constraints submitted for one solve.
_Avoid_: Drawing, model, scene

**Constraint**:
A geometric relationship that must hold between entities or between an entity and a value, such as parallel, coincident or distance.
_Avoid_: Rule, relation

**Component**:
An independent group of constraints linked through the entities they share, solved separately from the rest of the sketch.
_Avoid_: Cluster, island, subgraph, subsystem

**Solved**:
Every constraint in scope (one constraint, a component, or the whole sketch) holds to within tolerance, whether it already did or the solver made it so.
_Avoid_: Satisfied, converged

**Degrees of freedom**:
The number of independent ways an entity can still move without breaking any constraint or changing a fixed parameter.
_Avoid_: DOF (in prose), freedom

**Fully constrained**:
An entity with zero degrees of freedom. A Line is fully constrained when both of its endpoints are.
_Avoid_: Locked, determined, rigid

**Guide**:
An input a constraint copies across, such as a mirror's axis or an array's center or direction. When the Guide moves, the copies follow. A free Guide moves like any other free geometry, by a drag of a copy or by other constraints on the copies (adding Horizontal to a patterned side turns the pattern about its center, moving the center as needed); pin it to keep it put.
_Avoid_: Anchor, reference (Reference Geometry means fixed input geometry), driver

**Conflicting**:
A constraint that can't hold together with the others in its Component: no positions satisfy them all. Reported per constraint so a caller can refuse an edit that introduces one.
_Avoid_: Over-constrained (that's a property of the sketch, not a constraint), inconsistent

**Redundant**:
A constraint already implied by the others in its Component: removing it changes neither the solution nor the degrees of freedom.
_Avoid_: Duplicate, dependent

### Geometry

**Entity**:
A geometric object in a sketch: a Point, Line, Circle, Arc, or Ellipse. Referenced by a string ID.
_Avoid_: Primitive (that's the JSON wire format), geometry, shape, object

**Line**:
A bounded segment between two endpoint Points; it has no position of its own. Constraints measure against the segment itself unless they are explicitly Extension constraints.
_Avoid_: Segment, edge, infinite line

**Extension**:
The infinite line through a Line's endpoints. Only constraints named for it (Extension constraints) measure against it; they exist so sketches authored under infinite-line semantics keep their meaning.
_Avoid_: Infinite line, supporting line

**Arc**:
A counter-clockwise span of a circle that references three Points (its center, start and end) and owns its radius and start/end angles. Its endpoints always lie on it, without any constraint saying so, which is what lets Lines share them. Constraints measure against the span, never the whole circle: an arc is an arc and a circle is a circle.
_Avoid_: Circular segment, partial circle

**Ellipse**:
A closed ellipse that references two Points, its center and one focus (which sets the direction of its major axis), and owns its minor radius. Its major radius and second focus follow from those. Its axis endpoints are not part of it: they are ordinary Points held at the ends of an axis by constraints.
_Avoid_: Oval, conic

**Fixed**:
A flag on an entity that stops the solver from changing that entity's *own* values (a Point's coordinates, a Circle's radius, an Arc's radius and start/end angles, an Ellipse's minor radius). It does not reach referenced entities: a fixed Circle whose center Point is free can still move. Reference Geometry (`isReference`) is always Fixed.
_Avoid_: Pinned, locked, anchored

**Parameter**:
A named value in a sketch that constraints share by ID (wire type `param`), such as a Linked Offset's distance. The solver holds it fixed. Not the per-entity numbers the solver moves (a Point's x, a Circle's radius); those are internal.
_Avoid_: Variable, named dimension

### Wire format

**Primitive**:
One JSON object in a solve request's flat array, describing either an Entity or a Constraint.
_Avoid_: Using it to mean Entity
