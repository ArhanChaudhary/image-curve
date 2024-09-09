import wasmInit, { runMain } from "./pkg/image_curve.js";
await wasmInit();
if (import.meta.env.DEV) {
  runMain(
    new Worker(new URL("worker.js", import.meta.url), {
      type: "module",
    })
  );
} else {
  runMain(
    new Worker(new URL("worker.js", import.meta.url), {
      type: "classic",
    })
  );
}
