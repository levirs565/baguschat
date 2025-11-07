<script lang="ts">
  import { encryptAffine } from "$lib/affine";
  import { activeContactId, chatWsStore } from "$lib/stores/chatStore";
  import { isSecretChatModalOpen } from "$lib/stores/uiStore";
  import { Button, Checkbox, Input, Label, Modal, P } from "flowbite-svelte";
  let message = $state("");
  let keyA = $state(1);
  let keyB = $state(1);

  function deriveEncrypt() {
    try {
      return {
        cipher: encryptAffine(keyA, keyB, message),
        canSubmit: true,
      };
    } catch (e: any) {
      return {
        cipher: `Error: ${e.message}`,
        canSubmit: false,
      };
    }
  }

  let output = $derived(deriveEncrypt());

  function onSubmit({ action }: { action: string }) {
    if (!$chatWsStore) {
      alert("Sesi terputus");
      return false;
    }

    if (!$activeContactId) {
      alert("Belum ada user yang dipilih");
      return false;
    }

    $chatWsStore.sendText($activeContactId, output.cipher);
    return true;
  }
</script>

<Modal
  form
  title="Kirim Pesan Rahasia"
  outsideclose={false}
  onaction={onSubmit}
  onclose={() => {
    keyA = 1;
    keyB = 1;
    message = "";
  }}
  bind:open={$isSecretChatModalOpen}
>
  <div class="flex flex-row items-center">
    <Label for="keyA" class="me-2">Kunci A</Label>
    <Input
      class="inline-block !w-24"
      id="keyA"
      type="number"
      placeholder="Angka"
      min={1}
      max={1000}
      bind:value={keyA}
    />

    <Label for="keyB" class="mx-2">Kunci B</Label>
    <Input
      class="inline-block !w-24"
      id="keyB"
      type="number"
      placeholder="Angka"
      min={0}
      max={1000}
      bind:value={keyB}
    />
  </div>
  <div>
    <Label for="message" class="mb-2 block">Pesan</Label>
    <Input
      bind:value={message}
      id="message"
      placeholder="Masukkan pesan rahasia"
    />
  </div>
  <div>
    <Label for="cipher" class="mb-2 block">Hasil Enkripsi</Label>
    <Input
      disabled
      bind:value={output.cipher}
      id="cipher"
      placeholder="Hasil Enkripsi"
    />
  </div>
  {#snippet footer()}
    <Button disabled={!output.canSubmit} type="submit" value="send"
      >Kirim</Button
    >
    <Button color="dark" type="submit" value="cancel">Batal</Button>
  {/snippet}
</Modal>
