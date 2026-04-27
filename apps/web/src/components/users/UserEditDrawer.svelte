<script lang="ts">
	import * as Sheet from "$lib/components/ui/sheet/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { updateUser, impersonateUser } from "$lib/api/users";
	import type { User } from "$lib/types/user";
	import { Shield, User as UserIcon, UserCog, Mail, Calendar, LogIn } from "lucide-svelte";

	let { user, open = $bindable(false), onUpdated } = $props<{
		user: User | null;
		open: boolean;
		onUpdated?: () => void;
	}>();

	let loading = $state(false);
	let name = $state("");
	let role = $state<User["role"]>("user");
	let isActive = $state(true);

	$effect(() => {
		if (user) {
			name = user.name ?? "";
			role = user.role;
			isActive = user.isActive;
		}
	});

	async function handleSave() {
		if (!user) return;
		loading = true;
		try {
			await updateUser(user.id, {
				name,
				role,
				isActive
			});
			open = false;
			onUpdated?.();
		} catch (err) {
			alert(err instanceof Error ? err.message : "Update failed");
		} finally {
			loading = false;
		}
	}

	async function handleImpersonate() {
		if (!user) return;
		if (!confirm(`Are you sure you want to impersonate ${user.email}? You will be logged in as them.`)) return;
		
		try {
			const token = await impersonateUser(user.id);
			// Guardar el nuevo token y refrescar
			localStorage.setItem("access_token", token);
			window.location.href = "/dashboard";
		} catch (err) {
			alert(err instanceof Error ? err.message : "Impersonation failed");
		}
	}
</script>

<Sheet.Root bind:open>
	<Sheet.Content side="right" class="w-[400px] sm:w-[540px]">
		<Sheet.Header>
			<Sheet.Title class="flex items-center gap-2">
				<UserCog class="h-5 w-5 text-primary" />
				Edit User
			</Sheet.Title>
			<Sheet.Description>
				Update user profile, roles, and account status.
			</Sheet.Description>
		</Sheet.Header>

		{#if user}
			<div class="space-y-6 py-6">
				<!-- Quick Info -->
				<div class="flex items-center gap-4 p-4 bg-muted/50 rounded-lg">
					<div class="h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center">
						<UserIcon class="h-6 w-6 text-primary" />
					</div>
					<div>
						<div class="font-semibold">{user.email}</div>
						<div class="text-xs text-muted-foreground flex items-center gap-1">
							<Shield class="h-3 w-3" />
							ID: {user.id.slice(0, 8)}...
						</div>
					</div>
				</div>

				<!-- Form -->
				<div class="space-y-4">
					<div class="grid gap-2">
						<Label for="name">Full Name</Label>
						<Input id="name" bind:value={name} placeholder="John Doe" />
					</div>

					<div class="grid gap-2">
						<Label for="role">Role</Label>
						<select 
							id="role"
							bind:value={role}
							class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
						>
							<option value="user">User</option>
							<option value="moderator">Moderator</option>
							<option value="admin">Admin</option>
						</select>
					</div>

					<div class="flex items-center justify-between p-4 border rounded-lg">
						<div class="space-y-0.5">
							<Label>Account Status</Label>
							<div class="text-xs text-muted-foreground">
								Inactive users cannot log in.
							</div>
						</div>
						<Button 
							variant={isActive ? "outline" : "destructive"} 
							onclick={() => isActive = !isActive}
							size="sm"
						>
							{isActive ? "Active" : "Banned"}
						</Button>
					</div>
				</div>

				<!-- Metadata -->
				<div class="space-y-3 pt-4 border-t text-sm text-muted-foreground">
					<div class="flex items-center gap-2">
						<Calendar class="h-4 w-4" />
						Created: {new Date(user.createdAt).toLocaleDateString()}
					</div>
					<div class="flex items-center gap-2">
						<LogIn class="h-4 w-4" />
						Last Login: {user.lastLoginAt ? new Date(user.lastLoginAt).toLocaleString() : "Never"}
					</div>
				</div>

				<!-- Dangerous Actions -->
				<div class="pt-6 space-y-4">
					<Label class="text-red-500 font-bold">Admin Actions</Label>
					<Button 
						variant="secondary" 
						class="w-full justify-start" 
						onclick={handleImpersonate}
					>
						<Shield class="h-4 w-4 mr-2" />
						Impersonate User
					</Button>
				</div>
			</div>
		{/if}

		<Sheet.Footer class="absolute bottom-0 left-0 right-0 p-6 bg-background border-t">
			<Button variant="outline" onclick={() => open = false} disabled={loading}>
				Cancel
			</Button>
			<Button onclick={handleSave} disabled={loading}>
				{loading ? "Saving..." : "Save Changes"}
			</Button>
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>
