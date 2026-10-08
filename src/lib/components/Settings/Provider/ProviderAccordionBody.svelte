<script lang="ts">
  import { Effect } from "effect";
  import * as ProviderAccordion from "#lib/components/Settings/Provider/index.ts";
  import { getSupportedAuthSchema } from "#lib/effects/provider.ts";
  import { authSchemaComponents } from "../Auth/index.ts";
  import type { ProviderTypeProxy } from "#lib/ProviderTypeManager.svelte.ts";
  import type { ProviderKey, ProviderType } from "#lib/bindings.ts";

  type Props = {
    type: ProviderType;
    proxy: ProviderTypeProxy;
    incProgress: () => void;
    decProgress: () => void;
  };
  let { type, proxy, incProgress, decProgress }: Props = $props();
</script>

<ProviderAccordion.Root title={type}>
  <button onclick={() => proxy.addKey()}>Add new {type}</button>
  {#each proxy.keys as key, i (i)}
    {#await Effect.runPromise(getSupportedAuthSchema(type)) then supportedSchema}
      {#each supportedSchema as schema}
        {#if authSchemaComponents.has(schema)}
          {@const AuthComponent = authSchemaComponents.get(schema)}
          <AuthComponent
            {type}
            {key}
            onKeyChange={(v: ProviderKey) => proxy.setKey(i, v)}
            {incProgress}
            {decProgress}
          ></AuthComponent>
        {/if}
      {/each}
    {/await}
  {/each}
</ProviderAccordion.Root>
