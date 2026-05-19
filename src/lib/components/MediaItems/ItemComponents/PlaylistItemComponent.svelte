<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { PlaylistItem } from "../MediaItems";

  type Props = {
    item: PlaylistItem;
  };

  let { item }: Props = $props();
  let styling = $derived(item.defaultStyling + " " + item.outlineGradient);
  let content = $derived(item.content);
</script>

<Button {styling}><a href="/pages/playlist/{item.hash}">{content}</a></Button>
