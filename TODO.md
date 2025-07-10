# TODO: LLM Library Package Implementation

## Remove main.rs from LLM package (library not binary)

**Task**: Delete `/packages/llm/src/main.rs` since LLM package should be a library, not a binary. Update package structure to be library-only.

**Files to modify**: 
- Delete: `packages/llm/src/main.rs`
- Verify: `packages/llm/Cargo.toml` has no `[[bin]]` section

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## QA: Verify LLM package is properly configured as library

Act as an Objective QA Rust developer and rate the work performed previously on removing main.rs and configuring LLM as library-only package. Verify that the package structure is correct for a library and no binary artifacts remain.

## Create domain/mod.rs to expose domain modules

**Task**: Create `packages/llm/src/domain/mod.rs` to properly expose all domain modules (agent, tool, context, etc.) so they can be imported and used by the FluentAI builder.

**Files to modify**:
- Create: `packages/llm/src/domain/mod.rs`
- Update: `packages/llm/src/lib.rs` to include domain module

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## QA: Verify domain modules are properly exposed

Act as an Objective QA Rust developer and rate the work performed previously on creating domain/mod.rs and exposing domain modules. Verify that all domain objects can be properly imported and used.

## Implement real FluentAI builder with domain objects

**Task**: Replace mock implementations in `packages/llm/src/llm_builder.rs` with real implementations that use the domain objects from `./domain/`. Integrate FluentAI builder with actual Tool, Agent, Context, and other domain objects.

**Files to modify**:
- `packages/llm/src/llm_builder.rs` - Replace mocks with real domain object integration
- Import domain objects and use them in builder methods

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## QA: Verify FluentAI builder uses real implementations

Act as an Objective QA Rust developer and rate the work performed previously on implementing real FluentAI builder with domain objects. Verify that no mock implementations remain and all builder methods work with actual domain objects.

## Integrate JSON object syntax via hashbrown macro

**Task**: Implement JSON object syntax support in FluentAI builder methods using `sugars_macros::collections::hashbrown::hash_map_fn!` macro. Methods like `Tool<Perplexity>::new({"citations" => "true"})`, `.additional_params({"beta" => "true"})`, and `.metadata({"key" => "val"})` must work.

**Files to modify**:
- `packages/llm/src/llm_builder.rs` - Add hashbrown macro usage to builder methods
- Implement FnOnce closures that accept JSON object syntax
- Use `hash_map_fn!` internally to enable `{"key" => "value"}` syntax

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## QA: Verify JSON object syntax works via hashbrown macro

Act as an Objective QA Rust developer and rate the work performed previously on integrating JSON object syntax via hashbrown macro. Verify that `{"key" => "value"}` syntax works correctly in all builder methods without mocking.

## Fix compilation errors in LLM package

**Task**: Resolve all compilation errors in the LLM package. Run `cargo check --package sugars_llm` and fix any import errors, type mismatches, or other compilation issues.

**Files to modify**:
- Fix any import/export issues in `packages/llm/src/lib.rs`
- Resolve type errors in `packages/llm/src/llm_builder.rs`
- Fix dependency issues in `packages/llm/Cargo.toml`

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## QA: Verify LLM package compiles without errors

Act as an Objective QA Rust developer and rate the work performed previously on fixing compilation errors. Verify that `cargo check --package sugars_llm` passes with zero errors and warnings.

## Update examples to import FluentAI from sugars_llm package

**Task**: Update `packages/sugars-examples/src/main.rs` to import `FluentAI` from the `sugars_llm` package instead of defining it locally. Add `sugars_llm` dependency to examples package.

**Files to modify**:
- `packages/sugars-examples/src/main.rs` - Replace local FluentAI with import from sugars_llm
- `packages/sugars-examples/Cargo.toml` - Add dependency on sugars_llm package
- Remove any mock implementations from examples

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## QA: Verify examples use real LLM package

Act as an Objective QA Rust developer and rate the work performed previously on updating examples to use sugars_llm package. Verify that examples properly import and use the real FluentAI builder without any local mocks.

## Test complete flow works end-to-end

**Task**: Verify that the complete flow works: examples import FluentAI from sugars_llm, JSON object syntax works in builder methods, and the example code from README compiles and runs correctly.

**Files to verify**:
- `cargo check --package sugars_llm` passes
- `cargo check --package sugars-examples` passes
- JSON object syntax works in examples
- No mocking or simulation in any part of the flow

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## QA: Verify complete flow works without mocking

Act as an Objective QA Rust developer and rate the work performed previously on testing the complete flow. Verify that the entire system works end-to-end with real implementations, proper compilation, and functional JSON object syntax.