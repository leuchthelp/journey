<script lang="ts">
  import Button from "./MediaItemButton.svelte";
  import type { MediaItem } from "$lib/db/relations";

  type Props = {
    item: MediaItem;
  };

  let { item }: Props = $props();
</script>

<div>
  <a href="/pages/{item.type}/{item.uuid}">
    <Button {item}></Button>
  </a>
</div>
