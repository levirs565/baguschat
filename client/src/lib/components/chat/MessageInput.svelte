<script lang="ts">
  import { Textarea, Button, P } from "flowbite-svelte";
  import PaperClipOutline from "flowbite-svelte-icons/PaperClipOutline.svelte";
  import CameraPhotoOutline from "flowbite-svelte-icons/CameraPhotoOutline.svelte";
  import PaperPlaneOutline from "flowbite-svelte-icons/PaperPlaneOutline.svelte";
  import { activeContactId, chatWsStore } from "$lib/stores/chatStore";
  import Uppy, { type UppyFile } from "@uppy/core";
  import { onDestroy, onMount } from "svelte";
  import FilleDetailModal from "../modals/FilleDetailModal.svelte";
  import AwsS3, { type AwsS3UploadParameters } from "@uppy/aws-s3";
  import { sessionStore } from "$lib/stores/sessionStore";
  import type { CryptoService } from "$lib/api";
  import { toBase64 } from "@smithy/util-base64";
  import { encryptXcacha20 } from "$lib/crypto";
  import { ImageOutline, LockSolid } from "flowbite-svelte-icons";
  import Compressor from "@uppy/compressor";
  import { addEOFMessage } from "$lib/stegano";
  import SecretMessageModal from "../modals/SecretMessageModal.svelte";
  import { isSecretChatModalOpen } from "$lib/stores/uiStore";
  import ReadSecretMessageModal from "../modals/ReadSecretMessageModal.svelte";

  let messageText = "";

  function handleSend() {
    if (!messageText.trim()) return;

    let id = $activeContactId;
    let chatWs = $chatWsStore;
    if (!id || !chatWs) return;
    chatWs.sendText(id, messageText);
    messageText = "";
  }
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  }

  let fileInput: HTMLInputElement;
  let imageInput: HTMLInputElement;

  type FileMeta = Awaited<ReturnType<CryptoService["prepareFileKey"]>>;
  const uppy = new Uppy({
    autoProceed: false,
  });

  function onFileRemoved() {
    fileInput.value = "";
  }

  async function onFileUploaded(file: UppyFile<any, any>) {
    const chat_id = file.meta.chat_id;
    await $sessionStore?.apiService.finishFileChatUpload({
      id: chat_id,
    });
  }

  const encryptPreprocessor = async (fileIds: string[], uploadIds: string) => {
    for await (const fileId of fileIds) {
      const file = uppy.getFile(fileId);
      const meta = file.meta as any as FileMeta;
      if (!(file.data instanceof Blob || file.data instanceof File))
        throw new Error("Unsupported");
      const bytes = new Uint8Array(await file.data.arrayBuffer());
      const encrypted = encryptXcacha20(meta.plain_key, bytes);
      const blob = new Blob([encrypted]);

      uppy.setFileState(fileId, {
        type: file.type,
        name: file.name,
        data: blob,
        size: blob.size,
      });
    }
  };

  const steganoPreprocessor = async (fileIds: string[], uploadIds: string) => {
    for await (const fileId of fileIds) {
      const file = uppy.getFile(fileId);
      const meta = file.meta as any;

      if (!meta.secretMessage) continue;

      if (!(file.data instanceof Blob || file.data instanceof File))
        throw new Error("Unsupported");
      const bytes = await file.data.arrayBuffer();
      const encrypted = addEOFMessage(bytes, meta.secretMessage);
      const blob = new Blob([encrypted]);

      uppy.setFileState(fileId, {
        type: file.type,
        name: file.name,
        data: blob,
        size: blob.size,
      });
    }
  };

  const installCompressor = () => {
    uppy.use(Compressor, {
      maxHeight: 1000,
      maxWidth: 1000,
      quality: 0.8,
    });
  };

  const uninstallCompressor = () => {
    const plugin = uppy.getPlugin("Compressor");
    if (plugin) uppy.removePlugin(plugin);
  };

  onMount(() => {
    uppy.use(AwsS3, {
      limit: 1,
      shouldUseMultipart(file) {
        return false;
      },
      endpoint: "",
      async getUploadParameters(file, options): Promise<AwsS3UploadParameters> {
        const meta = file.meta as any as FileMeta;
        const parameters = await $sessionStore?.apiService.startFileChatUpload({
          file_type: (file.meta as any).mode,
          receiver_key: meta.receiver_key,
          sender_key: meta.sender_key,
          filename: file.name,
          receiver_id: $activeContactId!,
          mime_type: file.type,
          enrypted_size: file.size ?? 0,
          size: (file.meta as any).fileSize,
        });

        uppy.setFileMeta(file.id, {
          chat_id: parameters!.id,
        });

        return {
          url: parameters!.presign_url,
          method: "PUT",
          headers: {
            "content-type": file.type,
          },
        };
      },
    });

    const onFileChange = (event: Event) => {
      const files = Array.from((event.target as HTMLInputElement).files ?? []);
      const mode = event.target == fileInput ? "File" : "Image";

      uppy.removePreProcessor(encryptPreprocessor);
      uppy.removePreProcessor(steganoPreprocessor);
      uninstallCompressor();

      if (mode == "Image") {
        installCompressor();
        uppy.addPreProcessor(steganoPreprocessor);
      }
      uppy.addPreProcessor(encryptPreprocessor);

      files.forEach((file) => {
        try {
          uppy.addFile({
            source: "file input",
            name: file.name,
            type: file.type,
            data: file,
            meta: {
              mode: mode,
            },
          });
        } catch (err: any) {
          if (err.isRestriction) {
            console.log("Restriction error:", err);
          } else {
            console.error(err);
          }
        }
      });
    };

    fileInput.addEventListener("change", onFileChange);
    imageInput.addEventListener("change", onFileChange);

    uppy.on("file-removed", onFileRemoved);
    uppy.on("upload-success", onFileUploaded as any);
    uppy.on("complete", onFileRemoved);
  });

  onDestroy(() => {
    uppy.off("file-removed", onFileRemoved);
    uppy.off("upload-success", onFileUploaded as any);
    uppy.off("complete", onFileRemoved);
  });
</script>

<form
  class="p-4 bg-white dark:bg-gray-800 border-t dark:border-gray-700"
  class:hidden={!$activeContactId}
  on:submit={handleSend}
>
  <FilleDetailModal {uppy} />
  <SecretMessageModal />
  <ReadSecretMessageModal />

  <div class="flex items-center space-x-2">
    <input type="file" class="hidden" bind:this={fileInput} />
    <Button
      color="alternative"
      class="mr-2 p-2"
      onclick={() => fileInput.click()}
    >
      <PaperClipOutline class="w-6 h-6" />
    </Button>

    <input
      type="file"
      accept="image/png,image/jpeg"
      class="hidden"
      bind:this={imageInput}
    />
    <Button
      color="alternative"
      class="mr-2 p-2"
      onclick={() => imageInput.click()}
    >
      <ImageOutline class="w-6 h-6" />
    </Button>

    <Button
      color="alternative"
      class="mr-2 p-2"
      onclick={() => isSecretChatModalOpen.set(true)}
    >
      <LockSolid class="w-6 h-6" />
    </Button>

    <div class="relative flex-1" on:keydown={handleKeydown}>
      <Textarea
        bind:value={messageText}
        placeholder="Ketik pesan Anda..."
        rows={1}
        class="w-full resize-none pr-14"
      />

      <Button type="submit" class="absolute right-2.5 bottom-2 p-2">
        <PaperPlaneOutline class="w-5 h-5 rotate-z-90" />
      </Button>
    </div>
  </div>
</form>
