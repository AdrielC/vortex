# Template Engine Plan

This document captures the agreed project structure, phase plan, and guardrails for the template engine workstream.

## Project Structure Overview

- Rust workspace (Cargo) under `template-engine/` with three crates:
  - `crates/template_engine/` (core, pure Rust; JSON-in/out facade + typed API).
  - `crates/template_engine_py/` (Python bindings via `pyo3` + `maturin`).
  - `crates/template_engine_wasm/` (WASM bindings via `wasm-bindgen` + TS wrapper).
- Specs and schemas live at the top level under `spec/` and `schemas/` to keep wire-format evolution explicit.
- Fixtures live under `fixtures/` and are validated in CI against the schemas.
- Tooling helpers live under `tools/` (schema check, optional schema generator).

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

## Agent Fork → Propose → Diff → Apply → Revert → Commit Loop (Concrete Plan)

This section makes the agent loop concrete with module layout, wire types, and Rust code examples. The ABI is JSON-in/out.

### Modules + Responsibilities

```
template_engine/src/
  ast.rs                 # Template/Segment
  tokenize.rs            # parse :slug: to segments
  render.rs              # render template/resolved/annotated
  bind/                  # env placement + binding report

  edit/
    mod.rs
    plan.rs              # EditPlan + selectors
    compile.rs           # EditPlan -> new Template (deterministic)
    diff.rs              # Template-aware diff (segments + text hunks)
    patch.rs             # PatchBundle forward/inverse + hashing guards
    apply.rs             # apply_patch_bundle, revert, patch chains

  diff/
    fionn.rs             # optional: fionn-diff for JSON diffs
    rfc6902.rs           # json-patch RFC6902 apply
```

Key design choice: EditPlan compiles to a new `Template` deterministically, then we compute a patch bundle as `diff(old, new)`. This keeps reversibility and agent sanity.

### Wire Types

