<script module lang="ts">
  import { componentOptions } from "$lib/components/MediaItems/ItemComponents/ItemComponents";
  import { type MediaItems } from "../../db/schema";

  export { toSvelteComponent };
</script>

{#snippet toSvelteComponent(item: MediaItems)}
  {#if componentOptions.has(item.type)}
    {@const SvelteComponent = componentOptions.get(item.type)}
    <SvelteComponent {item}></SvelteComponent>
  {/if}
{/snippet}
