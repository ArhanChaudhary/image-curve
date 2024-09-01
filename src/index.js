import wasmInit, { handleMessage, renderPixelData } from "./pkg/image_gilbert.js";

let canvas = document.querySelector("canvas");
let ctx = canvas.getContext("2d");

let worker = new Worker("worker.js", { type: "module" });
let loadWorker = new Promise((resolve) => {
  worker.addEventListener(
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
  handleMessage({
    action: "canvasInit",
    payload: {
      canvas,
    },
  });
});

await Promise.all([loadWorker, loadWasmMemory]);
worker.postMessage({
  wasmModule: wasmInit.__wbindgen_wasm_module,
  wasmMemory,
});

await new Promise((resolve) => {
  worker.addEventListener(
    "message",
    (e) => {
      if (e.data === "ready") {
        resolve();
      }
    },
    { once: true }
  );
});

let input = document.querySelector("input");

async function toBase64(file) {
  return new Promise((resolve) => {
    let reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result);
  });
}

async function reset() {
  let src = await toBase64(input.files[0]);
  let img = new Image();
  img.onload = function () {
    // canvas.width = 512;
    // canvas.height = 512;
    ctx.drawImage(img, 0, 0, 512, 512);
    handleMessage({
      action: "loadImage",
    });
  };
  img.src = src;
}

input.addEventListener("change", reset);

function step() {
  handleMessage({ action: "step" });
}

window.step = step;
window.renderPixelData = renderPixelData;