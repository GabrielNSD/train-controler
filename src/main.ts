import { invoke } from "@tauri-apps/api/tauri";
import { renderCanvas, TrainPosition } from "./rendering";

let canvasEl: HTMLCanvasElement | null;
let speedAdujuster1: HTMLInputElement | null;
let speedAdujuster2: HTMLInputElement | null;
let speedAdujuster3: HTMLInputElement | null;
let speedAdujuster4: HTMLInputElement | null;

async function render() {
  if (canvasEl) {
    const trains: TrainPosition[] = (
      (await invoke("get_trains")) as number[][]
    ).map((train: number[]) => ({ x: train[0], y: train[1] }));
    renderCanvas(canvasEl, ...trains);
  }

  setTimeout(render, 1000);
}

async function setTrainSpeed(trainId: number, speed: number) {
  await invoke("set_train_speed", { trainId, speed });
}

window.addEventListener("DOMContentLoaded", () => {
  canvasEl = document.querySelector("#trainCanvas");

  speedAdujuster1 = document.querySelector("#speedAdujuster1");
  speedAdujuster2 = document.querySelector("#speedAdujuster2");
  speedAdujuster3 = document.querySelector("#speedAdujuster3");
  speedAdujuster4 = document.querySelector("#speedAdujuster4");

  speedAdujuster1?.addEventListener("input", () => {
    setTrainSpeed(0, parseInt(speedAdujuster1?.value || "0") * 5);
  });

  speedAdujuster2?.addEventListener("input", () => {
    setTrainSpeed(1, parseInt(speedAdujuster2?.value || "0") * 5);
  });

  speedAdujuster3?.addEventListener("input", () => {
    setTrainSpeed(2, parseInt(speedAdujuster3?.value || "0") * 5);
  });

  speedAdujuster4?.addEventListener("input", () => {
    setTrainSpeed(3, parseInt(speedAdujuster4?.value || "0") * 5);
  });

  render();
});
