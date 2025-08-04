// DISCLAIMER:
// A bunch of this code is from Part3D, lots may not make sense, but this is just an example for the ACS anyway.

import { Line, ScreenSizer, type LineProps } from "@react-three/drei";
import { useThree } from "@react-three/fiber";
import { useGesture } from "@use-gesture/react";
import { useState } from "react";
import * as THREE from "three";
import { convert2DPointTo3D, convert3DPointTo2D } from "./utils";
import { Vector3 } from "three";

const mousePosition2D = new THREE.Vector2();
const mousePosition3D = new THREE.Vector3();
const dragPlaneNormal = new THREE.Vector3();
const dragPlane = new THREE.Plane();

export const Circle2D = ({
  center,
  radius,
  selected = false,
  onDrag,
  onClick,
}: {
  center: THREE.Vector2;
  radius: number;
  selected: boolean;
  onClick?: () => void;
  onDrag?: (newPosition: THREE.Vector2) => void;
}) => {
  const pointIn3D = new THREE.Vector3(center.x, center.y, 0);

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

        const endCircle2D = convert3DPointTo2D(mousePosition3D, {
          origin: new Vector3(0, 0, 0),
          xDir: new Vector3(1, 0, 0),
          yDir: new Vector3(0, 1, 0),
        });

        if (onDrag) {
          onDrag(new THREE.Vector2(endCircle2D.x, endCircle2D.y));
        }
      },
    },
    {
      drag: {
        filterTaps: true,
      },
    }
  );
  const ellipse = new THREE.EllipseCurve(
    center.x,
    center.y, // aX, aY
    radius,
    radius, // xRadius, yRadius
    0, // aStartAngle
    Math.PI * 2, // aEndAngle
    false,
    0 // aRotation
  );

  const points3D = ellipse.getPoints(70).map((c) =>
    convert2DPointTo3D(c, {
      origin: new Vector3(0, 0, 0),
      xDir: new Vector3(1, 0, 0),
      yDir: new Vector3(0, 1, 0),
    })
  );

  return (
    <>
      <Line
        points={points3D}
        visible={true}
        lineWidth={2}
        color={hovered ? "#278AFF" : selected ? "#ed145b" : "#000"}
        {...(bind() as Partial<LineProps>)}
      />
    </>
  );
};
