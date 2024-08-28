import wasmInit, { greet } from "./pkg/image_gilbert.js";

self.postMessage("loaded");
self.onmessage = async (e) => {
  await wasmInit(e.data.wasmModule, e.data.wasmMemory);
  self.onmessage = () => {
    greet();
  }
  self.postMessage("ready");
};
