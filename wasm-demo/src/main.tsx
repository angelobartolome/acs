import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import init, { acsSolveSketch } from "acs";

import App from "./App";
import { setSolverService, sketchStore } from "./core/sketch/store";
import { AcsSolverService } from "./core/solver/SolverService";
import { EXAMPLES } from "./examples";
import "./style.css";

async function main(): Promise<void> {
  await init();
  setSolverService(new AcsSolverService(acsSolveSketch));

  // start with a preset so the canvas is not empty
  sketchStore.getState().loadSketch(structuredClone(EXAMPLES[0].sketch));

  const container = document.getElementById("root");
  if (container === null) throw new Error("#root not found");
  createRoot(container).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

void main();
