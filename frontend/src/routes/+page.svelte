<script lang="ts">
	import '$lib/app.css';

	import wasm, { Engine } from 'physics-engine';

	import EngineView from '$lib/EngineView.svelte';
</script>

{#await wasm()}
	Loading
{:then}
	<EngineView PhysicsEngine={Engine}></EngineView>
{/await}
