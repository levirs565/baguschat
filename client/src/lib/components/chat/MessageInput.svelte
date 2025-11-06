<script lang="ts">
  import { Textarea, Button } from "flowbite-svelte";
  import PaperClipOutline from "flowbite-svelte-icons/PaperClipOutline.svelte";
  import CameraPhotoOutline from "flowbite-svelte-icons/CameraPhotoOutline.svelte";
  import PaperPlaneOutline from "flowbite-svelte-icons/PaperPlaneOutline.svelte";
  import { activeContactId, chatWsStore } from "$lib/stores/chatStore";

  let messageText = "";

  function handleSend() {
    if (!messageText.trim()) return;

	let id = $activeContactId
	let chatWs = $chatWsStore
	if (!id || !chatWs) return;
    chatWs.sendText(id, messageText);
	messageText = ""
  }
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  }
</script>

<form class="p-4 bg-white dark:bg-gray-800 border-t dark:border-gray-700" on:submit={handleSend}>
  <div class="flex items-center space-x-2">
    <Button color="alternative" class="mr-2 p-2">
      <PaperClipOutline class="w-6 h-6" />
    </Button>

    <Button color="alternative" class="mr-2 p-2">
      <CameraPhotoOutline class="w-6 h-6" />
    </Button>

    <div class="relative flex-1" on:keydown={handleKeydown}>
      <Textarea
        bind:value={messageText}
        placeholder="Ketik pesan Anda..."
        rows={1}
        class="w-full resize-none pr-14"
      />

      <Button type="submit" class="absolute right-2.5 bottom-2 p-2" >
        <PaperPlaneOutline class="w-5 h-5 rotate-z-90" />
      </Button>
    </div>
  </div>
</form>
