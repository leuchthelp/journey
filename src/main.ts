import { invoke } from "@tauri-apps/api/core";
import { SongItem } from "./MediaItems";
import Database from "@tauri-apps/plugin-sql";

const db = await Database.load("sqlite:test.db");

let res;
let query =
  "CREATE TABLE if NOT EXISTS sources (" +
  "id INTEGER PRIMARY KEY," +
  "name TEXT NOT NULL," +
  "details TEXT" +
  ");";
res = await db.execute(query);

res = await db.execute("DELETE FROM sources");

res = await db.execute("INSERT into sources (name, details)\n VALUES ($1, $2)", [
  "Laptop",
  '{"brand": "Dell", "price": 1200, "features": ["i7", "16GB RAM", "512GB SSD"]}',
]);

res = await db.select(
  "SELECT json_extract(details, '$.brand') AS brand\n" + "FROM sources;",
);
console.log(res)

res = await db.select(
  "SELECT json_extract(details, '$.features') AS features\n" + "FROM sources;",
) as Array<Object>;
console.log(res.at(0))

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
