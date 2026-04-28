<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { createUser, updateUser } from "$lib/api/users";
	import type { User, UserCreateInput, UserUpdateInput } from "$lib/types/user";
	import { type } from "arktype";

	interface Props {
		user?: User | null;
		onSuccess?: () => void;
		showTrigger?: boolean;
	}

	let { user = null, onSuccess, showTrigger = true }: Props = $props();

	let open = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);

	let name = $state(user?.name ?? "");
	let email = $state(user?.email ?? "");
	let password = $state("");

	const isEdit = $derived(user !== null);

	const UserSchema = type({
		email: "string.email",
		password: isEdit ? "string?" : "string >= 8",
		name: "string?"
	});

	let validation = $derived(UserSchema({ email, password, name }));
	let isValid = $derived(!validation.errors);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = null;
		loading = true;

		const result = UserSchema({ email, password, name });
		if (result.errors) {
			error = "Por favor corrige los errores";
			loading = false;
			return;
		}

		try {
			if (isEdit && user) {
				const data: UserUpdateInput = { name: name || undefined };
				await updateUser(user.id, data);
			} else {
				const data: UserCreateInput = {
					email,
					password,
					name: name || undefined
				};
				await createUser(data);
			}
			open = false;
			resetForm();
			onSuccess?.();
		} catch (err) {
			error = err instanceof Error ? err.message : "Error al guardar";
		} finally {
			loading = false;
		}
	}

	function resetForm() {
		name = "";
		email = "";
		password = "";
		error = null;
	}

	function handleOpenChange(isOpen: boolean) {
		open = isOpen;
		if (!isOpen) {
			resetForm();
		}
	}

	export function trigger() {
		open = true;
	}
</script>

<Dialog.Root bind:open onOpenChange={handleOpenChange}>
	{#if showTrigger}
		<Dialog.Trigger asChild>
			<Button>Crear Usuario</Button>
		</Dialog.Trigger>
	{/if}
	<Dialog.Content class="sm:max-w-[425px]">
		<Dialog.Header>
			<Dialog.Title>{isEdit ? "Editar Usuario" : "Crear Usuario"}</Dialog.Title>
			<Dialog.Description>
				{isEdit
					? "Actualiza la información del usuario."
					: "Completa los datos para crear un nuevo usuario."}
			</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={handleSubmit} class="grid gap-4 py-4">
			<div class="grid gap-2">
				<Label for="email">Email</Label>
				<Input
					id="email"
					type="email"
					placeholder="usuario@ejemplo.com"
					bind:value={email}
					disabled={loading}
					required
				/>
			</div>
			<div class="grid gap-2">
				<Label for="name">Nombre</Label>
				<Input
					id="name"
					type="text"
					placeholder="Nombre completo"
					bind:value={name}
					disabled={loading}
				/>
			</div>
			{#if !isEdit}
				<div class="grid gap-2">
					<Label for="password">Password</Label>
					<Input
						id="password"
						type="password"
						placeholder="Mínimo 8 caracteres"
						bind:value={password}
						disabled={loading}
						required
					/>
				</div>
			{/if}
			{#if error}
				<div class="text-sm text-red-500">{error}</div>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (open = false)} disabled={loading}>
					Cancelar
				</Button>
				<Button type="submit" disabled={loading || !isValid}>
					{loading ? "Guardando..." : isEdit ? "Actualizar" : "Crear"}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>