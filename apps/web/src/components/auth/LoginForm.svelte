<!--
  Ubicación: `apps/web/src/components/auth/LoginForm.svelte`

  Descripción: Formulario de login con validación simple,
               integración con auth store y manejo de errores.

  ADRs relacionados: 0022 (Frontend), 0008 (PASETO), 0021 (OpenAPI)
-->

<svelte:options runes={true} />

<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { login } from "$lib/api/auth";
	import * as m from "$lib/paraglide/messages.js";

	// Estado del formulario
	let email = $state("");
	let password = $state("");
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	// Validación simple sin ArkType
	function isValidEmail(email: string): boolean {
		return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
	}

	// Computed validation
	let isValid = $derived(isValidEmail(email) && password.length >= 8);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		errorMessage = null;

		if (!isValidEmail(email)) {
			errorMessage = m.error_invalid_email();
			return;
		}

		if (password.length < 8) {
			errorMessage = m.error_password_length();
			return;
		}

		isSubmitting = true;
		try {
			await login({ email, password });
			// Redirect to dashboard on success
			window.location.href = "/dashboard";
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : m.login_failed();
		} finally {
			isSubmitting = false;
		}
	}
</script>

<Card.Root>
	<Card.Content class="pt-6">
		<form onsubmit={handleSubmit} class="space-y-4">
			<div class="space-y-2">
				<Label for="email">{m.login_email()}</Label>
				<Input
					id="email"
					type="email"
					placeholder={m.email_placeholder()}
					bind:value={email}
					disabled={isSubmitting}
				/>
			</div>

			<div class="space-y-2">
				<Label for="password">{m.login_password()}</Label>
				<Input
					id="password"
					type="password"
					placeholder={m.password_placeholder()}
					bind:value={password}
					disabled={isSubmitting}
				/>
			</div>

			{#if errorMessage}
				<div class="text-sm text-red-600 bg-red-50 p-2 rounded">
					{errorMessage}
				</div>
			{/if}

			<Button type="submit" class="w-full" disabled={isSubmitting || !isValid}>
				{isSubmitting ? m.login_signing_in() : m.login_submit()}
			</Button>
		</form>
	</Card.Content>
</Card.Root>
