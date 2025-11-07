<script lang="ts">
  import { Button, Modal, P } from "flowbite-svelte";
  import Uppy, { type UppyFile } from "@uppy/core";
  import { onDestroy, onMount } from "svelte";
  import { FileSolid } from "flowbite-svelte-icons";
  import { sessionStore } from "$lib/stores/sessionStore";
  import { activeContactId } from "$lib/stores/chatStore";
  import type { CryptoService } from "$lib/api";
  let { uppy }: { uppy: Uppy } = $props();

  let isOpen = $state(false);
  let fileName = $state("");
  let fileSize = $state("");
  let lastFileId = $state<string>();
  let isUploading = $state<boolean>();

  function onFileAdded(file: UppyFile<any, any>) {
    isOpen = true;
    fileName = file.name ?? "";
    fileSize = `${file.size}`;
    lastFileId = file.id;
  }

  function onClose() {
    if (!lastFileId) return;
    uppy.removeFile(lastFileId);
  }

  function onSubmit(event: { action: string }) {
    if (
      (event.action != "send" && event.action) ||
      !lastFileId ||
      !$activeContactId ||
      !$sessionStore
    ) {
      if (lastFileId) uppy.removeFile(lastFileId);
      lastFileId = "";
      return true;
    }

    isUploading = true;
    const fileId = lastFileId;

    (async () => {
      uppy.setFileMeta(fileId, {
        ...(await $sessionStore.cryptoService.prepareFileKey($activeContactId)),
        fileSize: uppy.getFile(fileId).size,
      } as any);
      await uppy.upload();
    })();

    lastFileId = "";

    return false;
  }

  function onFileUploaded() {
    isOpen = false;
    isUploading = false;
  } 

  function onFailUpload() {
    isUploading = false;
  }

  onMount(() => {
    uppy.on("file-added", onFileAdded);
    uppy.on("upload-success", onFileUploaded as any);
    uppy.on("upload-error", onFailUpload)
  });
  onDestroy(() => {
    uppy.off("file-added", onFileAdded);
    uppy.off("upload-success", onFileUploaded as any);
    uppy.off("upload-error", onFailUpload)
  });
</script>

<Modal
  form
  title="Kirim File"
  bind:open={isOpen}
  onclose={onClose}
  onaction={onSubmit}
  outsideclose={false}
>
  <div class="flex justify-center"><FileSolid class=" w-16 h-16" /></div>
  <P class="text-center">{fileSize}, {fileName}</P>
  {#snippet footer()}
    <Button disabled={isUploading} type="submit" value="send">Kirim</Button>
    <Button disabled={isUploading} type="submit" value="cancel">Batal</Button>
  {/snippet}
</Modal>
