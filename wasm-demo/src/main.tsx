import "./style.css";

import * as THREE from "three";
import { createRoot } from "react-dom/client";
import App from "./App";

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

createRoot(document.getElementById("root")!).render(<App />);

if (import.meta.hot) {
  import.meta.hot.accept();
}
