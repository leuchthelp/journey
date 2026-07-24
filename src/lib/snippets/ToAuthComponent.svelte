<script module lang="ts">
  import ProviderAccordionItem from "#lib/components/Settings/Provider/ProviderAccordionItem.svelte";
  import { providerAuthOptions } from "#lib/components/Settings/Auth/index.ts";
  export { toAuthComponent };
</script>

{#snippet toAuthComponent(type: string, userId?: string)}
  <ProviderAccordionItem>
    {#if providerAuthOptions.has(type)}
      {@const SvelteComponent = providerAuthOptions.get(type)}
      <SvelteComponent {userId}></SvelteComponent>
    {/if}
  </ProviderAccordionItem>
{/snippet}
