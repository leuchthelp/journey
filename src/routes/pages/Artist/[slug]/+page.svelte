<script lang="ts">
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<h1>
  {#await data.itemReq then item}
    {#if item}
      {item.uuid}
    {/if}
  {/await}
</h1>
