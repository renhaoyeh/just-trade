# Just Trade - Development Guidelines

## Development Workflow

### Testing Requirements
- **Every task must include unit tests** before committing
- **UI changes must include e2e tests** (Playwright) to verify functionality
- Run `npm test` (Vitest) and ensure all tests pass before committing
- Run `npm run test:e2e` (Playwright) for e2e tests when UI changes are involved

### Build Verification
- Before committing, ensure the app can actually run: `cargo check` (from src-tauri/) and `npm run dev`
- DB migrations must have unique version numbers — check existing files in `src-tauri/migrations/` before creating new ones

### Commit Strategy
- Commit per logical step (not one giant commit)
- Each commit should have passing tests
- Follow conventional commit format: `feat:`, `fix:`, `chore:`, `test:`, etc.

## i18n
- All user-facing strings must use `t("key")` from `react-i18next`, never hardcode text in components
- Default language: `zh-TW`, fallback: `en`
- Translation files: `src/i18n/locales/{zh-TW,en}.json`
- Adding new UI text: add the key to **both** locale files, then use `t("section.key")` in the component
- Interpolation: `t("key", { value })` → `"P/E: {{value}}"` in JSON
- Switch language at runtime: `i18n.changeLanguage("en")`
- Tailwind v4 uses OKLCH colors — use `var(--color-*)` in inline styles (not `hsl(var(--*))`)
