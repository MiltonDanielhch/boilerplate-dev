<!--
  Ubicación: `apps/web/src/components/layout/CommandPalette.svelte`

  Descripción: Paleta de comandos tipo Spotlight (Ctrl+K) para navegación rápida.
               Filtra acciones según permisos RBAC del usuario.
               Soporta navegación por teclado (arrows, Enter, Escape).

  ADRs relacionados: 0022 (Frontend), 0006 (RBAC)
-->

<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import * as Command from "$lib/components/ui/command/index.js";
	import { authStore } from "$lib/stores/auth.svelte";
	import * as m from "$lib/paraglide/messages.js";
	import {
		LayoutDashboard,
		Users,
		Shield,
		ClipboardList,
		Settings,
		LogOut,
		Search,
		FileText,
		HelpCircle,
		Moon,
		Sun
	} from "lucide-svelte";

	let { open = $bindable(false) }: { open?: boolean } = $props();

	interface CommandItem {
		id: string;
		label: string;
		href?: string;
		icon: typeof LayoutDashboard;
		permission?: string;
		action?: () => void;
		shortcut?: string;
		group: string;
	}

	// Todos los comandos disponibles con sus permisos
	const allCommands: CommandItem[] = [
		// Navegación
		{ id: 'dashboard', label: m.sidebar_dashboard(), href: '/dashboard', icon: LayoutDashboard, group: 'Navigation' },
		{ id: 'users', label: m.sidebar_users(), href: '/dashboard/users', icon: Users, permission: 'users:read', group: 'Navigation' },
		{ id: 'roles', label: m.sidebar_roles(), href: '/dashboard/roles', icon: Shield, permission: 'roles:read', group: 'Navigation' },
		{ id: 'audit', label: m.audit_title(), href: '/dashboard/audit', icon: ClipboardList, permission: 'audit:read', group: 'Navigation' },
		{ id: 'settings', label: m.sidebar_settings(), href: '/dashboard/settings', icon: Settings, group: 'Navigation' },

		// Acciones rápidas (requieren permisos)
		{ id: 'create-user', label: m.action_add() + ' ' + m.sidebar_users(), href: '/dashboard/users?action=create', icon: Users, permission: 'users:write', group: 'Actions' },

		// Sistema
		{ id: 'docs', label: 'Documentation', href: '/docs', icon: FileText, group: 'System' },
		{ id: 'help', label: 'Help & Support', href: '/help', icon: HelpCircle, group: 'System' },
	];

	// Filtrar comandos según permisos
	const availableCommands = $derived(
		allCommands.filter(cmd => {
			if (!cmd.permission) return true;
			return authStore.hasPermission(cmd.permission);
		})
	);

	// Agrupar comandos
	const groupedCommands = $derived(() => {
		const groups: Record<string, CommandItem[]> = {};
		availableCommands.forEach(cmd => {
			if (!groups[cmd.group]) groups[cmd.group] = [];
			groups[cmd.group].push(cmd);
		});
		return groups;
	});

	function handleSelect(cmd: CommandItem) {
		open = false;
		if (cmd.action) {
			cmd.action();
		} else if (cmd.href) {
			window.location.href = cmd.href;
		}
	}

	// Keyboard shortcut Ctrl+K / Cmd+K
	$effect(() => {
		const handleKeydown = (e: KeyboardEvent) => {
			if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
				e.preventDefault();
				open = true;
			}
		};
		document.addEventListener('keydown', handleKeydown);
		return () => document.removeEventListener('keydown', handleKeydown);
	});

	// Listen for open-command-palette event from Topbar
	$effect(() => {
		const handleOpen = () => {
			open = true;
		};
		document.addEventListener('open-command-palette', handleOpen);
		return () => document.removeEventListener('open-command-palette', handleOpen);
	});
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="gap-0 p-0 overflow-hidden max-w-2xl">
		<Command.Root class="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-muted-foreground">
			<Command.Input
				placeholder={m.command_palette_placeholder()}
				class="border-0 border-b px-4 h-12 focus:ring-0"
			/>
			<Command.List class="max-h-[300px] overflow-y-auto p-2">
				<Command.Empty class="py-6 text-center text-sm text-muted-foreground">
					{m.no_results()}
				</Command.Empty>

				{#each Object.entries(groupedCommands()) as [groupName, items]}
					<Command.Group heading={groupName}>
						{#each items as cmd}
							<Command.Item
								value={cmd.id}
								onSelect={() => handleSelect(cmd)}
								class="flex items-center gap-2 px-2 py-2 cursor-pointer"
							>
								<cmd.icon class="h-4 w-4 text-muted-foreground" />
								<span>{cmd.label}</span>
								{#if cmd.shortcut}
									<kbd class="ml-auto inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium">
										{cmd.shortcut}
									</kbd>
								{/if}
							</Command.Item>
						{/each}
					</Command.Group>
					<Command.Separator class="my-1" />
				{/each}
			</Command.List>
			<div class="border-t bg-muted/50 px-4 py-2 text-xs text-muted-foreground flex items-center justify-between">
				<div class="flex items-center gap-4">
					<span class="flex items-center gap-1">
						<kbd class="inline-flex h-5 items-center rounded border bg-background px-1.5 font-mono text-[10px] font-medium">↑</kbd>
						<kbd class="inline-flex h-5 items-center rounded border bg-background px-1.5 font-mono text-[10px] font-medium">↓</kbd>
						to navigate
					</span>
					<span class="flex items-center gap-1">
						<kbd class="inline-flex h-5 items-center rounded border bg-background px-1.5 font-mono text-[10px] font-medium">↵</kbd>
						to select
					</span>
				</div>
				<span class="flex items-center gap-1">
					<kbd class="inline-flex h-5 items-center rounded border bg-background px-1.5 font-mono text-[10px] font-medium">Esc</kbd>
					to close
				</span>
			</div>
		</Command.Root>
	</Dialog.Content>
</Dialog.Root>
