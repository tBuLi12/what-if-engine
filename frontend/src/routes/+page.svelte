<script lang="ts">
	import WasmTest from '$lib/WasmTest.svelte';
	/// @ts-ignore
	import wasm from '../../../physics-engine/Cargo.toml';

	console.log(wasm);
</script>

{#await wasm()}
	Loading
{:then binding}
	<WasmTest {binding}></WasmTest>
{/await}
