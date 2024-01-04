<script lang="ts">
	import { Layer, Canvas, type Render } from 'svelte-canvas';

	export let binding: any;

	const render: Render = ({ context: ctx, width, height }) => {
		ctx.beginPath();
		ctx.moveTo(0, 0);
		ctx.lineTo(width, height);
		ctx.stroke();
	};
	let count = 0;
	$: console.log(count);
</script>

<div class="h-screen w-screen">
	<button on:click={() => (count = binding.add(count, 2))}>{count}</button>
	<Canvas>
		<Layer {render} />
	</Canvas>
</div>
