import * as THREE from "three";
import { Canvas } from "@react-three/fiber";
import { Line2D } from "./components/Line2D";
import { Grid } from "@react-three/drei";

export default function App() {
  return (
    <Canvas
      orthographic
      camera={{ position: [0, 0, 200], rotation: [0, 0, 0], zoom: 200 }}
      gl={{ antialias: true }}
      style={{ height: "100vh", width: "100vw" }}
    >
      <ambientLight intensity={Math.PI / 2} />
      <Line2D points={[new THREE.Vector2(0, 0), new THREE.Vector2(1, 1)]} />

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
    </Canvas>
  );
}
