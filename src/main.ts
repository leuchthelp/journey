import { invoke } from "@tauri-apps/api/core";
import { SongItem } from "./MediaItems";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  let main_space = document.getElementById("main-space") as HTMLDivElement;
  let newSongItem = new SongItem();

  let styling = newSongItem.defaultStyling();

  const newElement = document.createElement("div");

  for (var style of styling) {
    newElement.classList.add(style);
  }
  newElement.classList.add("ring");
  newElement.classList.add(newSongItem.outlineGradient());

  const spawn = [{ transform: "scale(0)" }, { transform: "scale(1)" }];
  const spawnTime = {
    duration: 1000,
    iterations: 1,
  };
  newElement.animate(spawn, spawnTime);
  main_space.appendChild(newElement);
});
