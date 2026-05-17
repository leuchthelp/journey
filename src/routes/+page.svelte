<script lang="ts">
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import { componentOptions } from "$lib/components/MediaItems/ItemComponents/ItemComponents";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<main class="mt-5 h-full">
  {#each data.post as item}
    {#if componentOptions.has(item.type)}
      {@const SvelteComponent = componentOptions.get(item.type)}
      <SvelteComponent {item}></SvelteComponent>
    {/if}
  {/each}
</main>
<Playbar />
