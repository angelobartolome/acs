import { Vector2, type Vector3 } from "three";

export function convert3DPointTo2D(
  point: Vector3,
  plane: {
    origin: Vector3;
    xDir: Vector3;
    yDir: Vector3;
  }
) {
  let x = point
    .clone()
    .projectOnVector(plane.xDir.normalize())
    .distanceTo(plane.origin.clone().projectOnVector(plane.xDir.normalize()));

  let y = point
    .clone()
    .projectOnVector(plane.yDir.normalize())
    .distanceTo(plane.origin.clone().projectOnVector(plane.yDir.normalize()));

  if (plane.xDir.dot(point.clone().sub(plane.origin)) < 0) {
    x *= -1;
  }

  if (plane.yDir.dot(point.clone().sub(plane.origin)) < 0) {
    y *= -1;
  }

  return new Vector2(x, y);
}
