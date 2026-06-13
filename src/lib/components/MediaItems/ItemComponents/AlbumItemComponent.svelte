<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { AlbumItem } from "../MediaItems";

  type Props = {
    item: AlbumItem;
  };

  let { item }: Props = $props();

  $inspect(item)
</script>

<div>
  <a href="/pages/album/{item.uuid}">
    <Button {item}></Button>
  </a>
</div>
