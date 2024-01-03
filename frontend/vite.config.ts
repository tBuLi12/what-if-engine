// @ts-ignore
import rust from '@wasm-tool/rollup-plugin-rust';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [rust(), sveltekit()]
});
