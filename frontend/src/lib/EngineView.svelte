<script lang="ts">
	import type { Engine, Point } from 'physics-engine';
	import { onMount } from 'svelte';

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
					is_static: true,
					shape: { center: [100, 900], radius: 10 }
				}
			],
			flags_positions: [],
			polygons: [
				{
					is_bindable: false,
					is_static: true,
					shape: [
						[100, 1030],
						[100, 1000],
						[1600, 1005],
						[1600, 1035]
					]
				},
				{
					is_bindable: true,
					is_static: true,
					shape: [
						[500, 600],
						[500, 500],
						[600, 500],
						[600, 600]
					]
				}
			]
		});

		let observer = new ResizeObserver(() => {
			let pixelRatio = window.devicePixelRatio;
			let rect = canvas.getBoundingClientRect();
			canvas.width = Math.round(pixelRatio * rect.right) - Math.round(pixelRatio * rect.left);
			canvas.height = Math.round(pixelRatio * rect.bottom) - Math.round(pixelRatio * rect.top);
		});

		observer.observe(canvas);

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
				ctx.fillStyle = `rgb(${Math.floor(color[0] * 256)} ${Math.floor(
					color[1] * 256
				)} ${Math.floor(color[2] * 256)})`;
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
				ctx.fillStyle = `rgb(${Math.floor(color[0] * 256)} ${Math.floor(
					color[1] * 256
				)} ${Math.floor(color[2] * 256)})`;
				ctx.beginPath();
				ctx.arc(...circle.shape.center, circle.shape.radius, 0, 2 * Math.PI);
				ctx.fill();
				ctx.stroke();
			});
			switch (state.kind) {
				case StateKind.DrawingCircle: {
					ctx.beginPath();
					ctx.arc(...state.center, getRadius(state.startTime), 0, 2 * Math.PI);
					ctx.stroke();
					break;
				}
				case StateKind.DrawingPoly: {
					ctx.beginPath();
					const [x, y] = state.path[0];
					ctx.moveTo(x, y);
					for (const [x, y] of state.path.slice(1)) {
						ctx.lineTo(x, y);
					}
					ctx.stroke();
					break;
				}
				case StateKind.Waiting: {
					break;
				}
				case StateKind.None: {
					break;
				}
			}
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
			observer.disconnect();
		};
	});

	function mapCoordinatesOf(event: MouseEvent): { x: number; y: number } {
		let rect = canvas.getBoundingClientRect();
		return {
			x: Math.round(event.x * devicePixelRatio) - Math.round(rect.left * devicePixelRatio),
			y: Math.round(event.y * devicePixelRatio) - Math.round(rect.top * devicePixelRatio)
		};
	}

	enum StateKind {
		DrawingCircle,
		DrawingPoly,
		Waiting,
		None
	}

	type State =
		| { kind: StateKind.DrawingCircle; center: Point; startTime: Date }
		| { kind: StateKind.DrawingPoly; path: Point[] }
		| { kind: StateKind.Waiting; circleTriggerTimer: ReturnType<typeof setTimeout> }
		| { kind: StateKind.None };

	let state: State = { kind: StateKind.None };

	function onMouseDown(event: MouseEvent) {
		let { x, y } = mapCoordinatesOf(event);

		switch (key) {
			case Key.Ereaser: {
				engine.erase_at(x, y);
				break;
			}
			case Key.Rigid: {
				engine.add_rigid(x, y);
				break;
			}
			case Key.Hinge: {
				engine.add_hinge(x, y);
				break;
			}
			case Key.None: {
				state = {
					kind: StateKind.Waiting,
					circleTriggerTimer: setTimeout(() => {
						state = { kind: StateKind.DrawingCircle, center: [x, y], startTime: new Date() };
					}, 400)
				};
				break;
			}
		}
	}

	function getRadius(start: Date): number {
		return (+new Date() - +start) / 20;
	}

	function onMouseUp(_: MouseEvent) {
		switch (state.kind) {
			case StateKind.DrawingCircle: {
				let radius = getRadius(state.startTime);
				engine.add_circle(state.center[0], state.center[1], radius);
				state = { kind: StateKind.None };
				break;
			}
			case StateKind.DrawingPoly: {
				if (state.path.length > 3) {
					engine.add_polygon({ vertices: state.path });
				}
				state = { kind: StateKind.None };
				break;
			}
			case StateKind.Waiting: {
				clearTimeout(state.circleTriggerTimer);
				state = { kind: StateKind.None };
				break;
			}
			case StateKind.None: {
				break;
			}
		}
	}

	function onMouseMove(event: MouseEvent) {
		switch (state.kind) {
			case StateKind.DrawingCircle: {
				break;
			}
			case StateKind.DrawingPoly: {
				let { x, y } = mapCoordinatesOf(event);
				state.path.push([x, y]);
				break;
			}
			case StateKind.Waiting: {
				clearTimeout(state.circleTriggerTimer);
				let { x, y } = mapCoordinatesOf(event);
				state = { kind: StateKind.DrawingPoly, path: [[x, y]] };
				break;
			}
			case StateKind.None: {
				break;
			}
		}
	}

	let gravity = 1.0;
	let friction = 1.0;
	let restitution = 1.0;
	let dynamic_enabled = true;
	let static_enabled = true;
	$: engine?.set_gravity_multipier(gravity);
	$: engine?.set_friction_multipier(friction);
	$: engine?.set_restitution_multipier(restitution);
	$: engine?.set_dynamic_friction(dynamic_enabled);
	$: engine?.set_static_friction(static_enabled);

	enum Key {
		Ereaser,
		Rigid,
		Hinge,
		None
	}

	let key = Key.None;

	function onKeyDown(event: KeyboardEvent) {
		switch (event.code) {
			case 'KeyD': {
				key = Key.Hinge;
				break;
			}
			case 'KeyA': {
				key = Key.Ereaser;
				break;
			}
			case 'KeyS': {
				key = Key.Rigid;
				break;
			}
		}
	}

	function onKeyUp(event: KeyboardEvent) {
		switch (event.code) {
			case 'KeyD': {
				if (key == Key.Hinge) {
					key = Key.None;
				}
				break;
			}
			case 'KeyA': {
				if (key == Key.Ereaser) {
					key = Key.None;
				}
				break;
			}
			case 'KeyS': {
				if (key == Key.Rigid) {
					key = Key.None;
				}
				break;
			}
		}
	}
