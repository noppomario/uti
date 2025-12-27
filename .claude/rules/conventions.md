# Code Conventions

## Language Requirements

### All Code & Documentation
- **English only** - No Japanese in code, comments, docs, or commit messages
- Exception: User-facing UI text can be localized in the future

## Documentation Standards

### TypeScript/JavaScript
Use **TSDoc** format:

```typescript
/**
 * Brief description of function/component
 *
 * Detailed explanation if needed.
 *
 * @param paramName - Description
 * @returns Description of return value
 *
 * @example
 * ```typescript
 * exampleUsage();
 * ```
 */
```

### Rust
Use **Rust doc comments**:

```rust
/// Brief description
///
/// Detailed explanation.
///
/// # Arguments
///
/// * `param` - Description
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function returns an error
///
/// # Examples
///
/// ```no_run
/// example_usage();
/// ```
```

Module-level docs:
```rust
//! Module description
//!
//! Detailed module documentation.
```

## Code Style

### General
- Max line width: **100 characters**
- Line endings: **LF** (Unix-style)
- Encoding: **UTF-8**
- Final newline: **Required**

### TypeScript/React
- Indent: **2 spaces**
- Quotes: **Single quotes** for JS, **double quotes** for JSX
- Semicolons: **Required**
- Trailing commas: **ES5 style**
- Arrow functions: Parentheses **only when needed**

```typescript
// Good
const fn = x => x * 2;
const fn2 = (x, y) => x + y;

// Bad
const fn = (x) => x * 2;
```

### Rust
- Indent: **4 spaces**
- Max line width: **100 characters**
- Use `rustfmt` defaults with project config
- Prefer `?` operator over explicit error handling
- Use `impl Trait` for return types when appropriate

## Naming Conventions

### Files
- TypeScript/React: **PascalCase** for components (`App.tsx`)
- TypeScript: **camelCase** for utilities (`utils.ts`)
- Rust: **snake_case** (`main.rs`)
- Config files: **lowercase** with extensions (`.editorconfig`, `biome.json`)

### Code
- **TypeScript**:
  - Components: `PascalCase`
  - Functions/variables: `camelCase`
  - Constants: `UPPER_SNAKE_CASE`
  - Types/Interfaces: `PascalCase`

- **Rust**:
  - Functions/variables: `snake_case`
  - Types/Traits: `PascalCase`
  - Constants: `UPPER_SNAKE_CASE`
  - Modules: `snake_case`

## Import Organization

### TypeScript
Biome automatically organizes imports. Order:
1. External packages
2. Internal absolute imports
3. Relative imports

### Rust
Use `rustfmt` import grouping:
1. `std` imports
2. External crates
3. Internal modules

## Git Commit Messages

### Format
```
<type>: <short description>

<optional detailed description>

<optional footer>
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style/formatting (no logic change)
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Adding/updating tests
- `chore`: Tooling, dependencies, config

### Examples
```
feat: Add system tray icon support

Implement tray icon using tauri-plugin-tray with toggle menu.

Closes #123
```

```
fix: Prevent daemon crash on keyboard disconnect

Add error handling for evdev device disconnection events.
```

## Linting Rules

### Enforced by Biome
- Accessibility warnings for interactive elements
- Type import enforcement (`import type`)
- No explicit `any` (warn)
- Button elements must have `type` attribute

### Enforced by clippy (future)
- Standard Rust best practices
- Explicit error handling
- Unused code detection

## Testing Conventions (Future)

### TypeScript
- Test files: `*.test.tsx` or `*.spec.tsx`
- Place tests next to source files
- Use React Testing Library

### Rust
- Unit tests: In same file with `#[cfg(test)]`
- Integration tests: In `tests/` directory
- Use standard `#[test]` attribute

## Documentation Requirements

### When to Document
- All public functions/components
- All exported types/interfaces
- Complex logic or algorithms
- Non-obvious decisions

### When NOT to Document
- Self-evident code
- Private helper functions (unless complex)
- Getters/setters with obvious behavior

## Monorepo Task Naming

Pattern: `[workspace]:[task]`

**Workspaces:** `frontend`, `tauri`, `daemon`, `app`, `rust`, `all`
**Tasks:** `format`, `lint`, `typecheck`, `build`, `test`, `all`

Examples:
```bash
bun run frontend:all    # All tasks for frontend
bun run daemon:build    # Build daemon only
bun run all:lint        # Lint all workspaces
bun run ci:local        # CI equivalent check
```

## Code Review Checklist

Before committing:
1. ✅ Code formatted (auto on save)
2. ✅ No linting errors (`bun run check`)
3. ✅ Type check passes (`bun run type-check`)
4. ✅ **Run `bun run ci:local` for significant changes**
5. ✅ Documentation added for public APIs
6. ✅ Commit message follows convention
7. ✅ No personal information in changes
8. ✅ English only in code/comments
