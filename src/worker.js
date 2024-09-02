import wasmInit, { handleMessage } from "./pkg/image_gilbert.js";
self.onmessage = async (e) => {
  let wasmInitPromise = wasmInit(e.data.wasmModule, e.data.wasmMemory);
  self.onmessage = (e) =>
    wasmInitPromise.then(handleMessage.bind(null, e.data));
};
