# Ubicación: `apps/web/Dockerfile`
#
# Descripción: Dockerfile multi-stage para Astro SSR.
#              Build: compila el sitio
#              Runtime: sirve con node adapter
#
# ADRs relacionados: 0014 (Deploy)

# ============================================
# Stage 1: Builder
# ============================================
FROM node:22-alpine AS builder

WORKDIR /app

# Copiar package files
COPY package.json pnpm-lock.yaml ./

# Instalar dependencias
RUN corepack enable && \
    pnpm install --frozen-lockfile

# Copiar código fuente
COPY . .

# Generar mensajes Paraglide (i18n)
RUN pnpm paraglide-js compile --project ./project.inlang --outdir ./src/paraglide || true

# Build de producción
RUN pnpm build

# ============================================
# Stage 2: Runtime
# ============================================
FROM node:22-alpine AS runtime

WORKDIR /app

# Copiar solo archivos necesarios
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./package.json

# Instalar runtime solo
RUN corepack enable && \
    pnpm install --frozen-lockfile --prod

# Variable de entorno
ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=4321

EXPOSE 4321

# Serve con adapter standalone
CMD ["node", "./dist/server/entry.mjs"]