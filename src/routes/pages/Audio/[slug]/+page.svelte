<script lang="ts">
  import { error } from "@sveltejs/kit";

  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();

  let item = $derived(data.itemReq);
</script>

<h1>
  {item ? JSON.stringify(item) : error(404, "Not found")}
</h1>
