<script lang="ts">
  import type { PageProps } from "./$types";
  import { itemCache, homeCache } from "$lib/components/MediaItems/ItemCache";
  import { toSvelteComponent } from "$lib/snippets/ToSvelteComponent.svelte";

  let { data }: PageProps = $props();
</script>

{#each data.post as item}
  {#if itemCache.set(item.hash, item) && homeCache.set(item.hash, item)}
    {@render toSvelteComponent(item)}
  {/if}
{/each}
