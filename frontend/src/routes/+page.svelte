<script lang="ts">
	import '$lib/app.css';

	import wasm, { Engine, Init } from 'physics-engine';

	import EngineView from '$lib/EngineView.svelte';
</script>

{#await wasm()}
	Loading
{:then}
	<EngineView binding={{}}></EngineView>
{/await}
