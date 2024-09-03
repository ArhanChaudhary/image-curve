import wasmInit, { run } from "./pkg/image_gilbert.js";
await wasmInit();
run();
// let canvas = document.querySelector("canvas");
// let ctx = canvas.getContext("2d");
// let uploadInput = document.getElementById("upload");
// let startInput = document.getElementById("start");
// let stepInput = document.getElementById("step");
// let stopInput = document.getElementById("stop");
// let changeSpeedInput = document.getElementById("change-speed");
// let changeStepInput = document.getElementById("change-step");

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

// startInput.addEventListener("click", start);
// stepInput.addEventListener("click", step);
// stopInput.addEventListener("click", stop);
// changeSpeedInput.addEventListener("input", changeSpeed);
// changeStepInput.addEventListener("input", changeStep);
