<!-- Formulario simple de prueba sin validación -->
<script>
  let email = "";
  let password = "";
  
  function handleSubmit(e) {
    e.preventDefault();
    console.log("Submit:", { email, password });
  }
</script>

<div class="space-y-4">
  <div>
    <label for="email">Email</label>
    <input type="email" id="email" bind:value={email} class="border p-2 w-full" />
  </div>
  
  <div>
    <label for="password">Password</label>
    <input type="password" id="password" bind:value={password} class="border p-2 w-full" />
  </div>
  
  <button type="button" onclick={handleSubmit} class="bg-blue-500 text-white p-2 w-full">
    Sign in
  </button>
</div>
