// compile-migrations.ts

import { readMigrationFiles } from "drizzle-orm/migrator";

const migrations = readMigrationFiles({ migrationsFolder: "./drizzle/" });

const encoder = new TextEncoder();
const data = encoder.encode(JSON.stringify(migrations));
console.log(new URL("../migrations.json", import.meta.url).pathname);

// @ts-ignore: script will be run in deno environment
await Deno.writeFile(
  new URL("./migrations.json", import.meta.url).pathname,
  data,
);

console.log("Migrations compiled!");
