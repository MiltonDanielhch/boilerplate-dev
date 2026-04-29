<!--
  Ubicación: `apps/web/src/components/auth/RegisterForm.svelte`

  Descripción: Formulario de registro con validación ArkType,
               integración con auth store y manejo de errores.

  ADRs relacionados: 0022 (Frontend), 0008 (PASETO), 0021 (OpenAPI)
-->

<script lang="ts">
	import { type } from "arktype";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { register } from "$lib/api/auth";
	import * as m from "$lib/paraglide/messages.js";

	// Schema de validación inline (ArkType)
	const RegisterSchema = type({
		email: "string.email",
		password: "string >= 8",
		name: "string?",
		confirmPassword: "string"
	}).narrow((data, problems) => {
		if (data.password !== data.confirmPassword) {
			problems.throw(m.error_passwords_not_match());
		}
		return true;
	});

	// Estado del formulario
	let email = $state("");
	let password = $state("");
	let confirmPassword = $state("");
	let name = $state("");
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let successMessage = $state<string | null>(null);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		errorMessage = null;
		successMessage = null;

		const result = RegisterSchema({ email, password, name, confirmPassword });
		if (result.errors) {
			errorMessage = result.errors.summary ?? m.error_check_inputs();
			return;
		}

		isSubmitting = true;
		try {
			await register({ email, password, name: name || undefined });
			successMessage = m.register_success();
			// Redirect to dashboard after short delay
			setTimeout(() => {
				window.location.href = "/dashboard";
			}, 1500);
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : m.error_generic();
		} finally {
			isSubmitting = false;
		}
	}
</script>

<Card.Root>
	<Card.Content class="pt-6">
		<form onsubmit={handleSubmit} class="space-y-4">
			<div class="space-y-2">
				<Label for="name">{m.register_name_optional()}</Label>
				<Input
					id="name"
					type="text"
					placeholder={m.register_name_placeholder()}
					bind:value={name}
					disabled={isSubmitting}
				/>
			</div>

			<div class="space-y-2">
				<Label for="email">{m.register_email()}</Label>
				<Input
					id="email"
					type="email"
					placeholder={m.email_placeholder()}
					bind:value={email}
					disabled={isSubmitting}
					required
				/>
			</div>

			<div class="space-y-2">
				<Label for="password">{m.register_password()}</Label>
				<Input
					id="password"
					type="password"
					placeholder={m.password_placeholder()}
					bind:value={password}
					disabled={isSubmitting}
					required
				/>
			</div>

			<div class="space-y-2">
				<Label for="confirmPassword">{m.register_confirm_password()}</Label>
				<Input
					id="confirmPassword"
					type="password"
					placeholder={m.password_placeholder()}
					bind:value={confirmPassword}
					disabled={isSubmitting}
					required
				/>
			</div>

			{#if errorMessage}
				<div class="text-sm text-red-600 bg-red-50 p-2 rounded">
					{errorMessage}
				</div>
			{/if}

			{#if successMessage}
				<div class="text-sm text-green-600 bg-green-50 p-2 rounded">
					{successMessage}
				</div>
			{/if}

			<Button type="submit" class="w-full" disabled={isSubmitting}>
				{isSubmitting ? m.register_creating() : m.register_submit()}
			</Button>
		</form>
	</Card.Content>
</Card.Root>
