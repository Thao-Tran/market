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
	<div class="corner">
		<a href="/" class="logo">market</a>
	</div>

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
</header>

<style>
	header {
		margin: 0 1.5em;
		display: flex;
		justify-content: space-between;
		max-width: 1024px;
		width: 100%;
		padding: 1em;
	}

	header a,
	header button {
		font-family: var(--font-mono);
	}

	header a:hover,
	header button:hover {
		text-decoration: none;
		cursor: pointer;
	}

	.logo {
		font-weight: bolder;
		font-size: x-large;
	}

	.corner a {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
	}

	nav {
		display: flex;
		justify-content: center;
		--background: rgba(255, 255, 255, 0.7);
	}

	ul {
		position: relative;
		padding: 0;
		margin: 0;
		height: 3em;
		display: flex;
		justify-content: center;
		align-items: center;
		list-style: none;
	}

	li {
		position: relative;
		height: 100%;
		border-bottom: 2px solid var(--background);
	}

	li.active, li:hover {
		border-bottom: 2px solid var(--accent-color);
	}

	nav a,
	nav button {
		display: flex;
		height: 100%;
		align-items: center;
		padding: 0 1em;
		color: var(--heading-color);
		font-weight: bold;
		font-size: medium;
		letter-spacing: 0.1em;
		text-decoration: none;
		transition: color 0.2s linear;
		background: none;
		border: none;
	}
</style>
