# Template Engine Plan

This document captures the agreed project structure, phase plan, and guardrails for the template engine workstream.

## Project Tree (Annotated)

```
template-engine/
  Cargo.toml
  README.md                         # top-level: what this is, install, examples
  LICENSE*
  rust-toolchain.toml               # pin toolchain (keep MSRV documented)

  spec/
    WIRE_FORMAT.md                  # the product: JSON schema + versioning rules
    CANONICALIZATION.md             # stable ordering + deterministic output
    VAR_SYNTAX.md                   # :slug: grammar, escaping, max slug len
    DIFF_PATCH.md                   # patch semantics, reversibility, pretty diffs

  schemas/
    template.schema.json            # Template/Segment/VarKey JSON schema (v1.0)
    binding_report.schema.json      # BindResult + BindingReport schema
    chunk_plan.schema.json          # ChunkPlan schema
    diff.schema.json                # DiffResult + PatchBundle schema

  fixtures/
    templates/
      basic.json                    # known-good AST fixtures
      edge_cases.json               # escaping, adjacent vars, unicode, etc
    binding/
      overlay_hit.json              # overlay > env chain precedence
      env_chain.json                # scope label hit reporting
      unresolved.json               # missing values report
    chunking/
      preserves_vars.json           # never splits inside vars
      semantic_boundaries.json      # uses text-splitter boundaries
    diff/
      json_object_change.json       # structured changes
      string_lcs_pretty.json        # pretty diff hunks
      reversible_patch.json         # forward+inverse roundtrip

  crates/
    template_engine/                # PURE CORE (no IO, no pyo3, no wasm-bindgen)
      Cargo.toml
      src/
        lib.rs                      # JSON-in/out façade + typed API
        error.rs                    # stable errors
        ast.rs                      # Template/Segment/VarKey definitions (serde)
        tokenize.rs                 # streaming tokenizer: TextRun/VarToken spans
        parse.rs                    # parse :slug: into segments (uses tokenizer)
        render.rs                   # template/resolved/annotated render modes
        bind/
          mod.rs
          env.rs                    # Env + Overlay + PlacementMap
          binder.rs                 # placement pass + binding pass
          report.rs                 # BindingReport/Event canonicalization
        chunk/
          mod.rs
          planner.rs                # var-aware chunk planner
          splitter.rs               # adapter to text-splitter (text runs only)
        codecs/
          mod.rs
          registry.rs               # schema-uri -> codec
          builtins/
            time_iso.rs             # podium:schema:time (ISO-8601)
            money.rs                # currency + decimal canonicalization
            phone.rs                # E.164 canonicalization
            duration.rs             # ISO-8601 duration or Podium canonical
        diff/
          mod.rs
          fionn.rs                  # fionn-diff adapter (diff/patch/merge + pretty)
          rfc6902.rs                # json-patch adapter (RFC 6902/7396)
          reversible.rs             # PatchBundle {forward,inverse} semantics
          pretty.rs                 # string LCS hunks + path-level summaries
        canonical.rs                # deterministic ordering helpers

    template_engine_py/             # Python bindings (thin, JSON-in/out)
      Cargo.toml
      pyproject.toml                # maturin
      src/lib.rs                    # pyo3 exports
      python/template_engine/__init__.py   # nice Python types wrapping JSON

    template_engine_wasm/           # WASM bindings (thin, JSON-in/out)
      Cargo.toml
      src/lib.rs                    # wasm-bindgen exports
      ts/
        index.ts                    # typed wrapper, JSON.parse/stringify glue
      package.json
      tsconfig.json

  .github/workflows/
    ci.yml                          # fmt, clippy, tests, schema validation, wasm build

  tools/
    schema_check.rs                 # validates fixtures vs schemas (or ajv in node)
    gen_schemas.rs                  # optional: emit JSON schema from Rust types
```

## Phase Plan

### Phase 0: Non-negotiable constraints (day 0 decisions)

1. Wire format is the product: Template/BindResult/ChunkPlan/DiffResult schemas are versioned and frozen in `schemas/`.
2. Core is pure: no filesystem, no network, no time, no randomness.
3. Determinism: canonical ordering rules, stable event ordering, stable diff output ordering.
4. Vars are atoms: no splitter or diff algorithm is allowed to cut through a var token.

### Phase 1: Core template engine (AST, parse, render, bind)

Deliverables:

