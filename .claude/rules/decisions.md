# Architectural Decision Records (ADR)

This document records important architectural and technical decisions made for the project.

## ADR-001: Use Biome instead of ESLint + Prettier

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Need a linter and formatter for TypeScript/React code. Traditional choice is ESLint + Prettier.

### Decision
Use Biome (Rust-based, single tool) instead of ESLint + Prettier.

### Rationale
- **Performance**: 25-100x faster than ESLint + Prettier
- **Simplicity**: Single tool instead of two
- **Modern**: Built for modern JavaScript/TypeScript
- **Project size**: Small project, simpler tooling is better

### Consequences
**Positive**:
- Much faster linting and formatting
- Simpler configuration (one config file)
- Better VSCode integration

**Negative**:
- Fewer plugins available vs ESLint ecosystem
- Smaller community (but growing rapidly)

---

## ADR-002: Do NOT use oxlint/oxfmt/tsgo

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Zenn article suggested oxlint, oxfmt, and tsgo for maximum performance.

### Decision
Stay with current tooling (Biome + tsc + esbuild).

### Rationale
- **Project scale**: Only 77 lines of TypeScript
- **Stability**: tsgo is still in preview, unstable
- **No bottleneck**: Current tools are already fast enough
- **Simplicity**: Fewer tools = easier maintenance
- **YAGNI**: Don't optimize what's not a problem

### Consequences
**Positive**:
- Stable, proven tools
- Simpler setup
- Lower learning curve

**Negative**:
- Slightly slower than bleeding-edge tools (but not noticeable at this scale)

**Reconsider when**:
- Project grows to 50+ files
- Type checking takes >30 seconds
- CI time becomes a bottleneck
- tsgo reaches stable release

---

## ADR-003: No complex folder structure

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Typical React projects use feature-based or layered folder structures.

### Decision
Keep flat structure in `src/` directory.

### Current Structure
```
src/
├── App.tsx
├── main.tsx
└── index.css
```

### Rationale
- **YAGNI**: Only 3 files, no need for structure
- **Clarity**: Everything is immediately visible
- **Refactorable**: Easy to reorganize when needed

### Consequences
**Positive**:
- Zero cognitive overhead
- No time wasted on "where does this go?"
- Easy to understand for new contributors

**Negative**:
- Will need refactoring if project grows

**Reconsider when**:
- 5+ React components
- Multiple custom hooks
- Shared utilities needed
- Multiple features/pages

---

## ADR-004: D-Bus for daemon-to-app communication

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Need IPC between root daemon (evdev access) and user-level Tauri app.

### Decision
Use D-Bus session bus.

### Rationale
- **Linux standard**: Well-supported on GNOME/Wayland
- **Security**: Proper permission separation
- **Reliability**: Proven technology
- **zbus library**: Excellent Rust support

### Alternatives Considered
1. **Unix socket**: More complex permission handling
2. **TCP localhost**: Overkill for local IPC
3. **Shared memory**: Too complex for simple signals

### Consequences
**Positive**:
- Proper security boundaries
- Standard Linux approach
- Easy to debug with `dbus-monitor`

**Negative**:
- D-Bus not available on Windows
- Adds dependency on system D-Bus daemon

---

## ADR-005: Remove all personal information from commits

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Repository initially contained personal names in code, commits, and documentation.

### Decision
Rewrite git history and remove all personal identifiable information.

### Changes Made
- Removed author names from code
- Changed "うちお (@uchiwo)" to "Anonymous"
- Updated all D-Bus interfaces to use `io.github.noppomario.uti`
- Rewrote commit history

### Rationale
- Privacy concerns
- Professional presentation
- Easier to transfer ownership if needed

### Consequences
**Positive**:
- Clean, professional codebase
- Privacy protected

**Negative**:
- Lost original commit history
- Force-pushed to remote

---

## ADR-006: English-only for all code and documentation

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Original code contained Japanese comments and documentation.

### Decision
Translate everything to English.

### Rationale
- **International collaboration**: English is lingua franca of programming
- **Tool compatibility**: Better IDE/AI support
- **Best practice**: Standard in open source

### Consequences
**Positive**:
- Accessible to international contributors
- Better tooling support
- Professional standard

**Negative**:
- None (Japanese UI text can be added later for localization)

---

## ADR-007: Target Linux only (for now)

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Tauri supports Windows, macOS, and Linux. evdev is Linux-only.

### Decision
Focus on Linux (Fedora 43 GNOME/Wayland) for initial release.

### Rationale
- **evdev**: Linux-specific API
- **D-Bus**: More common on Linux
- **Focus**: Better to do one platform well
- **Future**: Can add Windows support later with different backend

### Windows Support Plan (Future)
- Replace evdev with Windows low-level keyboard hooks
- Replace D-Bus with TCP localhost or named pipes
- Use conditional compilation (`#[cfg(target_os)]`)

### Consequences
**Positive**:
- Simpler initial implementation
- Can optimize for Linux

**Negative**:
- Limited audience
- Future refactoring needed for Windows

---

## ADR-008: Monorepo task architecture with workspace-first design

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Project has three workspaces (frontend, tauri, daemon). Need standardized task execution across workspaces with CI/local parity.

