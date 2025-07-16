import { Canvas } from "@react-three/fiber";
import { Grid, Html } from "@react-three/drei";
import { useCallback, useState } from "react";
import { PointPrimitive, Primitive } from "./types";

export default function App() {
  const [primitives, setPrimitives] = useState<Primitive[]>([
    new PointPrimitive(0, 0),
    new PointPrimitive(1, 1),
  ]);

  const solve = useCallback(() => {
    console.log("Solve function called");
    // TODO
  }, []);

  return (
    <Canvas
      orthographic
      camera={{ position: [0, 0, 200], rotation: [0, 0, 0], zoom: 200 }}
      gl={{ antialias: true }}
      style={{ height: "100vh", width: "100vw" }}
    >
      <ambientLight intensity={Math.PI / 2} />
      {}

      <Grid
        cellColor={"#a0a0a0"}
        cellThickness={1}
        cellSize={0.1}
        sectionColor={"#7095faff"}
        sectionSize={1}
        sectionThickness={1}
        infiniteGrid
        rotation={[Math.PI / 2, 0, 0]}
      />

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
          }}
        >
          <button onClick={() => solve()}>Solve</button>
        </div>
      </Html>
    </Canvas>
  );
}
