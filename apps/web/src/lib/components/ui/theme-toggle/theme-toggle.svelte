<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import { Sun, Moon } from "lucide-svelte";

	let theme = $state(
		typeof window !== "undefined"
			? (localStorage.getItem("theme") as "light" | "dark") || "dark"
			: "dark"
	);

	function toggleTheme() {
		const newTheme = theme === "dark" ? "light" : "dark";
		theme = newTheme;
		if (typeof window !== "undefined") {
			localStorage.setItem("theme", newTheme);
			document.documentElement.setAttribute("data-theme", newTheme);
		}
	}

	$effect(() => {
		if (typeof window !== "undefined") {
			document.documentElement.setAttribute("data-theme", theme);
		}
	});
</script>

<Button variant="ghost" size="icon" onclick={toggleTheme} aria-label="Toggle theme">
	{#if theme === "dark"}
		<Moon class="h-4 w-4" />
	{:else}
		<Sun class="h-4 w-4" />
	{/if}
</Button>