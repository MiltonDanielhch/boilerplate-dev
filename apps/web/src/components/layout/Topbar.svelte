<!--
  Ubicación: `apps/web/src/components/layout/Topbar.svelte`

  Descripción: Barra superior del dashboard con búsqueda (CommandPalette),
               notificaciones, y menú de usuario con avatar.
               Integra RBAC para mostrar/ocultar acciones.

  ADRs relacionados: 0022 (Frontend), 0006 (RBAC), 0008 (PASETO)
-->

<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Avatar, AvatarFallback, AvatarImage } from "$lib/components/ui/avatar/index.js";
	import { Search, Bell, User as UserIcon, Settings, LogOut, Command, Menu } from "lucide-svelte";
	import { authStore } from "$lib/stores/auth.svelte";
	import { logout } from "$lib/api/auth";
	import type { User } from "$lib/types/user";
	import LanguageSelector from "./LanguageSelector.svelte";
	import ThemeToggle from "$lib/components/ui/theme-toggle/theme-toggle.svelte";
	import * as m from "$lib/paraglide/messages.js";

	let { onSearchClick }: { onSearchClick?: () => void } = $props();

	let user: User | null = $derived(authStore.user);

	// Dispatch event to open CommandPalette
	function handleSearchClick() {
		if (typeof document !== 'undefined') {
			document.dispatchEvent(new CustomEvent('open-command-palette'));
		}
	}

	function handleMobileMenuClick() {
		if (typeof document !== 'undefined') {
			document.dispatchEvent(new CustomEvent('sidebar-mobile-toggle'));
		}
	}

	async function handleLogout() {
		try {
			await logout();
		} catch {
			authStore.clearAuth();
		}
		window.location.href = '/login';
	}

	function getInitials(name: string | null | undefined): string {
		if (!name) return 'U';
		return name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2);
	}
</script>

<header class="sticky top-0 z-30 h-16 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
	<div class="flex h-full items-center justify-between px-6">
		<!-- Left: Search trigger -->
		<div class="flex items-center gap-4 flex-1">
			<Button variant="ghost" size="icon" class="lg:hidden" onclick={handleMobileMenuClick}>
				<Menu class="h-5 w-5" />
			</Button>
			<button
				onclick={handleSearchClick}
				class="flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors group"
			>
				<Search class="h-4 w-4" />
				<span class="text-sm hidden sm:inline">Search...</span>
				<kbd class="hidden md:inline-flex h-5 items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 group-hover:opacity-70">
					<Command class="h-3 w-3" />
					<span>K</span>
				</kbd>
			</button>
		</div>

		<!-- Right: Actions -->
		<div class="flex items-center gap-4">
			<!-- Notifications -->
			<DropdownMenu.Root>
				<DropdownMenu.Trigger asButton={false}>
					<button
						variant="ghost"
						size="icon"
						class="relative inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-8 w-8"
					>
						<Bell class="h-5 w-5" />
						<span class="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-primary text-[10px] font-medium text-primary-foreground">
							3
						</span>
					</button>
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end" class="w-80">
					<DropdownMenu.Label>{m.notifications_title()}</DropdownMenu.Label>
					<DropdownMenu.Separator />
					<div class="max-h-64 overflow-y-auto">
						<DropdownMenu.Item class="flex flex-col items-start gap-1 py-3">
							<p class="text-sm font-medium">{m.notification_new_user()}</p>
							<p class="text-xs text-muted-foreground">john@example.com {m.notification_new_user_desc()}</p>
						</DropdownMenu.Item>
						<DropdownMenu.Item class="flex flex-col items-start gap-1 py-3">
							<p class="text-sm font-medium">{m.notification_audit_exported()}</p>
							<p class="text-xs text-muted-foreground">{m.notification_audit_exported_desc()}</p>
						</DropdownMenu.Item>
						<DropdownMenu.Item class="flex flex-col items-start gap-1 py-3">
							<p class="text-sm font-medium">{m.notification_system_update()}</p>
							<p class="text-xs text-muted-foreground">API v1.2.0 {m.notification_system_update_desc()}</p>
						</DropdownMenu.Item>
					</div>
					<DropdownMenu.Separator />
					<DropdownMenu.Item class="justify-center text-primary">
						{m.notifications_view_all()}
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>

			<!-- Theme Toggle -->
			<ThemeToggle />

			<!-- Language Selector -->
			<LanguageSelector />

			<!-- User Menu -->
			<DropdownMenu.Root>
				<DropdownMenu.Trigger asButton={false}>
					<button
						variant="ghost"
						class="relative inline-flex h-8 w-8 items-center justify-center rounded-full shrink-0 overflow-hidden border transition-colors hover:opacity-80 focus:outline-none"
					>
						<Avatar class="h-8 w-8">
							<AvatarImage src="" alt={user?.name ?? 'User'} />
							<AvatarFallback>{getInitials(user?.name)}</AvatarFallback>
						</Avatar>
					</button>
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end" class="w-56">
					<DropdownMenu.Label class="font-normal">
						<div class="flex flex-col space-y-1">
							<p class="text-sm font-medium leading-none">{user?.name ?? 'User'}</p>
							<p class="text-xs leading-none text-muted-foreground">{user?.email}</p>
							{#if user?.role}
								<span class="inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-semibold mt-1 w-fit
									{user.role === 'admin' ? 'bg-primary/10 text-primary border-primary/20' :
									 user.role === 'moderator' ? 'bg-amber-500/10 text-amber-600 border-amber-500/20' :
									 'bg-muted text-muted-foreground'}">
									{user.role}
								</span>
							{/if}
						</div>
					</DropdownMenu.Label>
					<DropdownMenu.Separator />
					<DropdownMenu.Item onclick={() => window.location.href = '/dashboard/settings'}>
						<UserIcon class="mr-2 h-4 w-4" />
						Profile
					</DropdownMenu.Item>
					<DropdownMenu.Item onclick={() => window.location.href = '/dashboard/settings'}>
						<Settings class="mr-2 h-4 w-4" />
						Settings
					</DropdownMenu.Item>
					<DropdownMenu.Separator />
					<DropdownMenu.Item onclick={handleLogout} class="text-destructive focus:text-destructive">
						<LogOut class="mr-2 h-4 w-4" />
						Log out
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		</div>
	</div>
</header>
