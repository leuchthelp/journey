import { db } from "../../db/database";
import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import * as schema from "../../db/schema";
import { eq } from "drizzle-orm";

export const load: PageLoad = async ({ params }) => {
  let res = await db
    .select()
    .from(schema.mediaItems)
    .where(eq(schema.mediaItems.hash, params.slug))
    .limit(1);

  if (res.length !== 0)
    return {
      post: res,
    };

  error(404, "Not found");
};
