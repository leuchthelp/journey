<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { ArtistItem } from "../MediaItems";

  type Props = {
    item: ArtistItem;
  };

  let { item }: Props = $props();
  let styling = $derived(item.defaultStyling + " " + item.outlineGradient);
  let content = $derived(
    item.content.filter((item) => item.type === "Name").at(0),
  );
</script>

<Button {styling}
  ><a href="/pages/artist/{item.uuid}">{content?.description}</a></Button
>
