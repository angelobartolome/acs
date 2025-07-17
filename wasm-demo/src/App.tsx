import { Canvas } from "@react-three/fiber";
import { Grid, Html } from "@react-three/drei";
import { useCallback, useEffect, useState } from "react";
import { Point2D } from "./components/Point2D";
import { Vector2 } from "three";
import { Line2D } from "./components/Line2D";
import init, { ConstraintSolver, Line, Point } from "acs";

await init().then(() => {
  console.log("ACS initialized");
});

type Primitive = Point | Line;

export default function App() {
  const [primitives, setPrimitives] = useState<Primitive[]>([]);

  useEffect(() => {
    const pA = new Point(0, 0, 0, true);
    const pB = new Point(1, 1, 1, false);
    const pL1 = new Line(2, pA.id, pB.id);

    setPrimitives([pA, pB, pL1]);
  }, [setPrimitives]);

  const solve = useCallback(
    (type: "vertical" | "horizontal") => {
      let solver = new ConstraintSolver();

      primitives.forEach((primitive) => {
        if (primitive instanceof Point) {
          solver.add_point(primitive);
        } else if (primitive instanceof Line) {
          solver.add_line(primitive);
        }
      });

      console.log(solver.print_state());

      if (type === "horizontal") {
        // Add horizontal constraints
        solver.add_horizontal_constraint(2);
      } else if (type === "vertical") {
        solver.add_vertical_constraint(2);
      }

      solver.solve();

      console.log(solver.print_state());

      // Pull back the points to update their positions
      const updatedPrimitives = primitives.map((primitive) => {
        if (primitive instanceof Point) {
          return solver.get_point(primitive.id);
        } else if (primitive instanceof Line) {
          const start = solver.get_point(primitive.start);
          const end = solver.get_point(primitive.end);
          return new Line(primitive.id, start!.id, end!.id);
        }

        return primitive;
      });

      setPrimitives(updatedPrimitives as Primitive[]);
    },
    [primitives]
  );

  const renderPrimitives = (primitives: Primitive[]) => {
    return primitives.map((primitive) => {
      if (primitive instanceof Point) {
        return (
          <Point2D
            position={new Vector2(primitive.x, primitive.y)}
            key={`p-${primitive.id}`}
          />
        );
      } else if (primitive instanceof Line) {
        return (
          <Line2D
            points={[
              new Vector2(
                primitives.find((p) => p.id === primitive.start)?.x || 0,
                primitives.find((p) => p.id === primitive.start)?.y || 0
              ),
              new Vector2(
                primitives.find((p) => p.id === primitive.end)?.x || 0,
                primitives.find((p) => p.id === primitive.end)?.y || 0
              ),
            ]}
            key={`l-${primitive.id}`}
          />
        );
      }
      return null;
    });
  };

  return (
    <Canvas
      orthographic
      camera={{ position: [0, 0, 200], rotation: [0, 0, 0], zoom: 200 }}
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
        <div
          style={{
            position: "absolute",
            top: "10px",
            left: "10px",
            color: "white",
            backgroundColor: "rgba(0, 0, 0, 0.5)",
            padding: "10px",
            borderRadius: "5px",
            display: "flex",
            gap: "10px",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <button onClick={() => solve("vertical")}>Solve Vertical</button>
          <button onClick={() => solve("horizontal")}>Solve Horizontal</button>
        </div>
      </Html>
    </Canvas>
  );
}
