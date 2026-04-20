import { Canvas } from "@react-three/fiber";
import { Grid, Html } from "@react-three/drei";
import { useEffect, useState } from "react";
import { Point2D } from "./components/Point2D";
import { Vector2 } from "three";
import { Line2D } from "./components/Line2D";
import init, { ConstraintSolver, Circle, Point } from "acs";
import { Circle2D } from "./components/Circle2D";

let solver: ConstraintSolver;

await init().then(() => {
  console.log("ACS initialized");
  solver = new ConstraintSolver();
});

type Primitive = Point | Circle;

export default function App() {
  const [primitives, setPrimitives] = useState<Primitive[]>([]);
  const [constraints, setConstraints] = useState<
    {
      type:
        | "horizontal"
        | "vertical"
        | "parallel"
        | "point_on_line"
        | "equal_radius";
      points: string[];
    }[]
  >([]);

  const [selectedPrimitiveIds, setSelectedPrimitiveIds] = useState<string[]>(
    []
  );

  useEffect(() => {
    const pA = new Point("0", 0, 0, true);
    const pB = new Point("1", 1, 1, false);
    const pC = new Point("3", 0.4, 1.3, false);
    const pD = new Point("4", 0.5, 0.5, false);

    // Circles

    const centerA = new Point("centerA", 0, 1, false);
    const centerB = new Point("centerB", 1.5, 1.5, false);

    const circle = new Circle("c", "centerA", 0.5, false);
    const circleB = new Circle("d", "centerB", 0.2, false);

    setPrimitives([pA, pB, pC, pD, centerA, centerB, circle, circleB]);
  }, [setPrimitives]);

  console.log(constraints);

  const solve = (pList: (Point | Circle)[], cList: any) => {
    solver.reset();
    pList.forEach((p) => {
      if (p instanceof Point) {
        solver.add_point(p);
      }
      if (p instanceof Circle) {
        solver.add_circle(p);
      }
    });

    cList.forEach((c: any) => {
      console.log(
        `Adding constraint: ${c.type} with points ${c.points.join(", ")}`
      );
      if (c.type === "horizontal") {
        const [pA, pB] = c.points;
        solver.add_horizontal_constraint(pA, pB);
      } else if (c.type === "vertical") {
        const [pA, pB] = c.points;
        solver.add_vertical_constraint(pA, pB);
      } else if (c.type === "parallel") {
        const [laS, laE, lbS, lbE] = c.points;
        solver.add_parallel_constraint(laS, laE, lbS, lbE);
      } else if (c.type === "point_on_line") {
        const [p, pA, pB] = c.points;
        solver.add_point_on_line_constraint(p, pA, pB);
      } else if (c.type === "equal_radius") {
        const [cA, cB] = c.points;
        solver.add_equal_radius_constraint(cA, cB);
      }
    });

    solver.solve();

    // Pull back the points to update their positions
    return pList.map((p) => {
      if (p instanceof Point) {
        const updatedPoint = solver.get_point(p.id);
        if (updatedPoint) {
          console.log(
            `Updated point: ${p.id} to (${updatedPoint.x}, ${updatedPoint.y})`
          );
          return new Point(p.id, updatedPoint.x, updatedPoint.y, false);
        }
      } else if (p instanceof Circle) {
        const updatedCircle = solver.get_circle(p.id);
        if (updatedCircle) {
          console.log(updatedCircle);

          return new Circle(
            p.id,
            updatedCircle.center,
            updatedCircle.radius,
            false
          );
        }
      }
      return p;
    });
  };

  const addConstraint = (
    type:
      | "horizontal"
      | "vertical"
      | "parallel"
      | "point_on_line"
      | "equal_radius"
  ) => {
    if (selectedPrimitiveIds.length < 2) {
      alert("Select two primitives to add a constraint.");
      return;
    }

    const selectedPrimitives = primitives.filter((p) =>
      selectedPrimitiveIds.includes(p.id)
    );

    if (
      type === "horizontal" ||
      type === "vertical" ||
      type === "equal_radius"
    ) {
      if (selectedPrimitives.length !== 2) {
        alert("Select exactly two points for horizontal/vertical constraints.");
        return;
      }
      const [pA, pB] = selectedPrimitives as Point[];
      setConstraints((prev) => [...prev, { type, points: [pA.id, pB.id] }]);
    } else if (type === "parallel") {
      if (selectedPrimitives.length !== 4) {
        alert("Select exactly four points for parallel constraints.");
        return;
      }
      const [pA, pB, pC, pD] = selectedPrimitives as Point[];
      setConstraints((prev) => [
        ...prev,
        {
          type,
          points: [pA.id, pB.id, pC.id, pD.id],
        },
      ]);
    } else if (type === "point_on_line") {
      if (selectedPrimitives.length !== 3) {
        alert("Select exactly three points for point on line constraints.");
        return;
      }
      const [p, pA, pB] = selectedPrimitives as Point[];
      setConstraints((prev) => [
        ...prev,
        { type, points: [p.id, pA.id, pB.id] },
      ]);
    }
  };

  const renderPrimitives = (primitives: Primitive[]) => {
    return primitives.map((primitive) => {
      if (primitive instanceof Point) {
        return (
          <Point2D
            position={new Vector2(primitive.x, primitive.y)}
            key={`p-${primitive.id}`}
            onClick={() => {
              setSelectedPrimitiveIds((prev) => {
                if (prev.includes(primitive.id)) {
                  return prev.filter((id) => id !== primitive.id);
                } else {
                  return [...prev, primitive.id];
                }
              });
            }}
            selected={selectedPrimitiveIds.includes(primitive.id)}
            onDrag={(newPosition) => {
              const updatedPrimitives = primitives.map((p) => {
                if (p.id === primitive.id) {
                  return new Point(p.id, newPosition.x, newPosition.y, false);
                }
                return p;
              });

              setPrimitives(solve(updatedPrimitives as any, constraints));
            }}
          />
        );
      } else if (primitive instanceof Circle) {
        console.log(primitives);
        let centerPoint = primitives.find(
          (p) => p.id === primitive.center
        ) as Point;

        console.log(
          `Rendering Circle: ${primitive.id} with center ${centerPoint.x}, ${centerPoint.y} and radius ${primitive.radius}`
        );
        return (
          <Circle2D
            selected={selectedPrimitiveIds.includes(primitive.id)}
            center={new Vector2(centerPoint.x, centerPoint.y)}
            radius={primitive.radius}
            key={`c-${primitive.id}`}
            onClick={() => {
              setSelectedPrimitiveIds((prev) => {
                if (prev.includes(primitive.id)) {
                  return prev.filter((id) => id !== primitive.id);
                } else {
                  return [...prev, primitive.id];
                }
              });
            }}
          />
        );
      }
      return null;
    });
  };

  return (
    <Canvas
      orthographic
      camera={{ position: [0, 0, 200], rotation: [0, 0, 0], zoom: 100 }}
      gl={{ antialias: true }}
      style={{ height: "100vh", width: "100vw" }}
    >
      <ambientLight intensity={Math.PI / 2} />

      <Grid
        cellColor={"#e0e0e0"}
        cellThickness={1}
        cellSize={0.1}
        sectionColor={"#7095fa"}
        sectionSize={1}
        sectionThickness={1}
        infiniteGrid
        rotation={[Math.PI / 2, 0, 0]}
      />
      {renderPrimitives(primitives)}

      <Html fullscreen>
        <div className="absolute top-0 left-0 ">
          <div className="p-4 flex flex-col gap-2 z-100 bg-black m-4 rounded-lg drop-shadow-lg">
            <button onClick={() => addConstraint("horizontal")}>
              Add Horizontal Constraint
            </button>
            <button onClick={() => addConstraint("vertical")}>
              Add Vertical Constraint
            </button>

            <button onClick={() => addConstraint("parallel")}>
              Add Parallel Constraint
            </button>

            <button onClick={() => addConstraint("point_on_line")}>
              Add Point on Line Constraint
            </button>
            <button onClick={() => addConstraint("equal_radius")}>
              Add Equal Radius Constraint
            </button>
            <button
              onClick={() => setPrimitives(solve(primitives, constraints))}
            >
              Solve Constraints
            </button>
          </div>
        </div>

        <div className="absolute bottom-0 left-0 p-4 text-white bg-gray-700">
          <p>
            Select two points to add a horizontal or vertical constraint, or
            select two lines to add a parallel constraint.
          </p>
          <p>Selected Primitives: {selectedPrimitiveIds.join(", ")}</p>
          <p>Primitives: {primitives.map((p) => p.id).join(", ")}</p>
          <p>
            Constraints:{" "}
            {constraints
              .map((c) => `${c.type}(${c.points.join(", ")})`)
              .join(", ")}
          </p>
        </div>
      </Html>
    </Canvas>
  );
}
