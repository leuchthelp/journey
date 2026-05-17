<script lang="ts">
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import type { PageProps } from "./$types";
  import { toSvelteComponent } from "$lib/snippets/ToSvelteComponent.svelte";

  let { data }: PageProps = $props();
</script>

<main class="mt-5 h-full">
  {#each data.post as item}
    {@render toSvelteComponent(item)}
  {/each}
</main>
<Playbar />
