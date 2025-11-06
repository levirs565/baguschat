<script lang="ts">
	import { Modal, Button, Label, Input } from 'flowbite-svelte';
	import { isNewChatModalOpen } from '$lib/stores/uiStore.js';

	let username: string = '';

    function handleSearch() {
		console.log('Mencari:', username);
	}
</script>

<Modal title="Mulai Obrolan Baru" bind:open={$isNewChatModalOpen} autoclose>
	<form
        id="search-form"
		class="space-y-4"
		on:submit|preventDefault={handleSearch}
    >
		<div>
			<Label for="username" class="mb-2">Cari Berdasarkan Username</Label>
			<Input
				type="text"
				id="username"
				bind:value={username}
				placeholder="ketik username..."
				required
			/>
		</div>
	</form>

	<svelte:fragment>
		<Button type="submit" form="search-form" class="cursor-pointer">Cari & Mulai Chat</Button>
	</svelte:fragment>
</Modal>