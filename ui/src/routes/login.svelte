<script lang="ts">
  import { getApiClient } from '$lib/utils/api-client';
  import { isUserLoggedIn } from '$lib/utils/auth';
  import { onMount } from 'svelte';

  enum Mode {
    Login,
    Register
  }

  const LOGIN_FAIL = 'Failed to login:';
  const REGISTER_FAIL = 'Failed to register:';
  const client = getApiClient();
  let credentials = { email: '', password: '' };
  let mode = Mode.Login;
  let errorMsg = '';

  $: registerAction = {
    class: mode === Mode.Register ? 'primary' : 'secondary',
    type: mode === Mode.Register ? 'submit' : 'button',
    onClick: (e: Event) => {
      if (mode !== Mode.Register) {
        mode = Mode.Register;
        e.preventDefault();
      }
    }
  };
  $: loginAction = {
    class: mode === Mode.Login ? 'primary' : 'secondary',
    type: mode === Mode.Login ? 'submit' : 'button',
    onClick: (e: Event) => {
      if (mode !== Mode.Login) {
        mode = Mode.Login;
        e.preventDefault();
      }
    }
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

      if (res.status === 204) {
        window.location.href = '/';
        return;
      } else if (res.status === 404) {
        errorMsg = 'user with that email does not exist. Register instead?';
      } else if (res.status < 500) {
        errorMsg = 'invalid email/password combination.';
      } else {
        errorMsg = 'error occurred. Try again in a few minutes.';
      }

      errorMsg = `${LOGIN_FAIL} ${errorMsg}`;
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

      if (res.status === 201) {
        await onLogin();
        return;
      } else if (res.status === 409) {
        errorMsg = 'user with that email already exists. Login instead?';
      } else {
        errorMsg = 'error occurred. Try again in a few minutes.';
      }

      errorMsg = `${REGISTER_FAIL} ${errorMsg}`;
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
  {#if mode === Mode.Login}
    <title>market | login</title>
  {:else if mode === Mode.Register}
    <title>market | register</title>
  {/if}
</svelte:head>

<div class="login">
  {#if mode === Mode.Login}
    <h1>login</h1>
  {:else if mode === Mode.Register}
    <h1>register</h1>
  {/if}
  <form on:submit|preventDefault={onSubmit}>
    <label for="email">email</label>
    <input
      id="email"
      type="email"
      aria-label="Email address"
      placeholder="someone@domain.com"
      bind:value={credentials.email}
      required
    />
    <label for="password">password</label>
    <input id="password" type="password" aria-label="Password" bind:value={credentials.password} required />
    <div class="actions">
      <button
        class={registerAction.class}
        type={registerAction.type}
        on:click={registerAction.onClick}
      >
        register
      </button>
      <button class={loginAction.class} type={loginAction.type} on:click={loginAction.onClick}>
        login
      </button>
    </div>
    <div class="error-msg">
      {errorMsg}
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

  form > input {
    margin-top: 0.5rem;
    margin-bottom: 2rem;
  }

  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    column-gap: 1rem;
  }

  .error-msg {
    color: rgb(194, 3, 3);
    height: 6rem;
    margin-top: 2rem;
  }
</style>
