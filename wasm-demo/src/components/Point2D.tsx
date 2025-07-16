import { ScreenSizer } from "@react-three/drei";
import { useState } from "react";
import * as THREE from "three";

export const Point2D = ({ position }: { position: THREE.Vector2 }) => {
  const pointIn3D = new THREE.Vector3(position.x, position.y, 0);

  const [hovered, setHovered] = useState(false);

  return (
    <>
      <group position={pointIn3D}>
        <ScreenSizer scale={1}>
          <mesh
            visible={false}
            onPointerEnter={() => setHovered(true)}
            onPointerLeave={() => setHovered(false)}
          >
            <sphereGeometry args={[12]} />
            <meshBasicMaterial color={`#9e9e9e`} />
          </mesh>
        </ScreenSizer>

        {/* Point */}
        <ScreenSizer scale={1}>
          <mesh>
            <sphereGeometry args={hovered ? [5] : [3]} />
            <meshBasicMaterial color={hovered ? "#278AFF" : "#000"} />
          </mesh>
        </ScreenSizer>
      </group>
    </>
  );
};
