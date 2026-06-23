<script lang="ts">
  import { error } from "@sveltejs/kit";
  import type { PageProps } from "./$types";
  import { getMediaSource } from "$lib/signals/mediaSource.svelte";

  // Seems to always get wrapped into an array
  let { data }: PageProps = $props();

  let item = $derived(data.post);

  let src = getMediaSource();

  let url = $derived(item.providers.at(0)?.url);
  let test = $derived(
    `${url}/Audio/${item.original.at(0)?.uuid}/stream?static=true`,
  );

  $effect(() => {
    src.url = test;
  });

  $inspect(test);
</script>

<h1>
  {item.content ?? error(404, "Not found")}
</h1>
