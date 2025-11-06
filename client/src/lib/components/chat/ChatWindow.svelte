<script lang="ts">
  import MessageInput from "./MessageInput.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import { activeMessages, activeContactId } from "$lib/stores/chatStore";
  import { Avatar, Heading } from "flowbite-svelte";
  import { createUserDataQueryOptions } from "$lib/queries/user";
  import { createQuery } from "@tanstack/svelte-query";

  let feedContainer: HTMLDivElement;

  let options = $derived(createUserDataQueryOptions($activeContactId))
  let userData = createQuery(() => $options)
</script>

{#if $activeContactId}
  <div
    class="flex items-center p-4 border-b dark:border-gray-700 bg-white dark:bg-gray-800"
  >
    <Avatar src={""} class="mr-3" />
    <Heading tag="h5">{userData.data?.username}</Heading>
  </div>

  <div
    bind:this={feedContainer}
    class="flex-1 p-4 overflow-y-auto space-y-4 bg-gray-50 dark:bg-gray-900"
  >
    {#each $activeMessages as message (message.id)}
      <MessageBubble {message} />
    {/each}
  </div>
{:else}
  <div class="p-4 border-b dark:border-gray-700 bg-white dark:bg-gray-800">
    <p class="font-semibold dark:text-white">Pilih Obrolan</p>
  </div>
  <div
    class="flex-1 p-4 overflow-y-auto flex items-center justify-center bg-gray-50 dark:bg-gray-900"
  >
    <p class="text-center text-gray-500">
      Silakan pilih kontak di sebelah kiri untuk memulai percakapan.
    </p>
  </div>
{/if}

<MessageInput />
