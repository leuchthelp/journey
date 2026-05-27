import { betterAuth } from "better-auth";
import { drizzleAdapter } from "@better-auth/drizzle-adapter";
import { db } from "./db/database";

export const auth = betterAuth({
  database: drizzleAdapter(db, { 
    provider: "sqlite", // or "pg" or "mysql"
  }), 
  //... the rest of your config
});