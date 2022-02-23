<script lang="ts">
  import { page } from '$app/stores';
  import { getApiClient } from '$lib/utils/api-client';
  import { isUserLoggedIn } from '$lib/utils/auth';
  import { onMount, afterUpdate } from 'svelte';

  let showLogin: boolean;

  onMount(() => {
    showLogin = !isUserLoggedIn();
  });

  afterUpdate(() => {
    showLogin = !isUserLoggedIn();
  });

  async function onLogout() {
    const client = getApiClient();
    const req = client.buildRequestDelete({ type: 'tokens' });
    try {
      const res = await fetch(req.url, { ...req, credentials: 'include' });
      if (res.status === 204) {
        window.location.href = '/';
        return;
      }

      const body = await res.json();
      console.error(res.statusText, body);
    } catch (e) {
      console.error(e);
    }
  }
</script>

<header>
  <div class="content">
    <a href="/" class="logo">market</a>

    <nav>
      <ul>
        {#if showLogin}
          <li class:active={$page.url.pathname === '/login'}>
            <a sveltekit:prefetch href="/login">login</a>
          </li>
        {:else}
          <li>
            <button on:click={onLogout}>logout</button>
          </li>
        {/if}
      </ul>
    </nav>
  </div>
</header>

<style>
  header {
    display: flex;
    justify-content: center;
    width: 100%;
    padding: 2rem 0 0;
    height: var(--header-height);
  }

  .content {
    display: flex;
    flex: 1;
    justify-content: space-between;
    max-width: var(--max-width);
  }

  .logo {
    padding: 1rem;
    border: 0.5rem solid var(--accent-color);
    font-weight: 900;
    font-size: 3rem;
    color: var(--accent-color);
  }

  .logo:hover {
    box-shadow: 0rem 0.125rem 0.5rem var(--accent-color);
  }

  .logo:active, .logo:focus {
    outline: 0;
    box-shadow: 0rem 0.125rem 0.5rem var(--accent-color);
  }

  header a:hover,
  header button:hover {
    text-decoration: none;
    cursor: pointer;
  }

  nav {
    display: flex;
    justify-content: center;
  }

  ul {
    position: relative;
    padding: 0;
    margin: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    list-style: none;
  }

  li {
    position: relative;
    height: 100%;
    border-bottom: 0.25rem solid white;
  }

  li.active,
  li:hover {
    border-bottom: 0.25rem solid var(--accent-color);
  }

  nav a,
  nav button {
    display: flex;
    height: 100%;
    align-items: center;
    padding: 0 2rem;
    font-weight: 700;
    font-size: medium;
    letter-spacing: 0.1em;
    text-decoration: none;
    transition: color 0.2s linear;
    background: none;
    border: none;
    color: var(--accent-color);
  }
</style>
