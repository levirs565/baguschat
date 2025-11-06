<script>
  import { Card, Button, Label, Input, Heading } from "flowbite-svelte";
  import { sessionStore } from "$lib/stores/sessionStore";
  import { createCurrentUserQueryOptions } from "$lib/queries/user";
  import { createQuery } from "@tanstack/svelte-query";
  import { goto } from "$app/navigation";

  let username = "";
  let password = "";

  async function handleLogin() {
    await sessionStore.login(username, password);
  }

  
  const userQueryOptions = createCurrentUserQueryOptions();
  const user = createQuery(() => $userQueryOptions);

  $effect(() => {
    if (user.data) {
      goto("/chat")
    }
  })
</script>

<svelte:head>
  <title>Login</title>
</svelte:head>

<Card class="w-full max-w-md p-6">
  <form class="flex flex-col space-y-6" on:submit|preventDefault={handleLogin}>
    <Heading tag="h3" class="text-xl font-medium text-gray-900 dark:text-white">
      Login ke Akun Anda
    </Heading>

    <div>
      <Label for="username" class="mb-2">Username Anda</Label>
      <Input
        type="text"
        id="username"
        bind:value={username}
        placeholder="username"
        required
      />
    </div>

    <div>
      <Label for="password" class="mb-2">Password Anda</Label>
      <Input
        type="password"
        id="password"
        bind:value={password}
        placeholder="••••••••"
        required
      />
    </div>

    <Button type="submit" class="w-full">Login</Button>

    <div class="text-sm font-medium text-gray-500 dark:text-gray-300">
      Belum punya akun?
      <a
        href="/register"
        class="text-blue-700 hover:underline dark:text-blue-500"
      >
        Buat akun
      </a>
    </div>
  </form>
</Card>
