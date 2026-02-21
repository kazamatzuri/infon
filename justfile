# Infon Development Task Runner
# Use `just <recipe>` to run. Docker recipes are the default workflow;
# prefix with `local-` to run directly on the host.

# ---------- Docker Compose (default workflow) ----------

# Start all services in development mode
dev: up

# Start Docker Compose services (detached)
up:
    docker compose up --build -d

# Stop Docker Compose services
down:
    docker compose down

# Stop services and remove volumes (clean slate)
down-clean:
    docker compose down -v

# Rebuild containers from scratch (no cache)
rebuild:
    docker compose build --no-cache
    docker compose up -d

# View logs (follow mode)
logs:
    docker compose logs -f

# View backend logs only
logs-backend:
    docker compose logs -f backend

# View frontend logs only
logs-frontend:
    docker compose logs -f frontend

# Run backend tests inside the container
test:
    docker compose exec backend cargo test

# Run backend tests with output
test-verbose:
    docker compose exec backend cargo test -- --nocapture

# Validate original bot compatibility inside the container
validate-bots:
    docker compose exec backend cargo test -- original_bots --nocapture

# Run clippy checks inside the container
check:
    docker compose exec backend cargo clippy -- -D warnings

# Format backend code inside the container
fmt:
    docker compose exec backend cargo fmt

# Open a shell in the backend container
shell-backend:
    docker compose exec backend bash

# Open a shell in the frontend container
shell-frontend:
    docker compose exec frontend bash

# Run frontend lint inside the container
lint-frontend:
    docker compose exec frontend npm run lint

# Production build of frontend inside the container
build-frontend:
    docker compose exec frontend npm run build

# Production build of backend inside the container
build-backend:
    docker compose exec backend cargo build --release

# Build both services for production
build: build-backend build-frontend

# Show running containers and their status
status:
    docker compose ps

# ---------- Local development (no Docker) ----------

# Start both backend and frontend locally
local-dev: local-dev-backend local-dev-frontend

# Start backend server locally
local-dev-backend:
    cd backend && cargo run

# Start frontend dev server locally
local-dev-frontend:
    cd frontend && npm run dev

# Run backend tests locally
local-test-backend:
    cd backend && cargo test

# Run frontend tests locally
local-test-frontend:
    cd frontend && npm test

# Run all local tests
local-test: local-test-backend

# Build backend locally (release)
local-build-backend:
    cd backend && cargo build --release

# Build frontend locally (production)
local-build-frontend:
    cd frontend && npm run build

# Check code locally
local-check:
    cd backend && cargo clippy -- -D warnings

# Format code locally
local-fmt:
    cd backend && cargo fmt

# Validate bot compatibility locally
local-validate-bots:
    cd backend && cargo test -- original_bots --nocapture

# Install frontend dependencies locally
local-install-frontend:
    cd frontend && npm install
