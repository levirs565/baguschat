<script lang="ts">
  import type { DecryptedChat2 } from "$lib/api";
  import type { Message } from "$lib/stores/chatStore";
  import { currentUserId, sessionStore } from "$lib/stores/sessionStore";
  import { Button, Spinner } from "flowbite-svelte";
  import { DownloadSolid, FileSolid } from "flowbite-svelte-icons";
  import { onMount } from "svelte";

  let { message }: { message: DecryptedChat2 } = $props();

  let isMe = message.sender_id == $currentUserId;

  let isDownloadFile = $state(false);

  let imgSrc = $state("");

  onMount(() => {
    if (imgSrc) {
      URL.revokeObjectURL(imgSrc);
    }

    if (message.type != "File") return;
    if (message.file_type != "Image") return;
    const store = $sessionStore;
    const msg = message;
    (async () => {
      const buffer = await store!.downloadChatFile(msg.id, msg.key);
      const blob = new Blob([new Uint8Array(buffer)]);
      const blobUrl = URL.createObjectURL(blob);

      imgSrc = blobUrl;
    })();
  });

  async function downloadFile() {
    if (message.type == "Text") return;

    isDownloadFile = true;
    try {
      const buffer = await $sessionStore!.downloadChatFile(
        message.id,
        message.key
      );
      const blob = new Blob([new Uint8Array(buffer)]);
      const blobUrl = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = blobUrl;
      link.download = message.filename;
      link.click();

      URL.revokeObjectURL(blobUrl);
    } catch (e) {
      alert(JSON.stringify(e));
    }
    isDownloadFile = false;
  }
</script>

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
    {#if message.type == "Text"}
      <p class="text-sm">{message.message}</p>
    {/if}
    {#if message.type == "File"}
      {#if message.file_type == "File"}
        <div class="flex justify-center">
          <FileSolid class="w-12 h-12 mb-2" />
        </div>
        <p class="text-center text-sm">{message.size}, {message.filename}</p>
      {:else}
        <img src={imgSrc} class="max-w-64 max-h-64 mb-2" />
      {/if}
      <p class="text-center">
        {#if message.uploaed}
          <Button
            color="dark"
            size="sm"
            disabled={isDownloadFile}
            onclick={downloadFile}
          >
            {#if isDownloadFile}
              <Spinner class="me-3" size="4" />
            {:else}
              <DownloadSolid class="me-2 h-5 w-5" />
            {/if}
            Unduh
          </Button>
        {:else}
          Sedang menunggu upload
        {/if}
      </p>
    {/if}

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
