import { defineConfig } from "drizzle-kit";

export default defineConfig({
  dialect: "sqlite",
  schema: "./src/db/schema.ts",
  out: "./src-tauri/migrations",
  dbCredentials: {
    url: "sqlite:dev.db",
  },
  verbose: false,
  strict: true,
  casing: "snake_case",
  migrations: { prefix: "supabase" },
});
