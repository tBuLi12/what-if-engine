<script lang="ts">
	import type { Engine, Level, Point, DisplayMessage } from 'physics-engine';
	import { onMount } from 'svelte';
	import { Layer, Canvas, type Render } from 'svelte-canvas';

	export let PhysicsEngine: typeof Engine;

	let engine: Engine;
	let render: Render = () => {};
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
		engine = PhysicsEngine.create({
			initial_ball_position: [0, 0],
			circles: [],
			flags_positions: [],
			polygons: []
		});
		render = ({ context: ctx, width, height, time }) => {
			let message = engine.run_iteration(time - dt);
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
		};
		return () => engine.free();
	});

	let count = 0;
</script>

<div class="h-screen w-screen">
	<button on:click={() => console.log(engine.run_iteration(100))}>{count}</button>
	<Canvas>
		<Layer render={(props) => render(props)} />
	</Canvas>
</div>