<script lang="ts">
  import type { PageProps } from "./$types";
  import { itemCache } from "$lib/components/MediaItems/ItemCache";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import { toSvelteComponent } from "$lib/snippets/ToSvelteComponent.svelte";

  let { data }: PageProps = $props();
</script>

<main class="mt-5 h-full">
  {#each data.post as item}
    {#if itemCache.set(item.hash, item)}
      {@render toSvelteComponent(item)}
    {/if}
  {/each}
</main>
<Playbar />
