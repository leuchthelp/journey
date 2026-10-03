<script lang="ts">
  import type { MediaItemDTO } from "#lib/bindings.ts";
  import { Effect } from "effect";
  import Button from "./MediaItemButton.svelte";
  import { stream } from "#lib/effects/provider.ts";

  type Props = {
    item: MediaItemDTO;
  };

  let { item }: Props = $props();

  $inspect(item);
</script>

<button
  onclick={async () =>
    await Effect.runPromise(
      stream(item.providers?.at(0)?.key!, item.sources?.at(0)?.sourceId!),
    )}>Play</button
>
<div>
  <a href="/pages/{item.type}/{item.uuid}">
    <Button {item}></Button>
  </a>
</div>
