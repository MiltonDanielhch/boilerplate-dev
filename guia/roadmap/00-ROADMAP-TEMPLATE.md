# Roadmap — Template: [NOMBRE DEL MÓDULO]

> **Objetivo:** [Describir brevemente qué se va a implementar en este módulo]
>
> **Stack:** [Tecnologías principales separadas por ·]
>
> **ADRs:** [Listar ADRs relevantes, ej: ADR 0001, ADR 0003]
>
> **Criterio de activación:** [Cuándo implementar este módulo — condiciones concretas]
>
> **Prerrequisitos:** [Qué otros roadmaps deben completarse antes — ej: Auth Fullstack]
>
> **Relacionado con:** [Otros roadmaps que se tocan — ej: Backend, Frontend]

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso General

| Bloque | Nombre | Estado | Progreso |
|--------|--------|--------|----------|
| A.1 | [Nombre del bloque] | [ ] | 0% |
| A.2 | [Nombre del bloque] | [ ] | 0% |
| A.3 | [Nombre del bloque] | [ ] | 0% |

---

## A.1 — [Nombre del Bloque: Foundation / Setup]

> **ADRs:** [Listar ADRs relevantes]
> **Output:** [Qué se entrega al finalizar]
> **Tiempo estimado:** [X días]

```
[ ] Tarea 1: [Descripción específica]
    └─ Ref: [Documento de referencia si aplica]

[ ] Tarea 2: [Descripción específica]
    └─ Verificación: [Cómo saber que funciona]

[ ] Tarea 3: [Descripción específica]
```

---

## A.2 — [Nombre del Bloque: Core Feature]

> **ADRs:** [Listar ADRs relevantes]
> **Output:** [Qué se entrega al finalizar]
> **Tiempo estimado:** [X días]

```
[ ] Backend:
    [ ] [Tarea específica de backend]
        └─ Ref: [ADR o doc]
    [ ] [Otra tarea de backend]

[ ] Frontend:
    [ ] [Tarea específica de frontend]
        └─ Ref: [ADR o doc]
    [ ] [Otra tarea de frontend]

[ ] Integración:
    [ ] [Tarea de integración front-back]
```

---

## A.3 — [Nombre del Bloque: Advanced / Polish]

> **ADRs:** [Listar ADRs relevantes]
> **Output:** [Qué se entrega al finalizar]
> **Tiempo estimado:** [X días]

```
[ ] Feature avanzada 1:
    [ ] [Subtarea]
    [ ] [Subtarea]

[ ] Testing:
    [ ] Unit tests
    [ ] Integration tests
    [ ] E2E tests (si aplica)

[ ] Documentación:
    [ ] Actualizar API docs
    [ ] Actualizar este roadmap (marcar completado)
```

---

## Verificación Final

### Backend

```bash
# Test de endpoints
curl -X POST http://localhost:8080/api/v1/[endpoint] \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"key": "value"}'
# Esperado: 200 OK con body específico

# Verificar en DB
sqlite3 data/database.sqlite "SELECT * FROM [tabla] WHERE [condicion];"
# Esperado: [descripción de resultado esperado]
```

### Frontend

```bash
# Build sin errores
cd apps/web && pnpm build
# Esperado: build completo sin errores

# Tests pasan
cd apps/web && pnpm test
# Esperado: all tests passed

# Types correctos
cd apps/web && pnpm check
# Esperado: 0 errors
```

### Arquitectura

- [ ] Hexagonal: [Verificar regla específica]
- [ ] No hay código duplicado
- [ ] Tests cubren casos edge
- [ ] Documentación actualizada

---

## Troubleshooting — [Nombre del Módulo]

### Error: [Error común 1]

**Síntoma:** [Descripción]
**Causa:** [Por qué pasa]
**Solución:**
```bash
[Comando o pasos para arreglar]
```

### Error: [Error común 2]

**Síntoma:** [Descripción]
**Causa:** [Por qué pasa]
**Solución:**
```bash
[Comando o pasos para arreglar]
```

---

## Notas para el Futuro

- [Nota sobre decisiones técnicas tomadas]
- [Links a issues o PRs relevantes]
- [Decisiones pendientes de revisar]

---

## Checklist de "Listo para Producción"

- [ ] Todas las tareas marcadas [x]
- [ ] Verificaciones pasan
- [ ] Documentación actualizada en ADRs/docs
- [ ] README actualizado si aplica
- [ ] CI/CD pasa (si hay pipelines específicos)
- [ ] Monitoreo configurado (logs, alerts)

---

**Creado:** [YYYY-MM-DD]
**Última actualización:** [YYYY-MM-DD]
**Responsable:** [Rol o persona]

---

## Guía de Uso de este Template

1. **Copiar archivo:** `cp 00-ROADMAP-TEMPLATE.md XX-ROADMAP-NOMBRE.md`
2. **Reemplazar campos:** Todo entre `[corchetes]` debe personalizarse
3. **Ajustar bloques:** Añadir/eliminar bloques según complejidad del módulo
4. **Referenciar en MASTER:** Añadir entrada en `01-ROADMAP-MASTER.md`
5. **Enlazar ADRs:** Crear ADR si es decisión arquitectónica nueva

**Convenciones:**
- Usar código de bloque con `[ ]` para tareas checkables
- Referenciar líneas específicas de otros docs cuando sea relevante
- Incluir verificaciones concretas (curl, SQL, etc.)
- Mantener estados actualizados: `[ ]` pendiente → `[~]` progreso → `[x]` listo
