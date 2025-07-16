import * as THREE from "three";
import { Line2 } from "three/addons/lines/Line2.js";
import { LineMaterial } from "three/addons/lines/LineMaterial.js";
import { LineGeometry } from "three/addons/lines/LineGeometry.js";
import type { Mesh } from "three";

export class Point2D {
  x: number;
  y: number;

  constructor(x: number, y: number) {
    this.x = x;
    this.y = y;
  }
}

// Z is always 0 for this project
export class Line extends THREE.Object3D {
  mesh: Group;

  start: Point2D;
  end: Point2D;

  private lineMesh: Mesh;
  private startMesh: Mesh;
  private endMesh: Mesh;

  private geometry: LineGeometry;

  constructor(p1: Point2D, p2: Point2D) {
    super();

    this.start = p1;
    this.end = p2;

    this.geometry = new LineGeometry();

    let material = new LineMaterial({
      color: "#ed145b",
      linewidth: 2,
      alphaToCoverage: false,
    });

    this.lineMesh = new Line2(this.geometry, material);
    this.mesh = new THREE.Group();
    this.mesh.add(this.lineMesh);

    // Circle for start point
    this.createStartPoint();
    // Circle for end point
    this.createEndPoint();

    this.updateGeometry();
  }

  private createStartPoint() {
    const geometry = new THREE.SphereGeometry(0.4, 64, 64);
    const sphereMat = new THREE.MeshBasicMaterial({ color: 0xffff00 });
    const sphere = new THREE.Mesh(geometry, sphereMat);
    sphere.position.set(this.start.x, this.start.y, 0);
    sphere.name = "startPoint"; // Optional: name for identification
    this.startMesh = sphere;
    this.mesh.add(sphere);
  }

  private createEndPoint() {
    const geometry = new THREE.SphereGeometry(0.4, 64, 64);
    const sphereMat = new THREE.MeshBasicMaterial({ color: 0xffff00 });
    const sphere = new THREE.Mesh(geometry, sphereMat);
    sphere.position.set(this.end.x, this.end.y, 0);
    sphere.name = "endPoint"; // Optional: name for identification
    this.endMesh = sphere;
    this.mesh.add(sphere);
  }

  private updateGeometry() {
    const positions = [
      this.start.x,
      this.start.y,
      0,
      this.end.x,
      this.end.y,
      0,
    ];
    this.startMesh.position.set(this.start.x, this.start.y, 0);
    this.endMesh.position.set(this.end.x, this.end.y, 0);
    this.geometry.setPositions(positions);
  }

  // Call this method whenever start or end points change
  update() {
    this.updateGeometry();
  }
}
