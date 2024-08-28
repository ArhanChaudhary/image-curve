import wasmInit from "./pkg/image_gilbert.js";

let renderer = new Worker("worker.js", { type: "module" });
let loadRenderer = new Promise((resolve) => {
  renderer.addEventListener(
    "message",
    (e) => {
      if (e.data === "loaded") {
        resolve();
      }
    },
    { once: true }
  );
});
let wasmMemory;
let loadWasmMemory = wasmInit().then(({ memory }) => {
  wasmMemory = memory;
});

await Promise.all([loadRenderer, loadWasmMemory]);
renderer.postMessage({
  wasmModule: wasmInit.__wbindgen_wasm_module,
  wasmMemory,
});
await new Promise((resolve) => {
  renderer.addEventListener(
    "message",
    (e) => {
      if (e.data === "ready") {
        resolve();
      }
    },
    { once: true }
  );
});
renderer.postMessage(undefined);