- `ast.rs`: `Template`, `Segment`, `VarKind`, `Source`, `VarKey`.
- `tokenize.rs`: streaming tokenizer returning spans of `TextRun` and `VarToken` using strict `:slug:` grammar + `MAX_SLUG_LEN`.
- `parse.rs`: builds `Template { segments }` from string.
- `render.rs`: template | resolved | annotated render modes.
- `bind/`: two-pass binder:
  - Pass 1 placement: fill missing source for runtime vars using placement map.
  - Pass 2 binding: overlay first, then env chain; populate segment value and `BindingEvent`.
- `fixtures/templates` + `fixtures/binding` green in CI.

Exit criteria:

- JSON-in/out functions match in Rust/Python/WASM.
- Binding report is stable across runs.

### Phase 2: Var-aware semantic chunking (text-splitter integration)

Deliverables:

- `chunk/planner.rs`: builds `ChunkPlan { chunks: Vec<Template>, report }`.
- Algorithm:
  1. tokenize original template string or template segments
  2. treat vars as atomic segments
  3. run text-splitter on each text run with remaining chunk capacity (char or token sizer)
  4. assemble new `Template` chunks that preserve the original var segments
- Options:
  - capacity: range support (`min..max`)
  - sizer: chars | tiktoken | HF tokenizer (feature flags)
  - “prefer split at boundary X” knob (sentence/newline/markdown heading)

Exit criteria:

- `preserves_vars` fixture proves no var is split.
- Chunk plan is stable and predictable.

### Phase 3: Schema-driven codecs (time, money, phone, duration)

Deliverables:

- `codecs/registry.rs`: `CodecRegistry` with built-ins.
- Built-in codecs (first wave):
  - `podium:schema:time` -> ISO-8601 formatting/parsing
  - `podium:schema:money` -> decimal + currency canonical form (no floats)
  - `podium:schema:phone` -> E.164 canonical form
  - `podium:schema:duration` -> ISO-8601 duration canonical form
- Binder integration:
  - binder still binds JSON values
  - renderer (or `render_resolved_with_codecs`) uses codec to produce string output
  - report includes both raw JSON and rendered string when codec applies (for FE debugging)

Exit criteria:

- Roundtrip tests: `decode(encode(v)) == canonicalize(v)`.
- Rendering differences only where schema indicates.

### Phase 4: JSON diff + patch + reversibility (fionn-diff first, RFC 6902 also)

Why fionn-diff:

- It supports JSON Merge Patch (RFC 7396) and focuses on SIMD performance, including LCS in arrays.
- It exposes explicit diff/patch types in its API surface.

Deliverables:

- `diff/fionn.rs`: fionn-diff adapter:
  - `diff(left, right, options) -> DiffResult` (includes pretty hunks)
  - `apply_patch(doc, patch) -> doc`
  - `apply_merge_patch(doc, merge_patch) -> doc` (RFC 7396 semantics)
- `diff/rfc6902.rs`: json-patch adapter for RFC 6902 patch arrays + merge patch fallback.
- `diff/reversible.rs`:
  - `PatchBundle { forward, inverse }`
  - compute inverse by diffing reversed args (simple and reliable)
- `diff/pretty.rs`:
  - For strings: LCS-based hunks output for “git-like” diffs (display-only)
  - For objects: path-level summary (added/removed/changed keys)
  - For arrays: rely on fionn-diff’s LCS where possible, augment with display hunks

Exit criteria:

- `apply(forward, left) == right`
- `apply(inverse, right) == left`
- Pretty output is stable and readable.

### Phase 5: Python + WASM packaging (thin, disciplined)

Deliverables:

- Python:
  - pyo3 exports: JSON-in/out functions only
  - pure Python wrapper: typed convenience (dataclasses optional), but no logic
- WASM:
  - wasm-bindgen exports: JSON-in/out
  - TS wrapper: types + runtime validators (optional Ajv against schemas)

Exit criteria:

- A single golden fixture passes in Rust, Python, and browser test harness.

### Phase 6: CI + release hygiene

Deliverables:

- CI runs:
  - fmt/clippy/tests
  - schema validation of fixtures
  - wasm build
  - python wheels build (at least linux)
- Release:
  - bump wire_version only when breaking
  - keep a changelog that is mostly “wire changes” and “behavior changes”

## Guardrails

- Don’t let pretty diffs leak into patch semantics. Pretty is display; patch is executable.
- Don’t let codec formatting leak into storage identity. Store canonical JSON, format at render time.
- Don’t try to make JS bindings ergonomic by exporting complex structs. JSON strings are the truth.

## Diff Engine Choice

- fionn-diff is the primary native engine for JSON diff/patch/merge with SIMD acceleration and LCS for arrays.
- json-patch remains the workhorse for RFC 6902/7396 interoperability.
