<!--
  Ubicación: `apps/web/src/components/layout/Sidebar.svelte`

  Descripción: Componente de navegación lateral colapsable usando Svelte 5 Runes.
               Incluye menú de navegación con iconos de lucide-svelte,
               estado colapsable con $state y RBAC para filtrar items
               según permisos del usuario.

  ADRs relacionados: 0022 (Frontend), 0023 (i18n), 0006 (RBAC)
-->

<script lang="ts">
	import { cn } from "$lib/utils.js";
	import Button from "$lib/components/ui/button/button.svelte";
	import { ChevronLeft, ChevronRight, LayoutDashboard, Users, Settings, LogOut, Shield, ClipboardList } from "lucide-svelte";
	import { authStore } from "$lib/stores/auth.svelte";
	import { logout } from "$lib/api/auth";
	import { get } from "svelte/store";
	import * as m from "$lib/paraglide/messages.js";
	import { isTauri } from "$lib/tauri";

	let { class: className }: { class?: string } = $props();

	let collapsed = $state(false);

	// Cargar estado del sidebar desde localStorage
	$effect(() => {
		if (typeof localStorage !== 'undefined') {
			const saved = localStorage.getItem('sidebar_collapsed');
			if (saved !== null) {
				collapsed = saved === 'true';
			}
		}
	});

	// Persistir cambios en localStorage
	$effect(() => {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('sidebar_collapsed', String(collapsed));
		}
	});

	interface MenuItem {
		icon: typeof LayoutDashboard;
		label: string;
		href: string;
		permission?: string;
	}

	const allMenuItems: MenuItem[] = [
		{ icon: LayoutDashboard, label: m.sidebar_dashboard(), href: "/dashboard" },
		{ icon: Users, label: m.sidebar_users(), href: "/dashboard/users", permission: "users:read" },
		{ icon: ClipboardList, label: m.sidebar_audit(), href: "/dashboard/audit", permission: "audit:read" },
		{ icon: Shield, label: m.sidebar_roles(), href: "/dashboard/roles", permission: "roles:read" },
		{ icon: Settings, label: m.sidebar_settings(), href: "/dashboard/settings" },
	];

	// Filtrar items según permisos del usuario
	const menuItems = $derived(
		allMenuItems.filter(item => {
			if (!item.permission) return true;
			// Si auth no está inicializado, mostrar todos los items con permisos
			const user = get(authStore.userStore);
			if (!user) return true;
			return authStore.hasPermission(item.permission);
		})
	);
	
	function toggleSidebar() {
		collapsed = !collapsed;
		dispatchSidebarToggle(collapsed);
	}
	
	function dispatchSidebarToggle(isCollapsed: boolean) {
		if (typeof document !== 'undefined') {
			document.dispatchEvent(new CustomEvent('sidebar-toggle', {
				detail: { collapsed: isCollapsed }
			}));
		}
	}

	async function handleLogout() {
		try {
			await logout();
			window.location.href = '/login';
		} catch {
			// Si falla el logout en API, igual limpiamos local
			authStore.clearAuth();
			window.location.href = '/login';
		}
	}
</script>

<aside
	class={cn(
		"fixed left-0 top-0 z-40 h-screen border-r bg-sidebar transition-all duration-300",
		collapsed ? "w-16" : "w-64",
		className
	)}
>
	<div class="flex h-full flex-col">
		<!-- Header -->
		<div class="flex h-16 items-center justify-between border-b px-4">
			{#if !collapsed}
				<div class="flex flex-col">
					<span class="font-semibold text-sidebar-foreground">🚀 Boilerplate</span>
					{#if isTauri()}
						<span class="text-[10px] font-bold uppercase tracking-wider text-primary">Desktop Mode</span>
					{/if}
				</div>
			{/if}
			<Button variant="ghost" size="icon" onclick={toggleSidebar} class="ml-auto">
				{#if collapsed}
					<ChevronRight class="h-4 w-4" />
				{:else}
					<ChevronLeft class="h-4 w-4" />
				{/if}
			</Button>
		</div>
		
		<!-- Navigation -->
		<nav class="flex-1 space-y-1 p-2">
			{#each menuItems as item}
				{#if collapsed}
					<a
						href={item.href}
						class="flex h-10 w-full items-center justify-center rounded-md hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
						title={item.label}
					>
						<item.icon class="h-5 w-5" />
					</a>
				{:else}
					<a
						href={item.href}
						class="flex h-9 w-full items-center gap-3 rounded-md px-3 py-2 text-sm hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
					>
						<item.icon class="h-5 w-5" />
						{item.label}
					</a>
				{/if}
			{/each}
		</nav>
		
		<!-- Footer -->
		<div class="border-t p-2">
			{#if collapsed}
				<button
					class="flex h-10 w-full items-center justify-center rounded-md hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
					title={m.logout_button()}
					onclick={handleLogout}
				>
					<LogOut class="h-5 w-5" />
				</button>
			{:else}
				<Button variant="ghost" class="w-full justify-start gap-3" onclick={handleLogout}>
					<LogOut class="h-5 w-5" />
					{m.logout_button()}
				</Button>
			{/if}
		</div>
	</div>
</aside>
