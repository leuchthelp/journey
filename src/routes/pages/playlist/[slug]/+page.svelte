<script lang="ts">
  import type { PageData, PageProps, Snapshot } from "./$types";

  let { data }: PageProps = $props();

  export const snapshot: Snapshot<PageData> = {
    capture: () => data,
    restore: (value) => (data = value),
  };
</script>

<h1>
  {#if data.post}
    {data.post.uuid}
  {/if}
</h1>
