// DISCLAIMER:
// A bunch of this code is from Part3D, lots may not make sense, but this is just an example for the ACS anyway.

import { ScreenSizer } from "@react-three/drei";
import { useThree } from "@react-three/fiber";
import { useGesture } from "@use-gesture/react";
import { useState } from "react";
import * as THREE from "three";
import { convert3DPointTo2D } from "./utils";
import { Vector3 } from "three";

const mousePosition2D = new THREE.Vector2();
const mousePosition3D = new THREE.Vector3();
const dragPlaneNormal = new THREE.Vector3();
const dragPlane = new THREE.Plane();

export const Point2D = ({
  position,
  selected = false,
  onDrag,
  onClick,
}: {
  position: THREE.Vector2;
  selected: boolean;
  onClick?: () => void;
  onDrag?: (newPosition: THREE.Vector2) => void;
}) => {
  const pointIn3D = new THREE.Vector3(position.x, position.y, 0);

  const [hovered, setHovered] = useState(false);
  const { camera, size, raycaster } = useThree();

  const bind = useGesture(
    {
      onPointerOver: () => setHovered(true),
      onPointerOut: () => setHovered(false),

      onDragStart: ({ event, intentional, cancel, tap }) => {
        if (!intentional || tap) {
          cancel();
          return;
        }

        const { point } = event as any;

        const mousePosition3D = new THREE.Vector3();
        mousePosition3D.copy(point);

        const dragPlaneNormal = new THREE.Vector3(0, 0, 0);
        const dragPlane = new THREE.Plane();

        camera.getWorldDirection(dragPlaneNormal).negate();

        dragPlane.setFromNormalAndCoplanarPoint(
          dragPlaneNormal,
          mousePosition3D
        );
      },
      onDrag: ({ xy: [dragX, dragY], intentional, tap }) => {
        if (tap) onClick?.();
        if (!intentional) return;

        const normalizedMouseX = ((dragX - size.left) / size.width) * 2 - 1;
        const normalizedMouseY = -((dragY - size.top) / size.height) * 2 + 1;

        mousePosition2D.set(normalizedMouseX, normalizedMouseY);
        raycaster.setFromCamera(mousePosition2D, camera);

        camera.getWorldDirection(dragPlaneNormal).negate();

        dragPlane.setFromNormalAndCoplanarPoint(
          dragPlaneNormal,
          mousePosition3D
        );
        raycaster.ray.intersectPlane(dragPlane, mousePosition3D);

        const endPoint2D = convert3DPointTo2D(mousePosition3D, {
          origin: new Vector3(0, 0, 0),
          xDir: new Vector3(1, 0, 0),
          yDir: new Vector3(0, 1, 0),
        });

        if (onDrag) {
          onDrag(new THREE.Vector2(endPoint2D.x, endPoint2D.y));
        }
      },
    },
    {
      drag: {
        filterTaps: true,
      },
    }
  );

  return (
    <>
      <group position={pointIn3D}>
        <ScreenSizer scale={1} {...bind()}>
          <mesh visible={false}>
            <sphereGeometry args={[12]} />
            <meshBasicMaterial color={`#9e9e9e`} />
          </mesh>
        </ScreenSizer>

        {/* Point */}
        <ScreenSizer scale={1}>
          <mesh>
            <sphereGeometry args={hovered || selected ? [5] : [3]} />
            <meshBasicMaterial
              color={hovered ? "#278AFF" : selected ? "#ed145b" : "#000"}
            />
          </mesh>
        </ScreenSizer>
      </group>
    </>
  );
};
