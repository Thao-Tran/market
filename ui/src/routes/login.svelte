<script lang="ts">
	import { Token } from '$lib/models';
	import { getApiClient } from '$lib/utils/api-client';
	import { isUserLoggedIn } from '$lib/utils/auth';
	import { onMount } from 'svelte';

	let tokenReq = new Token();
	const client = getApiClient();

	onMount(() => {
		if (isUserLoggedIn()) {
			window.location.href = '/';
		}
	});

	async function onLogin() {
		const request = client.buildRequestCreate({ resource: tokenReq });
		// Need this because of jsonapi-rust issues
		request.data.data.id = '';

		try {
			const res = await fetch(request.url, {
				...request,
				body: JSON.stringify(request.data),
				credentials: 'include'
			});
			if (res.status !== 204) {
				const error = await res.json();
				console.error(res.statusText, error);
			} else {
				window.location.href = '/';
			}
		} catch (e) {
			console.error(e);
		}
	}
</script>

<svelte:head>
	<title>market | login</title>
</svelte:head>

<div class="login">
	<form on:submit|preventDefault={onLogin}>
		<input
			type="email"
			aria-label="Email address"
			placeholder="someone@domain.com"
			bind:value={tokenReq.email}
			required
		/>
		<input type="password" aria-label="Password" bind:value={tokenReq.password} required />
		<button type="submit">Login</button>
	</form>
</div>

<style>
</style>
