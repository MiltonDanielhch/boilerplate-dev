<script lang="ts">
	import { Switch as SwitchPrimitive } from "bits-ui";
	import { cn } from "$lib/utils";

	interface Props {
		checked?: boolean;
		onCheckedChange?: (checked: boolean) => void;
		disabled?: boolean;
		class?: string;
	}

	let {
		checked = $bindable(false),
		onCheckedChange,
		disabled = false,
		class: className = ""
	}: Props = $props();

	function handleChange(value: boolean) {
		checked = value;
		onCheckedChange?.(value);
	}
</script>

<SwitchPrimitive.Root
	{checked}
	{disabled}
	onCheckedChange={handleChange}
	class={cn(
		"peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-950 focus-visible:ring-offset-2 focus-visible:ring-offset-white disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-slate-900 data-[state=unchecked]:bg-slate-200 data-[state=checked]:hover:bg-slate-800 data-[state=unchecked]:hover:bg-slate-300",
		className
	)}
>
	<SwitchPrimitive.Thumb
		class={cn(
			"pointer-events-none block h-4 w-4 rounded-full bg-slate-50 shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
		)}
	/>
</SwitchPrimitive.Root>