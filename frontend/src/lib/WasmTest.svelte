<script lang="ts">
	export let binding: any;
	let count = 0;
</script>

<button on:click={() => (count = binding.add(count, 2))}>
	count {count}
</button>
