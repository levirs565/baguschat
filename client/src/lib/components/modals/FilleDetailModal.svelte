<script lang="ts">
  import { Button, Checkbox, Input, Label, Modal, P } from "flowbite-svelte";
  import Uppy, { type UppyFile } from "@uppy/core";
  import { onDestroy, onMount } from "svelte";
  import { FileSolid } from "flowbite-svelte-icons";
  import { sessionStore } from "$lib/stores/sessionStore";
  import { activeContactId } from "$lib/stores/chatStore";
  import type { CryptoService } from "$lib/api";
  let { uppy }: { uppy: Uppy } = $props();

  let isOpen = $state(false);
  let mode = $state("file");
  let fileName = $state("");
  let fileSize = $state("");
  let lastFileId = $state<string>();
  let isUploading = $state<boolean>();
  let imageUrl = $state("");
  let addSecret = $state(false);
  let secretMessage = $state("");

  function onFileAdded(file: UppyFile<any, any>) {
    mode = file.meta.mode;
    isOpen = true;
    fileName = file.name ?? "";
    fileSize = `${file.size}`;
    lastFileId = file.id;
    if (mode == "Image") {
      if (file.data instanceof Blob || file.data instanceof File)
        imageUrl = URL.createObjectURL(file.data);
    }
  }

  function cleanImageUrl() {
    if (imageUrl) {
      URL.revokeObjectURL(imageUrl);
      imageUrl = "";
    }
  }

  function onClose() {
    addSecret = false;
    secretMessage = "";
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
      cleanImageUrl();
      return true;
    }

    isUploading = true;
    const fileId = lastFileId;

    if (addSecret && mode == "Image")
      uppy.setFileMeta(fileId, {
        secretMessage: secretMessage,
      });

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
    cleanImageUrl();
    isOpen = false;
    isUploading = false;
  }

  function onFailUpload() {
    isUploading = false;
  }

  onMount(() => {
    uppy.on("file-added", onFileAdded);
    uppy.on("upload-success", onFileUploaded as any);
    uppy.on("upload-error", onFailUpload);
  });
  onDestroy(() => {
    uppy.off("file-added", onFileAdded);
    uppy.off("upload-success", onFileUploaded as any);
    uppy.off("upload-error", onFailUpload);
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
  <div class="flex justify-center">
    {#if mode == "File"}
      <FileSolid class=" w-16 h-16" />
    {:else}
      <img src={imageUrl} class="max-w-64 max-h-64" />
    {/if}
  </div>

  <P class="text-center">{fileSize}, {fileName}</P>

  {#if mode == "Image"}
    <Checkbox bind:checked={addSecret}>Tambahkan Pesan Rahasia</Checkbox>
    {#if addSecret}
      <div>
        <Label for="secret-message" class="mb-2 block">Pesan Rahasia</Label>
        <Input
          bind:value={secretMessage}
          id="secret-message"
          placeholder="Masukkan pesan rahasia"
        />
      </div>
    {/if}
  {/if}
  {#snippet footer()}
    <Button disabled={isUploading} type="submit" value="send">Kirim</Button>
    <Button color="dark" disabled={isUploading} type="submit" value="cancel"
      >Batal</Button
    >
  {/snippet}
</Modal>
