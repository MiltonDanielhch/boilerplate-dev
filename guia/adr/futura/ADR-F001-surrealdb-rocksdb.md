# ADR F-001 — Futuro: SurrealDB + RocksDB (Multi-modelo)

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — no implementar hasta Nivel 3 de escalamiento |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite WAL — persistencia actual), ADR 0031 (Escalamiento Nivel 2) |

---

## Contexto

Este ADR documenta una opción de persistencia futura para cuando el proyecto necesite
capacidades multi-modelo (grafos, documentos, time-series) que SQLite no ofrece nativamente.

**No implementar hasta que:**
- SQLite WAL y Turso (ADR 0031 Nivel 2a) ya no sean suficientes
- Se necesiten consultas de grafo reales (relaciones N:N complejas con traversal)
- El volumen de datos time-series justifique un motor especializado

---

## Decisión futura

Evaluar **SurrealDB** como base de datos multi-modelo con **RocksDB** como motor de almacenamiento.

### Cuándo activar

| Criterio | Umbral |
|----------|--------|
| Consultas de grafo | Traversal de >5 niveles de relaciones frecuentes |
| Time-series | >1M eventos/día que requieren agregaciones temporales |
| Documentos JSON | Esquemas variables que no encajan en SQL relacional |
| Multitenancy | >1000 tenants con esquemas completamente diferentes |

### Ventajas potenciales

- Un solo motor para SQL, grafos, documentos y time-series
- SurrealQL es compatible con SQL — curva de aprendizaje menor
- RocksDB como backend — rendimiento de escritura muy alto
- Schema flexible — útil para módulos con estructuras de datos variables

### Riesgos

- SurrealDB es relativamente nuevo — ecosistema en maduración
- SQLx no tiene soporte nativo — requeriría adaptador nuevo en `crates/database`
- La migración de datos desde SQLite requiere planificación cuidadosa

### Estrategia de migración

```rust
// La arquitectura hexagonal del ADR 0001 garantiza que:
// 1. Los repositorios en crates/database se reescriben para SurrealDB
// 2. El dominio (crates/domain) no cambia ninguna línea
// 3. Los casos de uso (crates/application) no cambian ninguna línea
// 4. Los tests de dominio y aplicación siguen pasando sin modificación
```

---

## Estado actual

No implementar. Mantener SQLite WAL (ADR 0004) hasta que los criterios de activación
sean evidentes en datos de producción reales.
