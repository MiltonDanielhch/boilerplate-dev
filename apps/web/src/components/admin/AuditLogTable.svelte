<script lang="ts">
	import { onMount } from "svelte";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import Table from "$lib/components/ui/table/table.svelte";
	import TableBody from "$lib/components/ui/table/table-body.svelte";
	import TableCell from "$lib/components/ui/table/table-cell.svelte";
	import TableHead from "$lib/components/ui/table/table-head.svelte";
	import TableHeader from "$lib/components/ui/table/table-header.svelte";
	import TableRow from "$lib/components/ui/table/table-row.svelte";
	import { Shield, User, Clock, Search, RefreshCw, Eye } from "lucide-svelte";
	import api from "$lib/api/axios";

	interface AuditEntry {
		id: string;
		timestamp: string;
		user_id: string | null;
		action: string;
		resource: string;
		resource_id: string | null;
		details: string | null;
		ip_address: string | null;
	}

	let logs = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let resourceFilter = $state("all");

	async function loadLogs() {
		loading = true;
		try {
			const params = new URLSearchParams();
			if (resourceFilter !== "all") params.append("resource", resourceFilter);
			
			const res = await api.get(`/audit?${params.toString()}`);
			logs = res.data.entries;
		} catch (err) {
			console.error("Failed to load audit logs", err);
		} finally {
			loading = false;
		}
	}

	function getActionColor(action: string) {
		if (action.includes('delete') || action.includes('ban')) return "bg-red-500";
		if (action.includes('create') || action.includes('assign')) return "bg-green-500";
		if (action.includes('update')) return "bg-amber-500";
		return "bg-slate-500";
	}

	onMount(() => loadLogs());
</script>

<Card.Root class="bg-slate-900 border-slate-800">
	<Card.Header>
		<div class="flex items-center justify-between">
			<div>
				<Card.Title>Action Log</Card.Title>
				<Card.Description>Immutable record of system events</Card.Description>
			</div>
			<Button variant="outline" size="sm" onclick={loadLogs}>
				<RefreshCw class="h-4 w-4 mr-2" />
				Refresh
			</Button>
		</div>
	</Card.Header>
	<Card.Content>
		<div class="flex gap-4 mb-6">
			<select 
				bind:value={resourceFilter} 
				onchange={loadLogs}
				class="bg-slate-950 border-slate-800 text-sm rounded-md px-3 py-2 outline-none focus:ring-2 focus:ring-primary"
			>
				<option value="all">All Resources</option>
				<option value="users">Users</option>
				<option value="leads">Leads</option>
				<option value="auth">Authentication</option>
				<option value="content">Content Blocks</option>
			</select>
		</div>

		{#if loading}
			<div class="py-12 text-center text-slate-500 animate-pulse">Loading security logs...</div>
		{:else if logs.length === 0}
			<div class="py-12 text-center text-slate-500">No logs found.</div>
		{:else}
			<Table>
				<TableHeader>
					<TableRow class="border-slate-800 hover:bg-slate-900/50">
						<TableHead class="text-slate-400">Timestamp</TableHead>
						<TableHead class="text-slate-400">Actor</TableHead>
						<TableHead class="text-slate-400">Action</TableHead>
						<TableHead class="text-slate-400">Resource</TableHead>
						<TableHead class="text-slate-400">IP Address</TableHead>
						<TableHead class="text-right text-slate-400">Details</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each logs as log (log.id)}
						<TableRow class="border-slate-800 hover:bg-slate-900/50">
							<TableCell class="text-xs font-mono text-slate-500">
								{new Date(log.timestamp).toLocaleString()}
							</TableCell>
							<TableCell>
								<div class="flex items-center gap-2">
									<User class="h-3 w-3 text-slate-400" />
									<span class="text-xs text-slate-300">{log.user_id?.slice(0,8) ?? "System"}</span>
								</div>
							</TableCell>
							<TableCell>
								<Badge class={`${getActionColor(log.action)} border-none text-[10px] uppercase font-bold`}>
									{log.action}
								</Badge>
							</TableCell>
							<TableCell class="text-xs text-slate-400">
								<span class="font-mono">{log.resource}</span>
								{#if log.resource_id}
									<span class="text-slate-600 ml-1">#{log.resource_id.slice(0,6)}</span>
								{/if}
							</TableCell>
							<TableCell class="text-xs text-slate-500 font-mono">
								{log.ip_address ?? "N/A"}
							</TableCell>
							<TableCell class="text-right">
								<Button variant="ghost" size="icon" class="h-8 w-8 text-slate-400">
									<Eye class="h-4 w-4" />
								</Button>
							</TableCell>
						</TableRow>
					{/each}
				</TableBody>
			</Table>
		{/if}
	</Card.Content>
</Card.Root>
