<script module lang="ts">
  import { providerAuthOptions } from "$lib/components/Settings/Provider/variants";
  import type { ProviderItem } from "$lib/db/schema/schema";
  export { toAuthComponent };
</script>

{#snippet toAuthComponent(item: ProviderItem)}
  {#if providerAuthOptions.has(item.type)}
    {@const SvelteComponent = providerAuthOptions.get(item.type)}
    <SvelteComponent id={item.id} url={item.url} type={item.type}
    ></SvelteComponent>
  {/if}
{/snippet}
