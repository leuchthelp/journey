<script module lang="ts">
  import ProviderAccordionItem from "#lib/components/Settings/Provider/ProviderAccordionItem.svelte";
  import { providerAuthOptions } from "#lib/components/Settings/Auth/index.ts";
  import { type ProviderVariant } from "#lib/bindings.ts";
  export { toAuthComponent };
</script>

{#snippet toAuthComponent(type: ProviderVariant, key?: [string, string])}
  <ProviderAccordionItem>
    {#if providerAuthOptions.has(type)}
      {@const SvelteComponent = providerAuthOptions.get(type)}
      <SvelteComponent {key}></SvelteComponent>
    {/if}
  </ProviderAccordionItem>
{/snippet}
