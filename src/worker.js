import { initSync, runWorker } from "./pkg/image_gilbert.js";
self.onmessage = (e) => {
  initSync(...e.data);
  runWorker();
};