</script>

<svelte:window on:keydown={onKeyDown} on:keyup={onKeyUp} />

<div class="h-screen w-screen p-8 flex flex-col">
	<div class="grow min-h-0 flex gap-8">
		<div class="grow-[2] basis-0">
			<div class="overflow-hidden aspect-square max-h-full mx-auto border border-gray-400">
				<canvas
					class="w-full h-full"
					bind:this={canvas}
					on:mousedown={onMouseDown}
					on:mouseup={onMouseUp}
					on:mousemove={onMouseMove}
				>
				</canvas>
			</div>
		</div>
		<div class="grow basis-0 flex flex-col">
			<span>gravity {gravity}</span>
			<input type="range" min="-10" max="10" step="0.1" bind:value={gravity} />
			<div class="flex">
				<span class="mr-auto">-10</span>
				<span>10</span>
			</div>
			<span class="pt-6">friction {friction}</span>
			<input type="range" min="0" max="10" step="0.1" bind:value={friction} />
			<div class="flex">
				<span class="mr-auto">0</span>
				<span>10</span>
			</div>
			<span class="pt-6">restitution {restitution}</span>
			<input type="range" min="-10" max="10" step="0.1" bind:value={restitution} />
			<div class="flex">
				<span class="mr-auto">-10</span>
				<span>10</span>
			</div>
			<div class="pt-4">
				<input type="checkbox" bind:checked={static_enabled} />
				<span>static friction</span>
			</div>
			<div class="pt-4">
				<input type="checkbox" bind:checked={dynamic_enabled} />
				<span>dynamic friction</span>
			</div>
			<div class="mt-4 pt-4 border-t border-gray-400 text-justify">
				The physics engine used for the simulation on the right is our toy physics engine written in
				Rust and compiled to WebAssembly. The engine uses two main algorithms to compute the
				collisions. First, the
				<a
					target="_blank"
					href="https://en.wikipedia.org/wiki/Gilbert%E2%80%93Johnson%E2%80%93Keerthi_distance_algorithm"
					>GJK</a
				>
				algorithm helps determine werther two shapes are in contact. If they are, the
				<a target="_blank" href="https://dyn4j.org/2010/05/epa-expanding-polytope-algorithm/">EPA</a
				>
				algorithm picks up at the collision vector, and finds the minimum penetration vector. Based on
				that, an
				<a
					target="_blank"
					href="https://en.wikipedia.org/wiki/Collision_response#Impulse-based_contact_model"
				>
					impulse</a
				>
				is calculated, which is then used to apply momentum to the relevant shapes at each time tick.
			</div>
		</div>
	</div>
</div>

<style>
	a {
		color: blue;
		text-decoration: underline;
	}
</style>
