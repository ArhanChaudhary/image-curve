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
    // let pixelData = getImageBuffer(img);
    ctx.drawImage(img, 0, 0, 512, 512);
    let pixelData = ctx.getImageData(0, 0, 512, 512).data;
    renderer.postMessage({
      action: "canvasInit",
      pixelData,
      width: 512,
      height: 512,
    });
  };
  img.src = src;
}
let canvas = document.querySelector("canvas");
let ctx = canvas.getContext("2d");

renderer.addEventListener("message", (e) => {
  debugger;
  let imgData = new ImageData(e.data.imageBuffer, 512, 512);
  ctx.putImageData(imgData, 0, 0);
});

input.addEventListener("change", reset);

function step() {
  renderer.postMessage({ action: "step" });
}

window.step = step;
