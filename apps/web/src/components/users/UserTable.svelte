<!--
  Ubicación: `apps/web/src/components/users/UserTable.svelte`

  Descripción: Tabla de usuarios con paginación, búsqueda y acciones.
               Integra RBAC para mostrar/ocultar botones según permisos.

  ADRs relacionados: 0022 (Frontend), 0006 (RBAC), 0006 (Soft Delete)
-->

<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import Table from "$lib/components/ui/table/table.svelte";
	import TableBody from "$lib/components/ui/table/table-body.svelte";
	import TableCell from "$lib/components/ui/table/table-cell.svelte";
	import TableHead from "$lib/components/ui/table/table-head.svelte";
	import TableHeader from "$lib/components/ui/table/table-header.svelte";
	import TableRow from "$lib/components/ui/table/table-row.svelte";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { ChevronLeft, ChevronRight, Search, Trash2, RefreshCw } from "lucide-svelte";
	import { listUsers, softDeleteUser, restoreUser } from "$lib/api/users";
	import { PermissionGate } from "$lib/components/ui/permission-gate";
	import type { User } from "$lib/types/user";

	// Estado
	let users = $state<User[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let search = $state("");
	let page = $state(1);
	let perPage = $state(10);
	let total = $state(0);
	let totalPages = $derived(Math.ceil(total / perPage));

	// Cargar usuarios
	async function loadUsers() {
		loading = true;
		error = null;
		try {
			const result = await listUsers({
				page,
				perPage,
				search: search || undefined
			});
			users = result.users;
			total = result.total;
		} catch (err) {
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
		<Card.Title>Users</Card.Title>
		<Card.Description>
			Manage user accounts and permissions
		</Card.Description>
	</Card.Header>
	<Card.Content>
		<!-- Search -->
		<div class="flex gap-2 mb-4">
			<Input
				type="search"
				placeholder="Search users..."
				bind:value={search}
				onkeydown={(e) => e.key === "Enter" && handleSearch()}
				class="max-w-sm"
			/>
			<Button variant="secondary" onclick={handleSearch}>
				<Search class="h-4 w-4 mr-2" />
				Search
			</Button>
			<Button variant="outline" onclick={loadUsers}>
				<RefreshCw class="h-4 w-4 mr-2" />
				Refresh
			</Button>
		</div>

		{#if loading}
			<div class="py-8 text-center text-muted-foreground">Loading...</div>
		{:else if error}
			<div class="py-8 text-center">
				{#if error.includes("401")}
					<div class="text-amber-600 bg-amber-50 p-4 rounded">
						<p class="font-medium">Authentication required</p>
						<p class="text-sm mt-1">Please log in to view users</p>
						<a href="/login" class="inline-block mt-3 px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90">
							Go to Login
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
				No users found
			</div>
		{:else}
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead>Email</TableHead>
						<TableHead>Name</TableHead>
						<TableHead>Status</TableHead>
						<TableHead>Created</TableHead>
						<TableHead class="text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each users as user (user.id)}
						<TableRow class={user.deletedAt ? "opacity-50" : ""}>
							<TableCell>{user.email}</TableCell>
							<TableCell>{user.name ?? "—"}</TableCell>
							<TableCell>
								{#if user.deletedAt}
									<Badge variant="destructive">Deleted</Badge>
								{:else if user.isActive}
									<Badge variant="default">Active</Badge>
								{:else}
									<Badge variant="secondary">Inactive</Badge>
								{/if}
							</TableCell>
							<TableCell>
								{new Date(user.createdAt).toLocaleDateString()}
							</TableCell>
							<TableCell class="text-right">
								{#if user.deletedAt}
									<PermissionGate permission="users:write">
										<Button
											variant="outline"
											size="sm"
											onclick={() => handleRestore(user.id)}
										>
											Restore
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

			<!-- Pagination -->
			<div class="flex items-center justify-between mt-4">
				<div class="text-sm text-muted-foreground">
					Showing {users.length} of {total} users
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
						Page {page} of {totalPages}
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
