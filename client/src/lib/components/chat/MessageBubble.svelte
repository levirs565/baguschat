<script lang="ts">
  import type { DecryptedChat2 } from "$lib/api";
  import type { Message } from "$lib/stores/chatStore";
  import { currentUserId } from "$lib/stores/sessionStore";

  let { message }: { message: DecryptedChat2 } = $props();

  let isMe = message.sender_id == $currentUserId;
</script>

{#if message.type == "Text"}
  <div class="flex" class:justify-end={isMe}>
    <div
      class="relative max-w-xs lg:max-w-md px-3 py-2 rounded-lg shadow"
      class:bg-blue-600={isMe}
      class:text-white={isMe}
      class:rounded-tr-none={isMe}
      class:bg-white={!isMe}
      class:dark:bg-gray-700={!isMe}
      class:rounded-tl-none={!isMe}
    >
      <p class="text-sm">{message.message}</p>

      <p
        class="text-xs mt-1 text-right"
        class:text-gray-200={isMe}
        class:text-gray-400={!isMe}
      >
        {message.created_at}
      </p>

      {#if isMe}
        <div
          class="flex w-0 h-0 border-solid top-0 right-[-10px] border-t-[10px] border-l-[10px] border-r-transparent border-b-transparent border-l-transparent border-t-blue-600"
        ></div>
      {:else}
        <div
          class="flex w-0 h-0 border-solid top-0 left-[-10px] border-t-[10px] border-r-[10px] border-l-transparent border-b-transparent border-r-transparent border-t-white dark:border-t-gray-700"
        ></div>
      {/if}
    </div>
  </div>
{/if}
