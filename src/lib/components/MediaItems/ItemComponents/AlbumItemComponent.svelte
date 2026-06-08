<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { AlbumItem } from "../MediaItems";

  type Props = {
    item: AlbumItem;
  };

  let { item }: Props = $props();
  let styling = $derived(item.defaultStyling + " " + item.outlineGradient);
  let content = $derived(item.content.filter((item) => item.type === "Name").at(0));
</script>

<Button {styling}><a href="/pages/album/{item.uuid}">{content?.description}</a></Button>
