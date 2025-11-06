<script lang="ts">
  import {
    Modal,
    Button,
    Label,
    Input,
    Avatar,
    Heading,
  } from "flowbite-svelte";
  import { isNewChatModalOpen } from "$lib/stores/uiStore.js";
  import { sessionStore } from "$lib/stores/sessionStore";
  import type { GetUserResponse } from "$lib/api";
  import { createUserListQueryOptions } from "$lib/queries/user";
  import { createQuery } from "@tanstack/svelte-query";
  import { selectContact } from "$lib/stores/chatStore";

  let username: string = $state("");

  let options = $derived(createUserListQueryOptions(username));

  let query = createQuery(() => $options);
</script>

<Modal title="Mulai Obrolan Baru" bind:open={$isNewChatModalOpen} autoclose>
  <form id="search-form" class="space-y-4">
    <div>
      <Label for="username" class="mb-2">Cari Berdasarkan Username</Label>
      <Input
        type="text"
        id="username"
        bind:value={username}
        placeholder="ketik username..."
        required
      />
    </div>

    {#if query.data}
      {#each query.data as item}
        <button
          type="button"
          on:click={() => selectContact(item.id)}
          class="flex items-center w-full px-4 py-2 text-left font-medium border-x-0 border-t-0 border-b dark:border-gray-700 cursor-pointer"
          class:bg-white={true}
          class:text-gray-900={true}
          class:dark:bg-gray-800={true}
          class:dark:text-white={true}
          class:hover:bg-gray-100={true}
          class:dark:hover:bg-gray-600={true}
        >
          <div class="flex items-center space-x-4">
            <Avatar src={""} />
            <div class="flex-1 min-w-0">
              <Heading tag="h5" class="truncate">{item.username}</Heading>
            </div>
          </div>
        </button>
      {/each}
    {/if}
  </form>
</Modal>
