import { defineConfig } from "drizzle-kit";

export default defineConfig({
  dialect: "sqlite",
  schema: "./src/lib/db/schema",
  out: "./src-tauri/migrations",
  dbCredentials: {
    url: "sqlite:dev.db",
  },
  verbose: false,
  strict: true,
  casing: "snake_case",
  migrations: { prefix: "supabase" },
});
