<script lang="ts">
  import type { MediaItemDTO } from "#lib/bindings.ts";
  import Button from "./MediaItemButton.svelte";

  type Props = {
    item: MediaItemDTO;
  };

  let { item }: Props = $props();
</script>

<div>
  <a href="/pages/{item.type}/{item.uuid}">
    <Button {item}></Button>
  </a>
</div>
