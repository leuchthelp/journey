<script lang="ts">
  import ProviderAccordion from "#lib/components/Settings/Provider/ProviderAccordion.svelte";
  import { Effect } from "effect";
  import { getSupportedAuthSchema } from "#lib/effects/provider.ts";
  import { authSchemaComponents } from "../Auth";
  import type { VariantProxy } from "#lib/VariantManager.svelte.ts";
  import type { ProviderKey } from "#lib/bindings.ts";

  type Props = {
    proxy: VariantProxy;
  };
  let { proxy }: Props = $props();
</script>

<ProviderAccordion title={proxy.name}>
  <button onclick={() => proxy.addKey()}>Add new {proxy.name}</button>
  {#each proxy.keys as key, i}
    {#await Effect.runPromise(getSupportedAuthSchema(proxy.name)) then supportedSchema}
      {#each supportedSchema as schema}
        {#if authSchemaComponents.has(schema)}
          {@const SvelteComponent = authSchemaComponents.get(schema)}
          <SvelteComponent
            {key}
            variant={proxy.name}
            onKeyChange={(v: ProviderKey) => proxy.setKey(i, v)}
          ></SvelteComponent>
        {/if}
      {/each}
    {/await}
  {/each}
</ProviderAccordion>
