# ADR 0011 — Estándares de Desarrollo: Ciclo Lab→Puente→Producción

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0010 (Testing), ADR 0012 (just + lefthook) |

---

## Contexto

El desarrollo suele volverse lento por acumulación de deuda técnica y procesos manuales.
Con un presupuesto de $5/mes y un equipo de una persona (o una persona + IA), no podemos
permitirnos código ineficiente ni módulos que crecen sin control.

---

## Decisión

Adoptar **Minimalismo Técnico** y el **Ciclo Lab→Puente→Producción** como ley de desarrollo.

### 1 — Restricciones de atomicidad

| Unidad | Límite | Acción si se supera |
|--------|--------|---------------------|
| Función | 30 líneas de lógica real | Subdividir con nombres descriptivos |
| Archivo | 200 líneas | Dividir en sub-módulos con responsabilidad única |
| Crate | Solo sus dependencias declaradas | El Cargo.toml hace cumplir esto |

**Regla del Boy Scout:** cada commit deja el código un poco más limpio de como lo encontró.

### 2 — El ciclo de vida

```
Laboratorio (local)
  → Experimentación pura
  → TDD activo — tests de dominio primero (ADR 0010)
  → Validación de contratos (/docs en Scalar, buf generate)
  → cargo check --workspace antes de cada commit
      ↓
Puente (CI / staging)
  → Build externo — nunca compilar en producción (ADR 0013)
  → cargo nextest run --all-targets
  → cargo clippy --all-targets -D warnings
  → cargo deny check + cargo audit
  → just types-check — verifica que api.ts no tiene diff
  → just prepare — verifica que .sqlx/ está actualizado
      ↓
Producción
  → Deploy atómico con Kamal (ADR 0014)
  → Zero-downtime por healthcheck en /health
  → Ping a Healthchecks.io (ADR 0015)
  → Rollback en 5 segundos si algo falla
```

### 3 — Código autodocumentado

```rust
// ❌ Comentario que explica el cómo — el código ya lo dice
let activos = usuarios.iter().filter(|u| u.deleted_at.is_none());

// ✅ Comentario que explica el por qué — la decisión de negocio
// Los usuarios borrados conservan sus audit_logs (ADR 0006)
// por eso usamos Soft Delete en lugar de DELETE real
let activos = usuarios.iter().filter(|u| u.is_active());
```

Preferir **tipos fuertes** sobre comentarios explicativos:

```rust
// ❌ String genérico que requiere comentario
fn crear_usuario(id: String, tipo: String) { ... }

// ✅ Tipos que documentan solos
fn crear_usuario(id: UserId, tipo: TipoUsuario) { ... }
```

### 4 — Comparativa SDLC

| Métrica | Tradicional | Este proyecto |
|---------|-------------|---------------|
| **Feedback loop** | Días / semanas | Minutos — TDD + cargo-watch |
| **Costo de cambio** | Sube exponencialmente | Constante — arquitectura hexagonal |
| **Documentación** | Manual y obsoleta | ADRs + código autodocumentado |
| **Deploy** | Evento estresante | `just deploy` — invisible |
| **Violaciones de arch** | Code review | `cargo check` + `sintonia check arch` |

---
## Herramientas y Librerías para Optimizar (Edición 2026)

Para automatizar el cumplimiento de estos estándares y mejorar la experiencia de desarrollo:

| Herramienta | Propósito en los Estándares |
| :--- | :--- |
| **`bacon`** | **Feedback Loop:** Corrección de errores y tests en tiempo real con una UI minimalista y eficiente. |
| **`typos`** | **Calidad de Texto:** Corrector ortográfico ultra-rápido para código y documentación (evita confusión en IAs). |
| **`cargo-public-api`** | **Control de Contratos:** Garantiza que los cambios en crates de dominio no rompan la compatibilidad accidentalmente. |
| **`clippy` (Pedantic)** | **Linter Avanzado:** Uso de lints `pedantic` y `nursery` para mantener el código en nivel "World-Class". |

---

## Consecuencias

### ✅ Positivas

- Pasar de idea a producción en horas, no semanas
- Las restricciones de tamaño obligan a pensar de forma simple
- Un agente IA puede entender y contribuir al código sin contexto adicional

### ⚠️ Negativas / Trade-offs

- Requiere disciplina para borrar código que "podría servir luego"
  → El límite de 200 líneas por archivo actúa como recordatorio físico
  → `cargo clippy` detecta código muerto — eliminar es fácil con la señal clara
- Muchos archivos pequeños pueden ser difíciles de navegar
  → La estructura de `crates/` del ADR 0001 organiza los archivos por responsabilidad
  → La Command Palette del editor (Ctrl+P) resuelve la navegación

### Decisiones derivadas

- lefthook pre-commit: `cargo fmt --check` — formato consistente (ADR 0012)
- lefthook pre-push: `cargo clippy -D warnings` — sin warnings en main (ADR 0012)
- `just deploy` incluye `just audit` y `just test` — imposible deployar sin pasar calidad
- Los comentarios de "por qué" se prefieren — no los de "cómo"
