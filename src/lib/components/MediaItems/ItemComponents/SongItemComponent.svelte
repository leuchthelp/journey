<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { SongItem } from "../MediaItems";

  type Props = {
    item: SongItem;
  };

  let { item }: Props = $props();
  let styling = $derived(item.defaultStyling + " " + item.outlineGradient);
  let content = $derived(item.content);
</script>

<Button {styling}><a href="/pages/song/{item.hash}">{content}</a></Button>
