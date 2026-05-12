import { invoke } from "@tauri-apps/api/core";
import { SongItem } from "./MediaItems";
import { MediaCache } from "./db/MediaCache.ts";

const cache = new MediaCache();

let test = [
  "Laptop",
  '{"brand": "Dell", "price": 1200, "features": ["i7", "16GB RAM", "512GB SSD"]}',
];
let res = await cache.addEntries(test);
let res1 = await cache.getEntries("features");
console.log(res1);

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
