<!--
  Ubicación: `apps/web/src/components/users/UserTable.svelte`

  Descripción: Tabla de usuarios con paginación, búsqueda y acciones.
               Integra RBAC para mostrar/ocultar botones según permisos.

  ADRs relacionados: 0022 (Frontend), 0006 (RBAC), 0006 (Soft Delete)
-->

<script lang="ts">
	import { onMount } from "svelte";
	import { authStore } from "$lib/stores/auth.svelte";
	import * as m from "$lib/paraglide/messages.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import Table from "$lib/components/ui/table/table.svelte";
	import TableBody from "$lib/components/ui/table/table-body.svelte";
	import TableCell from "$lib/components/ui/table/table-cell.svelte";
	import TableHead from "$lib/components/ui/table/table-head.svelte";
	import TableHeader from "$lib/components/ui/table/table-header.svelte";
	import TableRow from "$lib/components/ui/table/table-row.svelte";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { ChevronLeft, ChevronRight, Search, Trash2, RefreshCw } from "lucide-svelte";
	import { listUsers, softDeleteUser, restoreUser } from "$lib/api/users";
	import { PermissionGate } from "$lib/components/ui/permission-gate";
	import UserEditDrawer from "./UserEditDrawer.svelte";
	import type { User } from "$lib/types/user";

	// Estado
	let users = $state<User[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let search = $state("");
	let selectedRole = $state<string>("");
	let statusFilter = $state<string>("all");
	let page = $state(1);
	let perPage = $state(10);
	let total = $state(0);
	let totalPages = $derived(Math.ceil(total / perPage));

	// Drawer
	let selectedUser = $state<User | null>(null);
	let isDrawerOpen = $state(false);
	

	function openEdit(user: User) {
		selectedUser = user;
		isDrawerOpen = true;
	}

	// Cargar usuarios
	async function loadUsers() {
		loading = true;
		error = null;
		try {
			const result = await listUsers({
				page,
				perPage,
				search: search || undefined,
				role: selectedRole || undefined,
				isActive: statusFilter === "all" ? undefined : statusFilter === "active"
			});
			users = result.users;
			total = result.total;
		} catch (err) {
			console.error("Error loading users:", err);
			error = err instanceof Error ? err.message : "Failed to load users";
		} finally {
			loading = false;
		}
	}

	// Handlers
	async function handleDelete(id: string) {
		if (!confirm("Are you sure you want to delete this user?")) return;
		try {
			await softDeleteUser(id);
			await loadUsers();
		} catch (err) {
			alert(err instanceof Error ? err.message : "Delete failed");
		}
	}

	async function handleRestore(id: string) {
		try {
			await restoreUser(id);
			await loadUsers();
		} catch (err) {
			alert(err instanceof Error ? err.message : "Restore failed");
		}
	}

	function handleSearch() {
		page = 1;
		loadUsers();
	}

	function handlePrevPage() {
		if (page > 1) {
			page--;
			loadUsers();
		}
	}

	function handleNextPage() {
		if (page < totalPages) {
			page++;
			loadUsers();
		}
	}

	// Cargar al montar
	onMount(() => {
		loadUsers();
	});
</script>

<Card.Root>
	<Card.Header>
		<div class="flex items-center justify-between">
			<div>
				<Card.Title>Users</Card.Title>
				<Card.Description>
					Manage user accounts and permissions
				</Card.Description>
			</div>
			<!-- Botón movido a users.astro -->
		</div>
	</Card.Header>
	<Card.Content>
		<!-- Search -->
		<div class="flex flex-wrap gap-4 mb-6">
			<div class="flex-1 min-w-[200px]">
				<Input
					type="search"
					placeholder={m.search_placeholder_users()}
					bind:value={search}
					onkeydown={(e) => e.key === "Enter" && handleSearch()}
				/>
			</div>
			
			<select 
				bind:value={selectedRole} 
				onchange={handleSearch}
				class="bg-background border rounded px-3 py-2 text-sm focus:ring-2 focus:ring-primary outline-none"
			>
				<option value="">{m.filter_all_roles()}</option>
				<option value="admin">{m.role_admin()}</option>
				<option value="user">{m.role_user()}</option>
				<option value="moderator">{m.role_moderator()}</option>
			</select>

			<select 
				bind:value={statusFilter} 
				onchange={handleSearch}
				class="bg-background border rounded px-3 py-2 text-sm focus:ring-2 focus:ring-primary outline-none"
			>
				<option value="all">{m.filter_all_status()}</option>
				<option value="active">{m.filter_active_only()}</option>
				<option value="inactive">{m.filter_inactive_only()}</option>
			</select>

			<Button variant="secondary" onclick={handleSearch}>
				<Search class="h-4 w-4 mr-2" />
				{m.action_filter()}
			</Button>
			
			<Button variant="outline" onclick={loadUsers}>
				<RefreshCw class="h-4 w-4 mr-2" />
				{m.action_refresh()}
			</Button>
		</div>

		{#if loading}
			<div class="py-8 text-center text-muted-foreground">{m.loading()}</div>
		{:else if error}
			<div class="py-8 text-center">
				{#if error.includes("401")}
					<div class="text-amber-600 bg-amber-50 p-4 rounded">
						<p class="font-medium">{m.notification_error()}</p>
						<p class="text-sm mt-1">{m.error_generic()}</p>
						<a href="/login" class="inline-block mt-3 px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90">
							{m.login_title()}
						</a>
					</div>
				{:else}
					<div class="text-red-600 bg-red-50 p-4 rounded">
						{error}
					</div>
				{/if}
			</div>
		{:else if users.length === 0}
			<div class="py-8 text-center text-muted-foreground">
				{m.no_results()}
			</div>
		{:else}
			<div class="overflow-x-auto">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead>{m.table_user()}</TableHead>
							<TableHead>{m.table_role()}</TableHead>
							<TableHead>{m.table_status()}</TableHead>
							<TableHead>{m.table_last_login()}</TableHead>
							<TableHead class="text-right">{m.table_actions()}</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#each users as user (user.id)}
							<TableRow class={user.deletedAt ? "opacity-50" : ""}>
								<TableCell>
									<div class="font-medium">{user.name ?? "—"}</div>
									<div class="text-xs text-muted-foreground">{user.email}</div>
								</TableCell>
								<TableCell>
									<Badge variant="outline" class="capitalize">{user.role}</Badge>
								</TableCell>
								<TableCell>
									{#if user.deletedAt}
										<Badge variant="destructive">{m.status_deleted()}</Badge>
									{:else if user.isActive}
										<Badge variant="default" class="bg-emerald-500 hover:bg-emerald-600">{m.status_active()}</Badge>
									{:else}
										<Badge variant="secondary">{m.status_inactive()}</Badge>
									{/if}
								</TableCell>
								<TableCell class="text-xs">
									{user.lastLoginAt ? new Date(user.lastLoginAt).toLocaleString() : "—"}
								</TableCell>
<TableCell class="text-right">
									<PermissionGate permission="users:write">
										<Button
											variant="outline"
											size="sm"
											onclick={() => openEdit(user)}
										>
											{m.action_edit()}
										</Button>
									</PermissionGate>

									{#if user.deletedAt}
										<PermissionGate permission="users:write">
											<Button
												variant="outline"
												size="sm"
												onclick={() => handleRestore(user.id)}
											>
												{m.action_restore()}
											</Button>
										</PermissionGate>
									{:else}
										<PermissionGate permission="users:delete">
											<Button
												variant="destructive"
												size="sm"
												onclick={() => handleDelete(user.id)}
											>
												<Trash2 class="h-4 w-4" />
											</Button>
										</PermissionGate>
									{/if}
								</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			</div>

			<!-- Pagination -->
			<div class="flex items-center justify-between mt-4">
				<div class="text-sm text-muted-foreground">
					{m.pagination_showing({ count: users.length, total })}
				</div>
				<div class="flex gap-2">
					<Button
						variant="outline"
						size="sm"
						disabled={page <= 1}
						onclick={handlePrevPage}
					>
						<ChevronLeft class="h-4 w-4" />
					</Button>
					<span class="px-4 py-2 text-sm">
						{m.pagination_page({ current: page, total: totalPages })}
					</span>
					<Button
						variant="outline"
						size="sm"
						disabled={page >= totalPages}
						onclick={handleNextPage}
					>
						<ChevronRight class="h-4 w-4" />
					</Button>
				</div>
			</div>
		{/if}
	</Card.Content>
</Card.Root>

<UserEditDrawer 
	bind:open={isDrawerOpen} 
	user={selectedUser} 
	onUpdated={loadUsers} 
/>
