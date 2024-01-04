<script lang="ts">
	import type { Engine, Level, Point } from 'physics-engine';
	import { onMount } from 'svelte';
	import { Layer, Canvas, type Render } from 'svelte-canvas';

	export let PhysicsEngine: typeof Engine;

	let engine: Engine;

	onMount(() => {
		engine = PhysicsEngine.create({
			initial_ball_position: [0, 0],
			circles: [],
			flags_positions: [],
			polygons: []
		});

		return () => engine.free();
	});

	const render: Render = ({ context: ctx, width, height }) => {
		ctx.beginPath();
		ctx.moveTo(0, 0);
		ctx.lineTo(width, height);
		ctx.stroke();
	};
	let count = 0;
</script>

<div class="h-screen w-screen">
	<button on:click={() => console.log(engine.run_iteration(100))}>{count}</button>
	<Canvas>
		<Layer {render} />
	</Canvas>
</div>
