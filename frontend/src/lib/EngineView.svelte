<script lang="ts">
	import type { Engine, Level, Point, DisplayMessage } from 'physics-engine';
	import { onMount } from 'svelte';
	import { Layer, Canvas, type Render } from 'svelte-canvas';

	export let PhysicsEngine: typeof Engine;

	let engine: Engine;
	let canvas: HTMLCanvasElement;
	let dt = 0;
	function renderbinding(ctx: CanvasRenderingContext2D, positions: Point[], color: string) {
		positions.forEach((binding) => {
			ctx.fillStyle = color;
			ctx.beginPath();
			ctx.arc(...binding, 7, 0, 2 * Math.PI);
			ctx.fill();
			ctx.stroke();
		});
	}
	onMount(() => {
		let destroyed = false;
		engine = PhysicsEngine.create({
			initial_ball_position: [0, 0],
			circles: [
				{
					is_bindable: false,
					is_static: false,
					shape: { center: [10, 10], radius: 10 }
				}
			],
			flags_positions: [],
			polygons: []
		});

		let pixelRatio = window.devicePixelRatio;
		let ctx = canvas.getContext('2d')!;
		let rect = canvas.getBoundingClientRect();

		canvas.width = Math.round(pixelRatio * rect.right) - Math.round(pixelRatio * rect.left);
		canvas.height = Math.round(pixelRatio * rect.bottom) - Math.round(pixelRatio * rect.top);

		ctx.scale(pixelRatio, pixelRatio);

		let render = (time: number) => {
			if (destroyed) {
				return;
			}

			let ctx = canvas.getContext('2d')!;
			let { width, height } = canvas;

			let message = engine.run_iteration((time - dt) * 1000);
			dt = time;
			ctx.clearRect(0, 0, width, height);
			message.polygons.forEach((polygon) => {
				let color = polygon.color;
				ctx.fillStyle = `rgba(${color[0]}, ${color[1]}, ${color[2]}, 1)`;
				ctx.beginPath();
				let startPoint = polygon.shape.vertices[0];
				ctx.moveTo(...startPoint);
				polygon.shape.vertices.slice(1).forEach((vertex) => {
					ctx.lineTo(...vertex);
				});
				ctx.closePath();
				ctx.fill();
				ctx.stroke();
			});
			message.circles.forEach((circle) => {
				let color = circle.color;
				ctx.fillStyle = `rgba(${color[0]}, ${color[1]}, ${color[2]}, 1)`;
				ctx.beginPath();
				ctx.arc(...circle.shape.center, circle.shape.radius, 0, 2 * Math.PI);
				ctx.fill();
				ctx.stroke();
			});
			message.flags.forEach((flag) => {
				ctx.fillStyle = 'black';
				ctx.beginPath();
				let startPoint = flag.vertices[0];
				ctx.moveTo(...startPoint);
				flag.vertices.slice(1).forEach((vertex) => {
					ctx.lineTo(...vertex);
				});
				ctx.closePath();
				ctx.fill();
				ctx.stroke();
			});
			renderbinding(ctx, message.rigid_bindings, 'red');
			renderbinding(ctx, message.unbound_hinges, 'orange');
			renderbinding(ctx, message.hinges, 'blue');
			renderbinding(ctx, message.unbound_hinges, 'green');

			requestAnimationFrame(render);
		};

		requestAnimationFrame(render);

		return () => {
			engine.free();
			destroyed = true;
		};
	});

	let count = 0;
</script>

<div class="h-screen w-screen p-8 flex flex-col">
	<div>
		<button on:click={() => console.log('lmao')}>{count}</button>
	</div>
	<div class="grow min-h-0 flex gap-8">
		<div class="grow-[2]">
			<div class="overflow-hidden aspect-square max-h-full mx-auto">
				<canvas class="w-full h-full" bind:this={canvas}> </canvas>
			</div>
		</div>
		<div class="grow">controls</div>
	</div>
</div>
