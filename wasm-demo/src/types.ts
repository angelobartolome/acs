export class Primitive {}

export class PointPrimitive extends Primitive {
  x: number;
  y: number;

  constructor(x: number, y: number) {
    super();
    this.x = x;
    this.y = y;
  }
}

export class LinePrimitive extends Primitive {
  start: PointPrimitive;
  end: PointPrimitive;

  constructor(start: PointPrimitive, end: PointPrimitive) {
    super();
    this.start = start;
    this.end = end;
  }
}
