---
name: deal-with-vite
description: Troubleshoot Vite + Docker volume mount issues. Use when frontend changes aren't reflected, HMR fails, file watching breaks, or stale code is served in Docker.
argument-hint: [issue description]
---

# Vite + Docker Volume Troubleshooting

When diagnosing Vite issues in our Docker dev environment, follow this systematic approach.

## Project Context

- Frontend: React + TypeScript + Vite in `frontend/`
- Docker: `frontend/Dockerfile.dev` based on `node:22-bookworm-slim`
- Volume mounts: `frontend/src`, `frontend/public`, config files mapped into container
- Vite dev server runs inside container on port 5173
- Vite proxies `/api` and `/ws` to backend service on port 3000

## Issue 1: Stale Code Served After File Changes

**Symptoms**: You edit a `.tsx` file on the host, but the browser still runs the old code. Hard refresh (Ctrl+Shift+R) doesn't help. The file IS correct inside the container (`docker compose exec frontend cat /app/frontend/src/...`).

**Root cause**: Docker volume mounts on Windows don't reliably propagate filesystem events (inotify) into the container. Vite never detects the change and serves its in-memory cached version.

**Quick fix**:
```bash
docker compose restart frontend
# Then hard refresh browser (Ctrl+Shift+R)
```

**Permanent fix** - enable polling in `vite.config.ts`:
```ts
server: {
  watch: {
    usePolling: true,
    interval: 1000,
  }
}
```

**Verification**: Fetch the raw source from Vite to confirm what's actually being served:
```js
// In browser console:
fetch('/src/components/YourFile.tsx').then(r => r.text()).then(t => console.log(t.includes('your_new_code')))
```

## Issue 2: HMR WebSocket Connection Fails

**Symptoms**: Vite loads but shows "connecting..." or HMR updates don't reach the browser.

**Fix** - ensure HMR config allows external connections:
```ts
server: {
  host: '0.0.0.0',  // Already set via CMD --host flag
  hmr: {
    host: 'localhost',  // Browser connects to this
    port: 5173,
  }
}
```

## Issue 3: Proxy Errors (ECONNREFUSED to backend)

**Symptoms**: `[vite] http proxy error: /api/...` in frontend logs.

**Cause**: Frontend container started before backend finished compiling. This is normal on first boot.

**Fix**: Wait for backend to finish compiling (`docker compose logs -f backend`), then refresh. If persistent, check that the proxy target uses the Docker service name:
```ts
proxy: {
  '/api': { target: process.env.VITE_API_URL || 'http://localhost:3000' }
}
```
In Docker, `VITE_API_URL=http://backend:3000` is set via docker-compose.yml.

## Issue 4: node_modules Missing or Stale

**Symptoms**: Module not found errors in container.

**Fix**: We intentionally do NOT volume-mount `node_modules`. They're installed during `docker build`. If `package.json` changes, rebuild:
```bash
docker compose build frontend
docker compose up -d
```

## Diagnostic Checklist

1. Check container logs: `docker compose logs -f frontend`
2. Verify file in container: `docker compose exec frontend cat /app/frontend/src/<file>`
3. Check what Vite serves: fetch raw source in browser console
4. Restart frontend if stale: `docker compose restart frontend`
5. Full rebuild if deps changed: `docker compose build frontend && docker compose up -d`
