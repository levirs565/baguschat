<script lang="ts">
	import { Modal, Label, Input, Avatar } from 'flowbite-svelte';
	import { isProfileModalOpen } from '$lib/stores/uiStore.js';
	import { authStore } from '$lib/stores/authStore.js';

	type User = {
		username: string;
		name: string;
	} | null;

	type AuthStore = {
		user: User;
	};

	$: user = ($authStore as AuthStore).user;

	$: name = user?.name || '';

	let previewUrl = 'https://via.placeholder.com/150/808080/FFFFFF?text=U';

	function handleFileSelect(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files[0]) {
			const file = target.files[0];
			previewUrl = URL.createObjectURL(file);
		}
	}

	function handleSave() {
		console.log('Menyimpan nama:', name);
		isProfileModalOpen.set(false);
	}
</script>

<Modal title="Pengaturan Profil" bind:open={$isProfileModalOpen}>
	<div class="flex flex-col items-center space-y-4">
		<Avatar src={previewUrl} size="lg" />

		<input
			type="file"
			id="avatar-upload"
			class="hidden"
			accept="image/png, image/jpeg, image/jpg"
			on:change={handleFileSelect}
		/>

		<button
			type="button"
			on:click={() => document.getElementById('avatar-upload')?.click()}
			class="cursor-pointer text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-4 focus:ring-gray-100 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-gray-800 dark:text-white dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:border-gray-600 dark:focus:ring-gray-700"
		>
			Ganti Foto
		</button>
	</div>

	<form id="profile-form" class="space-y-4 mt-6" on:submit|preventDefault={handleSave}>
		<div>
			<Label for="name" class="mb-2">Nama Tampilan</Label>
			<Input type="text" id="name" bind:value={name} />
		</div>
		<div>
			<Label for="username" class="mb-2">Username</Label>
			<Input type="text" id="username" value={user?.username || ''} disabled />
		</div>
	</form>

	<svelte:fragment>
		<button
			type="submit"
			form="profile-form"
			class="cursor-pointer text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
		>
			Simpan Perubahan
		</button>

		<button
			type="button"
			on:click={() => isProfileModalOpen.set(false)}
			class="cursor-pointer text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-gray-800 dark:text-gray-400 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700"
		>
			Batal
		</button>
	</svelte:fragment>
</Modal>