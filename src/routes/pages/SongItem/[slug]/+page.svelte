<script lang="ts">
  import { error } from "@sveltejs/kit";
  import type { PageProps } from "./$types";

  // Seems to always get wrapped into an array
  let { data }: PageProps = $props();

  let item = $derived(data.post);

  let url = $derived(item.providers.at(0)?.url);
  let test = $derived(
    `${url}/Audio/${item.original.at(0)?.uuid}/stream?static=true`,
  );

  $inspect(test);
</script>

<h1>
  {item.content ?? error(404, "Not found")}
</h1>
