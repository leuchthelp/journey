<script>
  let { children } = $props();
  import "../app.css";

  import Navbar from "$lib/components/Navbar/Navbar.svelte";
  import NavbarButton from "$lib/components/Navbar/NavbarButton.svelte";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import PlaybarButton from "$lib/components/Playbar/PlaybarButton.svelte";
</script>

<Navbar>
  <NavbarButton>hello</NavbarButton>
</Navbar>
<main class="mt-5 h-full">
  {@render children()}
</main>
<Playbar>
  <PlaybarButton>1</PlaybarButton>
  <PlaybarButton>2</PlaybarButton>
  <PlaybarButton>3</PlaybarButton>
</Playbar>
