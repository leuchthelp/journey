<script>
  let { children } = $props();
  import "../app.css";

  import Navbar from "$lib/components/Navbar/Navbar.svelte";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
</script>

<Navbar />
<main class="mt-5 h-full">
  {@render children()}
</main>
<Playbar />
