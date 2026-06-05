<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { AlbumItem } from "../MediaItems";

  type Props = {
    item: AlbumItem;
  };

  let { item }: Props = $props();
  let styling = $derived(item.defaultStyling + " " + item.outlineGradient);
  let content = $derived(item.content);
</script>

<Button {styling}><a href="/pages/album/{item.uuid}">{content}</a></Button>
