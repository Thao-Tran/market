<script lang="ts">
  import { getApiClient } from '$lib/utils/api-client';
  import { isUserLoggedIn } from '$lib/utils/auth';
  import { onMount } from 'svelte';

  enum Mode {
    Login,
    Register
  }

  const client = getApiClient();
  let credentials = { email: '', password: '' };
  let mode = Mode.Login;

  $: registerAction = {
    class: mode === Mode.Register ? 'active' : '',
    type: mode === Mode.Register ? 'submit' : 'button',
    onClick: () => (mode = Mode.Register)
  };
  $: loginAction = {
    class: mode === Mode.Login ? 'active' : '',
    type: mode === Mode.Login ? 'submit' : 'button',
    onClick: () => (mode = Mode.Login)
  };

  onMount(() => {
    if (isUserLoggedIn()) {
      window.location.href = '/';
    }
  });

  async function onLogin() {
    const request = client.buildRequestCreate({ resource: credentials, type: 'tokens' });
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

  async function onRegister() {
    const request = client.buildRequestCreate({ resource: credentials, type: 'users' });
    // Need this because of jsonapi-rust issues
    request.data.data.id = '';

    try {
      const res = await fetch(request.url, {
        ...request,
        body: JSON.stringify(request.data)
      });
      if (res.status !== 201) {
        const error = await res.json();
        console.error(res.statusText, error);
      } else {
        await onLogin();
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function onSubmit() {
    if (mode === Mode.Login) {
      await onLogin();
    } else {
      await onRegister();
    }
  }
</script>

<svelte:head>
  <title>market | login</title>
</svelte:head>

<div class="login">
  <form on:submit|preventDefault={onSubmit}>
    <input
      type="email"
      aria-label="Email address"
      placeholder="someone@domain.com"
      bind:value={credentials.email}
      required
    />
    <input type="password" aria-label="Password" bind:value={credentials.password} required />
    <div class="actions">
      <button
        class={registerAction.class}
        type={registerAction.type}
        on:click={registerAction.onClick}
      >
        Register
      </button>
      <button class={loginAction.class} type={loginAction.type} on:click={loginAction.onClick}>
        Login
      </button>
    </div>
  </form>
</div>

<style>
  .login {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    justify-content: center;
    align-items: center;
  }

  form {
    display: flex;
    flex-direction: column;
    max-width: 48rem;
    width: 100%;
  }

  form > * {
    margin-bottom: 1rem;
  }

  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    column-gap: 1rem;
  }

  button {
    flex: 1;
    border: 0.125rem solid var(--accent-color);
    background-color: white;
    border-radius: 0.5rem;
  }

  button:hover {
    cursor: pointer;
  }

  button.active {
    background-color: var(--accent-color);
    color: white;
  }
</style>
