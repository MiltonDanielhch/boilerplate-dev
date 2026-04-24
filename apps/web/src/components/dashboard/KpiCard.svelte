<!--
  Ubicación: `apps/web/src/components/dashboard/KpiCard.svelte`

  Descripción: Componente de métrica KPI para el dashboard.
               Muestra título, valor, badge de estado y cambio porcentual.

  ADRs relacionados: 0022 (Frontend), 0021 (OpenAPI)
-->

<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import Badge from "$lib/components/ui/badge/badge.svelte";
	import { TrendingUp, TrendingDown } from "lucide-svelte";

	interface Props {
		title: string;
		value: string | number;
		badge?: {
			text: string;
			variant: "default" | "secondary" | "outline" | "destructive";
		};
		change?: {
			value: number;
			label: string;
			type?: "percentage" | "neutral";
		};
	}

	let { title, value, badge, change }: Props = $props();

	function isPositiveChange(val: number): boolean {
		return val >= 0;
	}
</script>

<Card.Root>
	<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
		<Card.Title class="text-sm font-medium">{title}</Card.Title>
		{#if badge}
			<Badge variant={badge.variant}>{badge.text}</Badge>
		{/if}
	</Card.Header>
	<Card.Content>
		<div class="text-2xl font-bold">{value}</div>
		{#if change}
			<p class="text-xs text-muted-foreground flex items-center gap-1">
				{#if change.type === "neutral"}
					<span class="text-muted-foreground">{change.label}</span>
				{:else if isPositiveChange(change.value)}
					<TrendingUp class="h-3 w-3 text-green-500" />
					<span class="text-green-500">+{change.value}%</span>
					<span>{change.label}</span>
				{:else}
					<TrendingDown class="h-3 w-3 text-red-500" />
					<span class="text-red-500">{change.value}%</span>
					<span>{change.label}</span>
				{/if}
			</p>
		{:else}
			<p class="text-xs text-muted-foreground">—</p>
		{/if}
	</Card.Content>
</Card.Root>
