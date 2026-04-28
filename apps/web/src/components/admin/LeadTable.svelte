<script lang="ts">
	import { onMount } from "svelte";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import Table from "$lib/components/ui/table/table.svelte";
	import TableBody from "$lib/components/ui/table/table-body.svelte";
	import TableCell from "$lib/components/ui/table/table-cell.svelte";
	import TableHead from "$lib/components/ui/table/table-head.svelte";
	import TableHeader from "$lib/components/ui/table/table-header.svelte";
	import TableRow from "$lib/components/ui/table/table-row.svelte";
	import { Search, RefreshCw, Mail, Calendar, User, Tag } from "lucide-svelte";
	import api from "$lib/api/axios";
	import * as m from "$lib/paraglide/messages.js";

	interface Lead {
		id: string;
		email: string;
		name: string | null;
		status: string;
		createdAt: string;
	}

	let leads = $state<Lead[]>([]);
	let loading = $state(true);
	let search = $state("");
	let statusFilter = $state("all");

	async function loadLeads() {
		loading = true;
		try {
			const params = new URLSearchParams();
			if (search) params.append("search", search);
			if (statusFilter !== "all") params.append("status", statusFilter);
			
			const res = await api.get(`/admin/leads?${params.toString()}`);
			leads = res.data.leads;
		} catch (err) {
			console.error("Failed to load leads", err);
		} finally {
			loading = false;
		}
	}

	async function updateStatus(id: string, newStatus: string) {
		try {
			await api.patch(`/admin/leads/${id}/status`, { status: newStatus });
			await loadLeads();
		} catch (err) {
			alert("Failed to update status");
		}
	}

	const statusColors: Record<string, string> = {
		new: "bg-blue-500",
		contacted: "bg-amber-500",
		qualified: "bg-emerald-500",
		converted: "bg-purple-500",
		archived: "bg-slate-500"
	};

	onMount(() => loadLeads());
</script>

<Card.Root class="bg-slate-900 border-slate-800">
	<Card.Header>
		<Card.Title>{m.leads_card_title()}</Card.Title>
		<Card.Description>{m.leads_card_desc()}</Card.Description>
	</Card.Header>
	<Card.Content>
		<div class="flex flex-wrap gap-4 mb-6">
			<div class="flex-1 min-w-[200px]">
				<Input 
					placeholder={m.leads_search_placeholder()} 
					bind:value={search} 
					onkeydown={(e) => e.key === "Enter" && loadLeads()}
					class="bg-slate-950 border-slate-800"
				/>
			</div>
			
			<select 
				bind:value={statusFilter} 
				onchange={loadLeads}
				class="bg-slate-950 border-slate-800 text-sm rounded-md px-3 py-2 outline-none focus:ring-2 focus:ring-primary"
			>
				<option value="all">{m.filter_all_status()}</option>
				<option value="new">{m.leads_status_new()}</option>
				<option value="contacted">{m.leads_status_contacted()}</option>
				<option value="qualified">{m.leads_status_qualified()}</option>
				<option value="converted">{m.leads_status_converted()}</option>
				<option value="archived">{m.leads_status_archived()}</option>
			</select>

			<Button variant="secondary" onclick={loadLeads}>
				<Search class="h-4 w-4 mr-2" />
				{m.action_filter()}
			</Button>
			
			<Button variant="outline" onclick={loadLeads}>
				<RefreshCw class="h-4 w-4 mr-2" />
				{m.action_refresh()}
			</Button>
		</div>

		{#if loading}
			<div class="py-12 text-center text-slate-500">{m.leads_loading()}</div>
		{:else if leads.length === 0}
			<div class="py-12 text-center text-slate-500">{m.leads_empty()}</div>
		{:else}
			<Table>
				<TableHeader>
					<TableRow class="border-slate-800 hover:bg-slate-900/50">
						<TableHead class="text-slate-400">{m.leads_table_info()}</TableHead>
						<TableHead class="text-slate-400">{m.table_status()}</TableHead>
						<TableHead class="text-slate-400">{m.leads_table_captured()}</TableHead>
						<TableHead class="text-right text-slate-400">{m.table_actions()}</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each leads as lead (lead.id)}
						<TableRow class="border-slate-800 hover:bg-slate-900/50">
							<TableCell>
								<div class="font-medium text-slate-200 flex items-center gap-2">
									<User class="h-3 w-3" />
									{lead.name ?? "Anonymous"}
								</div>
								<div class="text-xs text-slate-500 flex items-center gap-2 mt-1">
									<Mail class="h-3 w-3" />
									{lead.email}
								</div>
							</TableCell>
							<TableCell>
								<Badge class={`${statusColors[lead.status] || "bg-slate-500"} capitalize border-none`}>
									{lead.status}
								</Badge>
							</TableCell>
							<TableCell class="text-xs text-slate-400">
								<div class="flex items-center gap-1">
									<Calendar class="h-3 w-3" />
									{new Date(lead.createdAt).toLocaleDateString()}
								</div>
							</TableCell>
							<TableCell class="text-right">
								<div class="flex justify-end gap-2">
									{#if lead.status === 'new'}
										<Button size="sm" variant="outline" onclick={() => updateStatus(lead.id, 'contacted')}>
											Mark Contacted
										</Button>
									{:else if lead.status === 'contacted'}
										<Button size="sm" variant="secondary" onclick={() => updateStatus(lead.id, 'qualified')}>
											Qualify
										</Button>
									{/if}
									
									<select 
										value={lead.status}
										onchange={(e) => updateStatus(lead.id, (e.target as HTMLSelectElement).value)}
										class="bg-slate-950 border-slate-800 text-[10px] rounded px-2 py-1 outline-none"
									>
										<option value="new">Move to New</option>
										<option value="contacted">Move to Contacted</option>
										<option value="qualified">Move to Qualified</option>
										<option value="converted">Move to Converted</option>
										<option value="archived">Archive</option>
									</select>
								</div>
							</TableCell>
						</TableRow>
					{/each}
				</TableBody>
			</Table>
		{/if}
	</Card.Content>
</Card.Root>
