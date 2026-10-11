<script lang="ts">
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<h1>
  {#if data.post}
    {data.post.uuid}
  {/if}
</h1>
