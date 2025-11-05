<script>
	import MessageInput from './MessageInput.svelte';
</script>

<div class="flex flex-col h-full bg-gray-50 dark:bg-gray-900">
	<div class="p-4 border-b dark:border-gray-700 bg-white dark:bg-gray-800">
		<p class="font-semibold text-white">Pilih Obrolan</p>
	</div>

	<div class="flex-1 p-4 overflow-y-auto">
		<p class="text-center text-gray-500">
			Selamat datang! Silakan pilih kontak di sebelah kiri untuk memulai percakapan.
		</p>
	</div>

	<MessageInput />
</div>