### Decision
Use **workspace-first naming** pattern `[workspace]:[task]`:
- Workspaces: `frontend`, `tauri`, `daemon`, `app`, `rust`, `all`
- Tasks: `format`, `lint`, `typecheck`, `build`, `test`, `all`

Root `package.json` as task runner with `npm-run-all` for parallel execution.
Rust workspace (`Cargo.toml`) for unified Rust management.

### Rationale
- Workspace-centric thinking: `bun run daemon:all`
- Selective execution: Only run tasks for modified workspace
- CI/local parity: Same `ci:local` command everywhere
- Parallel execution: Independent workspaces run concurrently

### Consequences
**Positive**:
- Clear task naming
- Fast selective execution
- CI and local perfectly aligned

**Negative**:
- Requires bun/npm installed
- Learning curve for naming convention

---

## ADR-009: Pre-commit quality gate with ci:local requirement

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Need to prevent low-quality commits while maintaining fast workflow.

### Decision
Two-layer quality gate:
1. **Automatic**: Pre-commit hook formats and lints staged files
2. **Manual**: Developers run `bun run ci:local` before significant commits

### Rationale
- Automatic formatting prevents unformatted code
- Full `ci:local` check too slow for every commit
- Trust developers for comprehensive checks
- CI still catches everything if forgotten

### Consequences
**Positive**:
- Fast commits for small changes
- Automatic formatting
- Reduced CI failures

**Negative**:
- Developers might forget `ci:local`
- Requires discipline

**Reconsider when**:
- CI failure rate exceeds 20%

---

## ADR-010: Adopt Vitest for testing with TDD methodology

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context
Project had no automated tests. Need to establish testing infrastructure with TDD methodology for future development.

Two main options considered:
1. **Bun native test runner** - Zero additional dependencies, ultra-fast
2. **Vitest** - Mature ecosystem, Vite integration

### Decision
Use **Vitest** for frontend testing, standard `cargo test` for Rust.
Require **Test-Driven Development (TDD)** for all new code.

### Rationale

#### Why Vitest over Bun Test (2025 analysis)

**Test Isolation (Critical for TDD)**:
- Vitest: Complete isolation between test suites (default)
- Bun Test: Shared global state, mocks leak between tests

**Ecosystem Maturity**:
- Vitest: Full IDE integration, browser mode, sharding, type testing
- Bun Test: Limited IDE support, experimental coverage

**Vite Ecosystem Consistency**:
- Already using Vite (Tauri recommended)
- Vitest shares `vite.config.ts` configuration
- Same plugin ecosystem

**Production Readiness**:
- Vitest: Battle-tested, recommended by Evan You (Vite creator)
- Bun Test: Production-ready for internal tools, but isolated testing issues remain

#### Why NOT Bun Test
- Global state leakage breaks TDD reliability
- No IDE integration for watch mode debugging
- Coverage reporting still basic (CLI-only output)

#### Oxc Ecosystem Consideration
- oxfmt/oxlint (Dec 2025 alpha) are faster than Biome (2-3x)
- Decision: Keep Biome for now due to maturity and CST-based editor support
- Reconsider when oxfmt reaches stable release

### Alternatives Considered

1. **Bun native test runner**
   - Pro: 5x faster than Vitest, zero dependencies
   - Con: Test isolation issues, limited tooling

2. **Jest**
   - Pro: Most popular, huge ecosystem
   - Con: 8-20x slower than alternatives, outdated architecture

### Consequences

**Positive**:
- Reliable TDD workflow with isolated tests
- Excellent watch mode for red-green-refactor cycle
- Coverage reports integrated with CI
- Consistent with Vite build tooling
- Strong TypeScript/React support

**Negative**:
- 3 additional dependencies vs Bun native
- Slightly slower than Bun (though fast enough in practice)

**Migration Path**:
- If Bun test fixes isolation issues in future, migration is easy (Jest-compatible API)

### Implementation Details

**Dependencies**:
```bash
bun add -d vitest @testing-library/react @testing-library/dom happy-dom
```

**Coverage Targets**:
- Minimum: 70% line coverage
- Target: 80%+ overall
- Critical paths (daemon keyboard logic): 90%+

**TDD Enforcement**:
- Documented in conventions.md
- Required for all new features/bug fixes
- CI runs tests on every commit
- Watch mode recommended for development

### Reconsider When
- Bun test implements proper test isolation
- Project scales beyond 10,000+ tests (speed becomes critical)
- Vitest maintenance stalls

---

## Template for Future ADRs

```markdown
## ADR-XXX: [Title]

**Date**: YYYY-MM-DD
**Status**: [Proposed | Accepted | Deprecated | Superseded]
**Decision Makers**: [Who decided]

### Context
[What is the situation and problem?]

### Decision
[What are we doing?]

### Rationale
[Why this decision?]

### Alternatives Considered
1. [Alternative 1]
2. [Alternative 2]

### Consequences
**Positive**:
- [Good outcome 1]

**Negative**:
- [Trade-off 1]

**Reconsider when**:
- [Condition 1]
```
