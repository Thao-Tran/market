<script lang="ts">
  import Header from '$lib/components/Header.svelte';
  import '../app.css';
</script>

<Header />

<main>
  <slot />
</main>

<style>
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 3rem;
    width: 100%;
    max-width: var(--max-width);
    margin: 0 auto;
    box-sizing: border-box;
  }
</style>
