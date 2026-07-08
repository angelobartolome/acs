import { useStore } from "zustand";

import { sketchStore, type SketchState } from "../core/sketch/store";

/** React hook over the vanilla sketch store (kept out of core/). */
export function useSketchStore<T>(selector: (state: SketchState) => T): T {
  return useStore(sketchStore, selector);
}
