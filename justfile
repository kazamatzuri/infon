# Infon Development Task Runner

# Start everything for development
dev: dev-backend dev-frontend

# Start backend server
dev-backend:
    cd backend && cargo run

# Start frontend dev server
dev-frontend:
    cd frontend && npm run dev

# Run backend tests
test-backend:
    cd backend && cargo test

# Run frontend tests
test-frontend:
    cd frontend && npm test

# Run all tests
test: test-backend

# Build backend
build-backend:
    cd backend && cargo build --release

# Check code
check:
    cd backend && cargo clippy -- -D warnings

# Format code
fmt:
    cd backend && cargo fmt

# Validate bot compatibility
validate-bots:
    cd backend && cargo test -- original_bots --nocapture
