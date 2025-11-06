<script>
	import { Avatar } from 'flowbite-svelte';
	import { isProfileModalOpen } from '$lib/stores/uiStore.js';
</script>

<button
	type="button"
	class="flex items-center cursor-pointer"
	on:click={() => isProfileModalOpen.set(true)}
	aria-label="Buka pengaturan profil"
>
	<Avatar src="" />
</button>