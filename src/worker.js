import wasmInit, { handleMessage } from "./pkg/image_gilbert.js";

self.postMessage("loaded");
self.onmessage = async (e) => {
  await wasmInit(e.data.wasmModule, e.data.wasmMemory);
  self.onmessage = (e) => handleMessage(e.data);
  self.postMessage("ready");
};
