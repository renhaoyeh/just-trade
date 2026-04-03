# Just Trade - Development Guidelines

## Project Overview
A Tauri 2 (Rust backend) + React (TypeScript frontend) desktop trading application focused on Taiwan stocks.

## Tech Stack
- **Backend:** Rust, Tauri 2, SQLx (PostgreSQL), reqwest
- **Frontend:** React 19, TypeScript, Vite, shadcn/ui, recharts, Tailwind CSS
- **Testing:** Vitest (unit), Playwright (e2e)

## Development Workflow

### Testing Requirements
- **Every task must include unit tests** before committing
- **UI changes must include e2e tests** (Playwright) to verify functionality
- Run `npm test` (Vitest) and ensure all tests pass before committing
- Run `npm run test:e2e` (Playwright) for e2e tests when UI changes are involved

### Commit Strategy
- Commit per logical step (not one giant commit)
- Each commit should have passing tests
- Follow conventional commit format: `feat:`, `fix:`, `chore:`, `test:`, etc.

## Commands
- `npm run dev` — Start Vite dev server (frontend only)
- `npm run tauri dev` — Start full Tauri app (frontend + Rust backend)
- `npm test` — Run Vitest unit tests
- `npm run test:e2e` — Run Playwright e2e tests
- `cargo check` — Check Rust compilation (run from src-tauri/)
- `cargo test` — Run Rust unit tests (run from src-tauri/)

## Project Structure
```
src/                    # React frontend
  pages/                # Page components
  components/           # UI components (shadcn/ui)
  services/             # Tauri invoke wrappers
  types/                # TypeScript interfaces
  hooks/                # React hooks
src-tauri/              # Rust backend
  src/commands/         # Tauri command handlers
  src/services/         # Business logic (Yahoo Finance client, etc.)
  src/models/           # Data models
  src/db/               # Database access layer
  migrations/           # SQLx migrations
```

## Taiwan Stock Symbols
- TWSE (listed): `{code}.TW` (e.g., `2330.TW` for TSMC)
- TPEx (OTC): `{code}.TWO` (e.g., `6510.TWO`)
