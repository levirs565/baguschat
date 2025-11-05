import { redirect } from '@sveltejs/kit';

export function load() {
	// 307 adalah kode status HTTP untuk "Temporary Redirect"
	// Ini akan langsung mengalihkan siapa pun yang mengunjungi "/"
	// ke halaman "/login".
	throw redirect(307, '/login');
}