<script lang="ts">
  import { Avatar, Heading } from "flowbite-svelte";
  import { selectContact, activeContactId } from "$lib/stores/chatStore";
  import type { DecryptedChatPartner } from "$lib/api";
  import {
    createUserDataQueryOptions,
    createCurrentUserQueryOptions,
  } from "$lib/queries/user";
  import { createQuery } from "@tanstack/svelte-query";

  let { contact }: {contact: DecryptedChatPartner} = $props();

  const userDataOptions = createUserDataQueryOptions(contact.id);
  const userData = createQuery(() => $userDataOptions);

  let isActive = $derived($activeContactId == contact.id);
</script>

<button
  type="button"
  on:click={() => selectContact(contact.id)}
  class="flex items-center w-full px-4 py-2 text-left font-medium border-x-0 border-t-0 border-b dark:border-gray-700 cursor-pointer"
  class:bg-blue-700={isActive}
  class:text-white={isActive}
  class:dark:bg-gray-600={isActive}
  class:bg-white={!isActive}
  class:text-gray-900={!isActive}
  class:dark:bg-gray-800={!isActive}
  class:dark:text-white={!isActive}
  class:hover:bg-gray-100={!isActive}
  class:dark:hover:bg-gray-600={!isActive}
>
  <div class="flex items-center space-x-4">
    <Avatar src={""} />
    <div class="flex-1 min-w-0">
      <Heading tag="h5" class="truncate"
        >{userData.data?.username}</Heading
      >
      <p class="text-sm text-gray-500 dark:text-gray-400 truncate">
        {contact.last_chat.type == "Text"
          ? contact.last_chat.message
          : contact.last_chat.file_type == "File"
            ? "Berkas"
            : "Gambar"}
      </p>
    </div>
  </div>
</button>
