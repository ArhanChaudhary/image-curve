import { initSync, runWorker } from "./pkg/image_curve.js";
self.onmessage = (e) => {
  initSync(...e.data);
  runWorker();
};
