import wasmInit, {
  handleMessage,
  renderPixelData,
} from "./pkg/image_gilbert.js";

let canvas = document.querySelector("canvas");
let ctx = canvas.getContext("2d");
let uploadInput = document.getElementById("upload");
let startInput = document.getElementById("start");
let stepInput = document.getElementById("step");
let stopInput = document.getElementById("stop");

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

async function toBase64(file) {
  return new Promise((resolve) => {
    let reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result);
  });
}

async function uploadedImage() {
  let src = await toBase64(uploadInput.files[0]);
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

let rafId;

function start() {
  worker.postMessage({
    action: "start",
  });
  rafId = requestAnimationFrame(renderPixelDataLoop);
}

function renderPixelDataLoop() {
  renderPixelData();
  rafId = requestAnimationFrame(renderPixelDataLoop);
}

function step() {
  handleMessage({
    action: "step",
  });
  renderPixelData();
}

function stop() {
  handleMessage({
    action: "stop",
  });
  cancelAnimationFrame(rafId);
}

uploadInput.addEventListener("change", uploadedImage);
startInput.addEventListener("click", start);
stepInput.addEventListener("click", step);
stopInput.addEventListener("click", stop);
