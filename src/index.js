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
let changeSpeedInput = document.getElementById("change-speed");
let changeStepInput = document.getElementById("change-step");

let worker = new Worker("worker.js", { type: "module" });

let { memory: wasmMemory } = await wasmInit();
handleMessage({
  action: "canvasInit",
  payload: {
    canvas,
  },
});
worker.postMessage({
  wasmModule: wasmInit.__wbindgen_wasm_module,
  wasmMemory,
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

function changeSpeed() {
  handleMessage({
    action: "changeSpeed",
    payload: {
      new_speed_percentage: changeSpeedInput.valueAsNumber,
    },
  });
}

function changeStep() {
  handleMessage({
    action: "changeStep",
    payload: {
      new_step_percentage: changeStepInput.valueAsNumber,
    },
  });
}

changeSpeed();
changeStep();
uploadInput.addEventListener("change", uploadedImage);
startInput.addEventListener("click", start);
stepInput.addEventListener("click", step);
stopInput.addEventListener("click", stop);
changeSpeedInput.addEventListener("input", changeSpeed);
changeStepInput.addEventListener("input", changeStep);
