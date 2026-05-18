<script lang="ts">
  import type { PageData, PageProps, Snapshot } from "./$types";
  import { itemCache, homeCache } from "$lib/components/MediaItems/ItemCache";
  import { toSvelteComponent } from "$lib/snippets/ToSvelteComponent.svelte";

  let { data }: PageProps = $props();

  export const snapshot: Snapshot<PageData> = {
    capture: () => data,
    restore: (value) => (data = value),
  };
</script>

{#each data.post as item}
  {#if itemCache.set(item.hash, item) && homeCache.set(item.hash, item)}
    {@render toSvelteComponent(item)}
  {/if}
{/each}
