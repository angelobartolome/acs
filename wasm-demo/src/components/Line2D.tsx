import * as THREE from "three";
import { Line } from "@react-three/drei";
import { Point2D } from "./Point2D";

export const Line2D = ({
  points,
}: {
  points: [THREE.Vector2, THREE.Vector2];
}) => {
  const line2DWidth = 1;
  const line2DColor = "#000";

  return (
    <>
      <Line points={points} color={line2DColor} lineWidth={line2DWidth} />

      <group>
        <Point2D position={points[0]} />
        <Point2D position={points[1]} />
      </group>
    </>
  );
};
