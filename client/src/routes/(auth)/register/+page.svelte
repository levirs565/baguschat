<script>
	import { Card, Button, Label, Input, Heading } from 'flowbite-svelte';

	let email = '';
	let password = '';
	let confirmPassword = '';

	function handleRegister() {
		if (password !== confirmPassword) {
			alert('Password tidak cocok!');
			return;
		}
		console.log('Mencoba mendaftar dengan:', { email, password });
	}
</script>

<svelte:head>
	<title>Register</title>
</svelte:head>

<Card class="w-full max-w-md p-6">
	<form class="flex flex-col space-y-6" on:submit|preventDefault={handleRegister}>
		<Heading tag="h3" class="text-xl font-medium text-gray-900 dark:text-white">
			Buat Akun Baru
		</Heading>

		<div>
			<Label for="username" class="mb-2">Username Anda</Label>
			<Input type="text" id="username" bind:value={email} placeholder="username" required />
		</div>

		<div>
			<Label for="password" class="mb-2">Password</Label>
			<Input
				type="password"
				id="password"
				bind:value={password}
				placeholder="••••••••"
				required
			/>
		</div>

		<div>
			<Label for="confirmPassword" class="mb-2">Konfirmasi Password</Label>
			<Input
				type="password"
				id="confirmPassword"
				bind:value={confirmPassword}
				placeholder="••••••••"
				required
			/>
		</div>

		<Button type="submit" class="w-full">Buat Akun</Button>

		<div class="text-sm font-medium text-gray-500 dark:text-gray-300">
			Sudah punya akun?
			<a href="/login" class="text-blue-700 hover:underline dark:text-blue-500">
				Login di sini
			</a>
		</div>
	</form>
</Card>
