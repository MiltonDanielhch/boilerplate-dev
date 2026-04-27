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
	import { Shield, User, Globe, LogOut, RefreshCw, Smartphone, Laptop } from "lucide-svelte";
	import api from "$lib/api/axios";

	interface Session {
		id: string;
		user_id: string;
		ip_address: string | null;
		user_agent: string | null;
		last_activity_at: string;
		expires_at: string;
	}

	let sessions = $state<Session[]>([]);
	let loading = $state(true);
	let revoking = $state<string | null>(null);

	async function loadSessions() {
		loading = true;
		try {
			const res = await api.get("/admin/sessions");
			sessions = res.data;
		} catch (err) {
			console.error("Failed to load sessions", err);
		} finally {
			loading = false;
		}
	}

	async function handleRevoke(id: string) {
		if (!confirm("Are you sure you want to revoke this session? The user will be logged out immediately.")) return;
		
		revoking = id;
		try {
			await api.delete(`/admin/sessions/${id}`);
			sessions = sessions.filter(s => s.id !== id);
		} catch (err) {
			alert("Failed to revoke session");
		} finally {
			revoking = null;
		}
	}

	function isMobile(ua: string | null) {
		if (!ua) return false;
		return /Mobile|Android|iPhone/i.test(ua);
	}

	onMount(() => loadSessions());
</script>

<div class="space-y-6">
	<div class="grid gap-4 md:grid-cols-3">
		<Card.Root class="bg-slate-900 border-slate-800">
			<Card.Header class="pb-2">
				<Card.Title class="text-xs font-medium text-slate-400 uppercase tracking-wider">Active Sessions</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">{sessions.length}</div>
				<p class="text-[10px] text-green-500 mt-1 flex items-center gap-1">
					<span class="h-1.5 w-1.5 rounded-full bg-green-500 animate-pulse"></span>
					System-wide activity
				</p>
			</Card.Content>
		</Card.Root>
	</div>

	<Card.Root class="bg-slate-900 border-slate-800">
		<Card.Header>
			<div class="flex items-center justify-between">
				<div>
					<Card.Title>Session Management</Card.Title>
					<Card.Description>Review and terminate active user sessions</Card.Description>
				</div>
				<Button variant="outline" size="sm" onclick={loadSessions}>
					<RefreshCw class="h-4 w-4 mr-2" />
					Refresh
				</Button>
			</div>
		</Card.Header>
		<Card.Content>
			{#if loading}
				<div class="py-12 text-center text-slate-500">Loading active sessions...</div>
			{:else}
				<Table>
					<TableHeader>
						<TableRow class="border-slate-800 hover:bg-slate-900/50">
							<TableHead class="text-slate-400">Device / IP</TableHead>
							<TableHead class="text-slate-400">User ID</TableHead>
							<TableHead class="text-slate-400">Last Activity</TableHead>
							<TableHead class="text-slate-400">Expires</TableHead>
							<TableHead class="text-right text-slate-400">Actions</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#each sessions as session (session.id)}
							<TableRow class="border-slate-800 hover:bg-slate-900/50">
								<TableCell>
									<div class="flex items-center gap-3">
										{#if isMobile(session.user_agent)}
											<Smartphone class="h-4 w-4 text-slate-500" />
										{:else}
											<Laptop class="h-4 w-4 text-slate-500" />
										{/if}
										<div class="flex flex-col">
											<span class="text-xs font-mono text-slate-300">{session.ip_address ?? "Unknown"}</span>
											<span class="text-[10px] text-slate-600 truncate max-w-[200px]" title={session.user_agent}>
												{session.user_agent ?? "No agent info"}
											</span>
										</div>
									</div>
								</TableCell>
								<TableCell>
									<div class="flex items-center gap-2">
										<User class="h-3 w-3 text-slate-500" />
										<span class="text-xs font-mono text-slate-400">{session.user_id.slice(0,8)}...</span>
									</div>
								</TableCell>
								<TableCell class="text-xs text-slate-500">
									{new Date(session.last_activity_at).toLocaleString()}
								</TableCell>
								<TableCell class="text-xs text-slate-500">
									{new Date(session.expires_at).toLocaleTimeString()}
								</TableCell>
								<TableCell class="text-right">
									<Button 
										variant="ghost" 
										size="icon" 
										class="h-8 w-8 text-red-500 hover:text-red-400 hover:bg-red-500/10"
										onclick={() => handleRevoke(session.id)}
										disabled={revoking === session.id}
									>
										{#if revoking === session.id}
											<RefreshCw class="h-3 w-3 animate-spin" />
										{:else}
											<LogOut class="h-4 w-4" />
										{/if}
									</Button>
								</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
