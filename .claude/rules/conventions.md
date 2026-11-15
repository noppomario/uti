# Code Conventions

## Language Requirements

### All Code & Documentation

- **English only** - No Japanese in code, comments, docs, or commit messages
- Exception: User-facing UI text can be localized in the future

## Documentation Standards

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

- Max line width: **100 characters** (code only; Markdown is unlimited)
- Line endings: **LF** (Unix-style)
- Encoding: **UTF-8**
- Final newline: **Required**

### JavaScript (GNOME extension)

- Indent: **2 spaces**
- Formatting enforced by Biome (`just lint-ext`)

### Rust Style

- Indent: **4 spaces**
- Max line width: **100 characters**
- Use `rustfmt` defaults with project config
- Prefer `?` operator over explicit error handling
- Use `impl Trait` for return types when appropriate

## Naming Conventions

### Files

- Rust: **snake_case** (`main.rs`)
- Config files: **lowercase** with extensions (`.editorconfig`, `biome.json`)

### Code

- **Rust**:
  - Functions/variables: `snake_case`
  - Types/Traits: `PascalCase`
  - Constants: `UPPER_SNAKE_CASE`
  - Modules: `snake_case`

## Import Organization

### Rust Imports

Use `rustfmt` import grouping:

1. `std` imports
2. External crates
3. Internal modules

## Git Commit Messages

### Format

```text
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
- `build`: Build system, packaging
- `ci`: CI workflows
- `chore`: Tooling, dependencies, config

### Examples

```text
feat: Add pin button to main window

Keep the window visible and request always-on-top from the GNOME extension.

Closes #123
```

```text
fix: Prevent daemon crash on keyboard disconnect

Add error handling for evdev device disconnection events.
```

## Linting Rules

### Enforced by clippy

- `cargo clippy` with `-D warnings` (`just lint-rust`)

### Enforced by Biome

- GNOME extension JavaScript only (`just lint-ext`)

### Enforced by rumdl

- Markdown, rules from `.markdownlint.json` (`just lint-md`)

## Testing Conventions

### Test-Driven Development (TDD)

**Required for all new features and bug fixes.**

#### TDD Cycle

1. **Write test first** - Test must fail initially
2. **Implement minimal code** - Make test pass
3. **Refactor** - Improve while keeping tests green
4. **Repeat** - Continue for next requirement

### GTK App Testing

Widgets are not unit-tested. Put UI logic in `app/src/state/` as pure state
transitions (no GTK types) and test them there; `app.rs` / `ui.rs` only apply
the resulting state to widgets.

### Rust Testing

#### Unit Tests

Place in same file with `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_tap_detection() {
        // Arrange
        let detector = DoubleTapDetector::new();

        // Act
        let result = detector.check_interval(250);

        // Assert
        assert!(result.is_within_threshold());
    }
}
```

#### Integration Tests

Place in `tests/` directory:

```rust
// tests/dbus_integration.rs
#[test]
fn test_dbus_signal_sending() {
    // Test D-Bus communication
}
```

### Coverage Requirements

- **Minimum**: 70% line coverage
- **Target**: 80%+ for critical paths
- **Critical code** (daemon keyboard logic): 90%+

### Running Tests

#### During Development (TDD)

```bash
# Watch mode - tests rerun on file change
cargo watch -x test     # Rust (requires cargo-watch)
```

#### Before Commit

```bash
just ci                 # lint + deny + test
```

#### Coverage Check

```bash
cargo tarpaulin         # Rust coverage (requires cargo-tarpaulin)
```

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

## Tasks (justfile)

`setup`, `fmt`, `lint` (`lint-rust`, `lint-md`, `lint-ext`, `lint-gha`, `lint-sh`), `deny`, `test`, `ci`,
`run *args`, `rpm`. For a single crate use `cargo -p <crate>`.

## Git Hooks (lefthook)

- pre-commit: formats staged files (rustfmt, biome, rumdl) and re-stages fixes
- commit-msg: Conventional Commits (`feat fix perf refactor docs test style build ci chore`;
  Merge/Revert/fixup allowed)
- pre-push: `just lint-rust test`
- Skip once with `LEFTHOOK=0`

## Code Review Checklist

Before committing:

1. ✅ Code formatted (`just fmt`; pre-commit hook formats staged files)
2. ✅ No linting errors (`just lint`)
3. ✅ **Run `just ci`**
4. ✅ Documentation added for public APIs
5. ✅ Commit message follows convention
6. ✅ No personal information in changes
7. ✅ English only in code/comments