#### `Template` + `Segment`

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub wire_version: String, // "1.0"
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Segment {
    #[serde(rename = "text")]
    Text { value: String },

    #[serde(rename = "var")]
    Var {
        slug: String,
        kind: VarKind,
        source: Option<Source>,
        schema: Option<String>,
        value: Option<Value>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum VarKind {
    Preset,
    Runtime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    AgentConfig,
    SectorConfigs,
}
```

#### `EditPlan` + selectors

```rust
use serde::{Deserialize, Serialize};
use crate::ast::{Segment, Source, VarKind};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditPlan {
    pub wire_version: String,        // "1.0"
    pub fork_id: String,             // "agent:proposal:123"
    pub edits: Vec<Edit>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Edit {
    #[serde(rename = "replace_text")]
    ReplaceText {
        target: TextTarget,
        value: String,
    },

    #[serde(rename = "insert_var")]
    InsertVar {
        target: TextTarget,
        var: VarSpec,
    },

    #[serde(rename = "delete_var")]
    DeleteVar { segment_index: usize },

    #[serde(rename = "update_var")]
    UpdateVar {
        segment_index: usize,
        patch: VarPatch,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextTarget {
    pub segment_index: usize,
    pub start: usize, // utf-8 byte offset
    pub end: usize,   // exclusive
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VarSpec {
    pub slug: String,
    pub kind: VarKind,
    pub source: Option<Source>,
    pub schema: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VarPatch {
    pub source: Option<Option<Source>>,
    pub schema: Option<Option<String>>,
    pub kind: Option<VarKind>,
    pub value: Option<Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Suggestion {
    #[serde(rename = "var_value")]
    VarValue {
        suggestion_id: String,
        slug: String,
        source: Source,
        schema: Option<String>,
        proposed_value: serde_json::Value,
        status: SuggestionStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionStatus {
    PendingUser,
    AutoAccepted,
    Rejected,
}
```

#### `PatchBundle` (reversible, hash-guarded)

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchBundle {
    pub wire_version: String, // "1.0"
    pub left_hash: String,    // "blake3:...."
    pub right_hash: String,   // "blake3:...."
    pub forward: Patch,
    pub inverse: Patch,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Patch {
    #[serde(rename = "rfc6902")]
    Rfc6902 { ops: Vec<Rfc6902Op> },

    #[serde(rename = "fionn")]
    Fionn { patch: Value },

    #[serde(rename = "merge7396")]
    Merge7396 { patch: Value },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rfc6902Op {
    pub op: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}
```

#### `TemplateDiff` (segment + text hunks)

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TemplateDiff {
    pub wire_version: String, // "1.0"
    pub segment_changes: Vec<SegmentChange>,
    pub text_hunks: Vec<TextHunkDiff>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SegmentChange {
    #[serde(rename = "insert_var")]
    InsertVar { at: usize, slug: String },

    #[serde(rename = "delete_var")]
    DeleteVar { at: usize, slug: String },

    #[serde(rename = "update_var")]
    UpdateVar { at: usize, slug: String, fields: Vec<String> },

    #[serde(rename = "replace_text")]
    ReplaceText { at: usize, before_len: usize, after_len: usize },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextHunkDiff {
    pub segment_index: usize,
    pub before: String,
    pub after: String,
    pub hunks: Vec<StringHunk>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StringHunk {
    pub op: String, // "equal" | "delete" | "insert"
    pub text: String,
}
```

### Deterministic Edit Application

```rust
use crate::ast::{Segment, Template};
use crate::edit::plan::{Edit, EditPlan, TextTarget};
use crate::error::EngineError;

pub fn apply_edits(base: &Template, plan: &EditPlan) -> Result<Template, EngineError> {
    let mut t = base.clone();

    let mut edits = plan.edits.clone();
    edits.sort_by(|a, b| sort_key(a).cmp(&sort_key(b)));

    for e in edits {
        apply_one(&mut t, e)?;
    }

    Ok(t)
}

fn sort_key(e: &Edit) -> (usize, usize, u8) {
    match e {
        Edit::ReplaceText { target, .. } => (target.segment_index, usize::MAX - target.start, 0),
        Edit::InsertVar { target, .. } => (target.segment_index, usize::MAX - target.start, 1),
        Edit::UpdateVar { segment_index, .. } => (*segment_index, 0, 2),
        Edit::DeleteVar { segment_index } => (*segment_index, 0, 3),
    }
}

fn apply_one(t: &mut Template, e: Edit) -> Result<(), EngineError> {
    match e {
        Edit::ReplaceText { target, value } => replace_text(t, target, value),
        Edit::InsertVar { target, var } => insert_var(t, target, var),
        Edit::DeleteVar { segment_index } => delete_var(t, segment_index),
        Edit::UpdateVar { segment_index, patch } => update_var(t, segment_index, patch),
    }
}

fn replace_text(t: &mut Template, target: TextTarget, value: String) -> Result<(), EngineError> {
    let seg = t.segments.get_mut(target.segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    let Segment::Text { value: ref mut s } = seg else {
        return Err(EngineError::InvalidArgument("ReplaceText target must be text segment".into()));
    };

    if target.start > target.end || target.end > s.len() {
        return Err(EngineError::InvalidArgument("invalid text range".into()));
    }

    s.replace_range(target.start..target.end, &value);
    Ok(())
}

fn insert_var(t: &mut Template, target: TextTarget, var: crate::edit::plan::VarSpec) -> Result<(), EngineError> {
    let seg = t.segments.get_mut(target.segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    let Segment::Text { value: ref mut s } = seg else {
        return Err(EngineError::InvalidArgument("InsertVar target must be text segment".into()));
    };

    if target.start > s.len() {
        return Err(EngineError::InvalidArgument("insert offset out of bounds".into()));
    }

    let right = s.split_off(target.start);
    let left = std::mem::take(s);

    t.segments[target.segment_index] = Segment::Text { value: left };
    t.segments.insert(target.segment_index + 1, Segment::Var {
        slug: var.slug,
        kind: var.kind,
        source: var.source,
        schema: var.schema,
        value: None,
    });
    t.segments.insert(target.segment_index + 2, Segment::Text { value: right });

    Ok(())
}

fn delete_var(t: &mut Template, segment_index: usize) -> Result<(), EngineError> {
    let seg = t.segments.get(segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    match seg {
        Segment::Var { .. } => {
            t.segments.remove(segment_index);
            Ok(())
        }
        _ => Err(EngineError::InvalidArgument("DeleteVar target must be var segment".into())),
    }
}

fn update_var(t: &mut Template, segment_index: usize, patch: crate::edit::plan::VarPatch) -> Result<(), EngineError> {
    let seg = t.segments.get_mut(segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    let Segment::Var { kind, source, schema, value, .. } = seg else {
        return Err(EngineError::InvalidArgument("UpdateVar target must be var segment".into()));
    };

    if let Some(k) = patch.kind {
        *kind = k;
    }
    if let Some(src) = patch.source {
        *source = src;
    }
    if let Some(sc) = patch.schema {
        *schema = sc;
    }
    if let Some(v) = patch.value {
        *value = v;
    }

    Ok(())
}
```

### Patch Bundle (RFC 6902 forward + inverse)

```rust
use serde_json::Value;
use crate::edit::patch::{Patch, PatchBundle, Rfc6902Op};
use crate::error::EngineError;

#[derive(Clone, Debug)]
pub struct ApplyOptions {
    pub enforce_left_hash: bool,
    pub enforce_right_hash: bool,
}

pub fn hash_json(v: &Value) -> String {
    format!("blake3:{}", blake3::hash(v.to_string().as_bytes()).to_hex())
}

pub fn make_patch_bundle_rfc6902(left: &Value, right: &Value) -> Result<PatchBundle, EngineError> {
    let forward_patch = json_patch::diff(left, right);
    let inverse_patch = json_patch::diff(right, left);

    let forward_ops = forward_patch.0.into_iter().map(op_from_json_patch).collect();
    let inverse_ops = inverse_patch.0.into_iter().map(op_from_json_patch).collect();

    Ok(PatchBundle {
        wire_version: "1.0".into(),
        left_hash: hash_json(left),
        right_hash: hash_json(right),
        forward: Patch::Rfc6902 { ops: forward_ops },
        inverse: Patch::Rfc6902 { ops: inverse_ops },
    })
}

fn op_from_json_patch(op: json_patch::PatchOperation) -> Rfc6902Op {
    use json_patch::PatchOperation::*;
    match op {
        Add(a) => Rfc6902Op { op: "add".into(), path: a.path.to_string(), from: None, value: Some(a.value) },
        Remove(r) => Rfc6902Op { op: "remove".into(), path: r.path.to_string(), from: None, value: None },
        Replace(r) => Rfc6902Op { op: "replace".into(), path: r.path.to_string(), from: None, value: Some(r.value) },
        Move(m) => Rfc6902Op { op: "move".into(), path: m.path.to_string(), from: Some(m.from.to_string()), value: None },
        Copy(c) => Rfc6902Op { op: "copy".into(), path: c.path.to_string(), from: Some(c.from.to_string()), value: None },
        Test(t) => Rfc6902Op { op: "test".into(), path: t.path.to_string(), from: None, value: Some(t.value) },
    }
}

pub fn apply_patch_bundle_forward(doc: &Value, b: &PatchBundle, opts: ApplyOptions) -> Result<Value, EngineError> {
    if opts.enforce_left_hash && hash_json(doc) != b.left_hash {
        return Err(EngineError::InvalidArgument("left_hash mismatch".into()));
    }
    apply_patch(doc, &b.forward)
}

pub fn apply_patch_bundle_inverse(doc: &Value, b: &PatchBundle, opts: ApplyOptions) -> Result<Value, EngineError> {
    if opts.enforce_right_hash && hash_json(doc) != b.right_hash {
        return Err(EngineError::InvalidArgument("right_hash mismatch".into()));
    }
    apply_patch(doc, &b.inverse)
}

pub fn apply_patch(doc: &Value, p: &Patch) -> Result<Value, EngineError> {
    match p {
        Patch::Rfc6902 { ops } => {
            let mut v = doc.clone();
            let patch = json_patch::Patch(ops.iter().map(op_to_json_patch).collect());
            json_patch::patch(&mut v, &patch)
                .map_err(|e| EngineError::InvalidArgument(e.to_string()))?;
            Ok(v)
        }
        Patch::Merge7396 { patch } => {
            let mut v = doc.clone();
            json_patch::merge(&mut v, patch);
            Ok(v)
        }
        Patch::Fionn { .. } => Err(EngineError::InvalidArgument("fionn patch apply not implemented yet".into())),
    }
}

fn op_to_json_patch(op: &Rfc6902Op) -> json_patch::PatchOperation {
    use json_patch::*;
    match op.op.as_str() {
        "add" => PatchOperation::Add(AddOperation { path: op.path.parse().unwrap(), value: op.value.clone().unwrap() }),
        "remove" => PatchOperation::Remove(RemoveOperation { path: op.path.parse().unwrap() }),
        "replace" => PatchOperation::Replace(ReplaceOperation { path: op.path.parse().unwrap(), value: op.value.clone().unwrap() }),
        "move" => PatchOperation::Move(MoveOperation { from: op.from.clone().unwrap().parse().unwrap(), path: op.path.parse().unwrap() }),
        "copy" => PatchOperation::Copy(CopyOperation { from: op.from.clone().unwrap().parse().unwrap(), path: op.path.parse().unwrap() }),
        "test" => PatchOperation::Test(TestOperation { path: op.path.parse().unwrap(), value: op.value.clone().unwrap() }),
        other => panic!("unsupported op {other}"),
    }
}
```

### End-to-End Apply (build template + diff + patch)

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ast::Template;
use crate::edit::plan::EditPlan;
use crate::edit::diff::TemplateDiff;
use crate::edit::patch::PatchBundle;
use crate::error::EngineError;

use crate::edit::{compile, patch, diff};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplyEditPlanResult {
    pub wire_version: String,      // "1.0"
    pub new_template: Template,
    pub patch_bundle: PatchBundle,
    pub diff: TemplateDiff,
    pub suggestions: Vec<crate::edit::plan::Suggestion>,
}

pub fn apply_edit_plan(base: &Template, plan: &EditPlan) -> Result<ApplyEditPlanResult, EngineError> {
    let new_template = compile::apply_edits(base, plan)?;

    let d = diff::diff_templates(base, &new_template);

    let left: Value = serde_json::to_value(base)?;
    let right: Value = serde_json::to_value(&new_template)?;
    let bundle = patch::make_patch_bundle_rfc6902(&left, &right)?;

    Ok(ApplyEditPlanResult {
        wire_version: "1.0".into(),
        new_template,
        patch_bundle: bundle,
        diff: d,
        suggestions: plan.suggestions.clone(),
    })
}
```

### Minimal Template Diff (starter)

```rust
use crate::ast::{Segment, Template};
use crate::edit::diff::{SegmentChange, TemplateDiff, TextHunkDiff, StringHunk};

pub fn diff_templates(a: &Template, b: &Template) -> TemplateDiff {
    let mut out = TemplateDiff { wire_version: "1.0".into(), ..Default::default() };

    let n = a.segments.len().max(b.segments.len());
    for i in 0..n {
        match (a.segments.get(i), b.segments.get(i)) {
            (Some(Segment::Var { slug: sa, .. }), Some(Segment::Var { slug: sb, .. })) if sa == sb => {}
            (Some(Segment::Var { slug, .. }), None) => out.segment_changes.push(SegmentChange::DeleteVar { at: i, slug: slug.clone() }),
            (None, Some(Segment::Var { slug, .. })) => out.segment_changes.push(SegmentChange::InsertVar { at: i, slug: slug.clone() }),
            (Some(Segment::Text { value: ta }), Some(Segment::Text { value: tb })) if ta != tb => {
                out.segment_changes.push(SegmentChange::ReplaceText { at: i, before_len: ta.len(), after_len: tb.len() });
                out.text_hunks.push(TextHunkDiff {
                    segment_index: i,
                    before: ta.clone(),
                    after: tb.clone(),
                    hunks: vec![
                        StringHunk { op: "delete".into(), text: ta.clone() },
                        StringHunk { op: "insert".into(), text: tb.clone() },
                    ],
                });
            }
            _ => {}
        }
    }

    out
}
```

### Agent Loop Example

```rust
use template_engine::ast::*;
use template_engine::edit::plan::*;
use template_engine::edit;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = Template {
        wire_version: "1.0".into(),
        segments: vec![
            Segment::Text { value: "We can come out today. Trip fee is ".into() },
            Segment::Text { value: " TBD.".into() },
        ],
    };

    let plan = EditPlan {
        wire_version: "1.0".into(),
        fork_id: "agent:proposal:123".into(),
        edits: vec![
            Edit::InsertVar {
                target: TextTarget { segment_index: 0, start: base_text_len(&base, 0), end: base_text_len(&base, 0) },
                var: VarSpec {
                    slug: "trip_fee".into(),
                    kind: VarKind::Runtime,
                    source: Some(Source::SectorConfigs),
                    schema: Some("podium:schema:money".into()),
                },
            }
        ],
        suggestions: vec![
            Suggestion::VarValue {
                suggestion_id: "sug-1".into(),
                slug: "trip_fee".into(),
                source: Source::SectorConfigs,
                schema: Some("podium:schema:money".into()),
                proposed_value: json!({"amount":"95.00","currency":"USD"}),
                status: SuggestionStatus::PendingUser,
            }
        ],
    };

    let res = edit::apply_edit_plan(&base, &plan)?;
    println!("Diff: {}", serde_json::to_string_pretty(&res.diff)?);

    let left = serde_json::to_value(&base)?;
    let right = template_engine::edit::apply::apply_patch_bundle_forward(
        &left,
        &res.patch_bundle,
        template_engine::edit::apply::ApplyOptions { enforce_left_hash: true, enforce_right_hash: false }
    )?;

    let reverted = template_engine::edit::apply::apply_patch_bundle_inverse(
        &right,
        &res.patch_bundle,
        template_engine::edit::apply::ApplyOptions { enforce_left_hash: false, enforce_right_hash: true }
    )?;

    assert_eq!(left, reverted);
    Ok(())
}

fn base_text_len(t: &Template, idx: usize) -> usize {
    match &t.segments[idx] {
        Segment::Text { value } => value.len(),
        _ => 0,
    }
}
```

### Notes on Agent Usefulness

- Two patch streams: template edits vs. config/env suggestions.
- Commit is naturally two-phase: accept template edits first, then accept suggestions as config changes.
- The patch bundle is reversible and guarded by hashes to prevent accidental application.
