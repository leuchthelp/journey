<script lang="ts">
  import { Effect } from "effect";
  import * as ProviderAccordion from "#lib/components/Settings/Provider/index.ts";
  import { getSupportedAuthSchema } from "#lib/effects/provider.ts";
  import { authSchemaComponents } from "../Auth/index.ts";
  import type { VariantProxy } from "#lib/VariantManager.svelte.ts";
  import type { ProviderKey } from "#lib/bindings.ts";

  type Props = {
    proxy: VariantProxy;
  };
  let { proxy }: Props = $props();
</script>

<ProviderAccordion.Root title={proxy.name}>
  <button onclick={() => proxy.addKey()}>Add new {proxy.name}</button>
  {#each proxy.keys as key, i (key)}
    {#await Effect.runPromise(getSupportedAuthSchema(proxy.name)) then supportedSchema}
      {#each supportedSchema as schema}
        {#if authSchemaComponents.has(schema)}
          {@const AuthComponent = authSchemaComponents.get(schema)}
          <AuthComponent
            {key}
            variant={proxy.name}
            onKeyChange={(v: ProviderKey) => proxy.setKey(i, v)}
          ></AuthComponent>
        {/if}
      {/each}
    {/await}
  {/each}
</ProviderAccordion.Root>
