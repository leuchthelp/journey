/// take from https://github.com/drizzle-team/drizzle-orm/discussions/2532#discussioncomment-11729397


import { db } from "./database.ts";
// @ts-ignore will be created automatically
import migrations from "./migrations.json" with { type: 'json' };

async function ensureMigrationsTable() {
  await db.execute(`
    CREATE TABLE IF NOT EXISTS drizzle_migrations (
      hash TEXT PRIMARY KEY,
      created_at TIMESTAMP NOT NULL DEFAULT NOW()
    )
  `);
}

async function getMigratedHashes(): Promise<string[]> {
  const result = await db.execute(`
    SELECT hash FROM drizzle_migrations ORDER BY created_at ASC
  `);
  return result.rows.map((row) => row.hash as string);
}

async function recordMigration(hash: string) {
  await db.execute(
    `
    INSERT INTO drizzle_migrations (hash, created_at)
    VALUES ('${hash}', NOW())
    ON CONFLICT DO NOTHING
  `,
  );
}

export async function migrate_pglite() {
  console.log("🚀 Starting pglite migration...");

  // Ensure migrations table exists
  await ensureMigrationsTable();

  // Get already executed migrations
  const executedHashes = await getMigratedHashes();

  // Filter and execute pending migrations
  const pendingMigrations = migrations.filter(
    (migration: any) => !executedHashes.includes(migration.hash),
  );

  if (pendingMigrations.length === 0) {
    console.log("✨ No pending migrations found.");
    return;
  }

  console.log(`📦 Found ${pendingMigrations.length} pending migrations`);

  // Execute migrations in sequence
  for (const migration of pendingMigrations) {
    console.log(`⚡ Executing migration: ${migration.hash}`);
    try {
      // Execute each SQL statement in sequence
      for (const sql of migration.sql) {
        await db.execute(sql);
      }

      // Record successful migration
      await recordMigration(migration.hash);
      console.log(`✅ Successfully completed migration: ${migration.hash}`);
    } catch (error) {
      console.error(`❌ Failed to execute migration ${migration.hash}:`, error);
      throw error;
    }
  }

  console.log("🎉 All migrations completed successfully");
}
