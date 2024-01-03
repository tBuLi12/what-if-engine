<script lang="ts">
	import '$lib/app.css';

	import wasm, { add } from 'physics-engine';
	import Engine from '$lib/Engine.svelte';
</script>

{#await wasm()}
	Loading
{:then}
	<Engine binding={{ add }}></Engine>
{/await}
