<script lang="ts">
  import { decryptAffine } from "$lib/affine";
  import {
    isReadSecretChatModalOpen,
    readSecretChatMessage,
  } from "$lib/stores/uiStore";
  import { Button, Input, Label, Modal, P } from "flowbite-svelte";
  let keyA = $state(1);
  let keyB = $state(1);

  function deriveDecrypt() {
    try {
      return decryptAffine(keyA, keyB, $readSecretChatMessage);
    } catch (e: any) {
      return `Error: ${e.message}`;
    }
  }

  let output = $derived(deriveDecrypt());
</script>

<Modal
  form
  title="Baca Pesan Rahasia"
  outsideclose={false}
  onclose={() => {
    keyA = 1;
    keyB = 1;
    $readSecretChatMessage = "";
  }}
  bind:open={$isReadSecretChatModalOpen}
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
    <Label for="cipher" class="mb-2 block">Cipher</Label>
    <Input disabled bind:value={$readSecretChatMessage} id="cipher" />
  </div>
  <div>
    <Label for="message" class="mb-2 block">Hasil Dekripsi</Label>
    <Input
      disabled
      bind:value={output}
      id="message"
      placeholder="Hasil dekripsi"
    />
  </div>

  {#snippet footer()}
    <Button color="dark" type="submit" value="cancel">Keluar</Button>
  {/snippet}
</Modal>
