<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { SongItem } from "../MediaItems";

  type Props = {
    item: SongItem;
  };

  let { item }: Props = $props();
  $inspect(item)

  let url = $derived(item.providers.at(0)?.url)
  let test = $derived(`${url}/Audio/${item.original.at(0)?.uuid}/stream?static=true`)

  $inspect(test)
</script>

<div>
  <a href="/pages/song/{item.uuid}">
    <Button {item}></Button>
  </a>
</div>

