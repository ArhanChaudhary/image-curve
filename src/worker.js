import wasmInit, { handleWorkerMessage } from "./pkg/image_gilbert.js";
self.onmessage = async (e) => {
  let wasmInitPromise = wasmInit(...e.data);
  self.onmessage = (e) =>
    wasmInitPromise.then(handleWorkerMessage.bind(null, e.data));
};
