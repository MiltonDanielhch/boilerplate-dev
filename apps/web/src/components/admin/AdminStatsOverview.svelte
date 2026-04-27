<script lang="ts">
	import { onMount } from "svelte";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Users, MousePointerClick, Activity, UserCheck } from "lucide-svelte";
	import api from "$lib/api/axios";

	interface Stats {
		total_users: number;
		active_users: number;
		total_leads: number;
		new_leads: number;
	}

	let stats = $state<Stats | null>(null);
	let loading = $state(true);

	async function loadStats() {
		try {
			const res = await api.get("/admin/stats");
			stats = res.data;
		} catch (err) {
			console.error("Failed to load admin stats", err);
		} finally {
			loading = false;
		}
	}

	onMount(() => loadStats());
</script>

<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
	<Card.Root class="bg-slate-900 border-slate-800">
		<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
			<Card.Title class="text-sm font-medium text-slate-400">Total Users</Card.Title>
			<Users class="h-4 w-4 text-blue-400" />
		</Card.Header>
		<Card.Content>
			{#if loading}
				<div class="h-8 w-16 bg-slate-800 animate-pulse rounded"></div>
			{:else}
				<div class="text-2xl font-bold">{stats?.total_users ?? 0}</div>
				<p class="text-xs text-slate-500 mt-1">Registered accounts</p>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root class="bg-slate-900 border-slate-800">
		<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
			<Card.Title class="text-sm font-medium text-slate-400">Active Users</Card.Title>
			<UserCheck class="h-4 w-4 text-green-400" />
		</Card.Header>
		<Card.Content>
			{#if loading}
				<div class="h-8 w-16 bg-slate-800 animate-pulse rounded"></div>
			{:else}
				<div class="text-2xl font-bold">{stats?.active_users ?? 0}</div>
				<p class="text-xs text-slate-500 mt-1">Unbanned accounts</p>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root class="bg-slate-900 border-slate-800">
		<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
			<Card.Title class="text-sm font-medium text-slate-400">Total Leads</Card.Title>
			<MousePointerClick class="h-4 w-4 text-purple-400" />
		</Card.Header>
		<Card.Content>
			{#if loading}
				<div class="h-8 w-16 bg-slate-800 animate-pulse rounded"></div>
			{:else}
				<div class="text-2xl font-bold">{stats?.total_leads ?? 0}</div>
				<p class="text-xs text-slate-500 mt-1">From landing page</p>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root class="bg-slate-900 border-slate-800">
		<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
			<Card.Title class="text-sm font-medium text-slate-400">New Leads</Card.Title>
			<Activity class="h-4 w-4 text-amber-400" />
		</Card.Header>
		<Card.Content>
			{#if loading}
				<div class="h-8 w-16 bg-slate-800 animate-pulse rounded"></div>
			{:else}
				<div class="text-2xl font-bold">{stats?.new_leads ?? 0}</div>
				<p class="text-xs text-amber-500/80 mt-1">Pending contact</p>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
