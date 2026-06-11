<script lang="ts">
  import { error } from "@sveltejs/kit";
  import type { PageProps } from "./$types";

  // Seems to always get wrapped into an array
  let { data }: PageProps = $props();

  let item = $derived(data.post);
</script>

<enhanced:img
  class="h-48 w-48 object-contain rounded-xl bg-amber-200"
  src={item.images[0]}
  alt="Loading"
/>
<h1>
  {item.content ?? error(404, "Not found")}
</h1>
