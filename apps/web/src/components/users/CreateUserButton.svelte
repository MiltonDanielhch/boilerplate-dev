<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import { Plus } from "lucide-svelte";
	import UserForm from "./UserForm.svelte";
	import * as m from "$lib/paraglide/messages.js";

	let open = $state(false);
</script>

<Button onclick={() => open = true}>
	<Plus class="h-4 w-4 mr-2" />
	{m.users_add()}
</Button>

<UserForm bind:open showTrigger={false} />
