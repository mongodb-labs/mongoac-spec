# MongoDB Async C Driver Design Document

## Abstract

This project is the initial design specification for the new MongoDB Async C Driver.

> [!CAUTION]
> This specification is still in a drafting state!
> As a Proof of Concept, this project uses heavy LLM-assistance using Kimi K2.7 Code and DeepSeek V4 Flash.
> Details in this README.md are still being reviewed and the reference implementation under `src/libmongoac` is still
>   being audited.
> Contents above the `<!-- Audit Progress -->` marker comment in this file have been manually reviewed, audited, and
>   edited for accuracy and intent.

## Terminology

| Term | Shortname | Description |
|---|---|---|
| **Async C Driver** | mongoac | This library (providing an async C API). |
| **C Driver** | mongoc | The existing synchronous C library. |
| **BSON Library** | bson2 | The existing C BSON library (v2). |
| **mongo-c-driver** | N/A | The repository providing bson2, mongoc, and mongoac. |
| **Rust Driver** | Rust API | The `mongodb` crate. |
| **Rust FFI** | mongoac | This library (translating the Rust API into a C FFI). |

## Specification

This section describes how the initial set of features for the mongoac library are expected to be implemented.

> [!IMPORTANT]
> Ambiguities, inconsistencies, and conflicts with this initial design specification are expected during real-world
>   implementation.

### Build System

The mongoac library is defined as a new CMake subproject under `src/libmongoac` alongside `src/libbson` and
  `src/libmongoc`.
All Rust crates (analogous to C components) are built using Cargo (rustc).
Crate headers are generated using `cbindgen`, as defined by the `build.rs` file.
CMake is completely responsible for passing the environment variables and compilation flags required to build the
  mongoac library.

> [!NOTE]
> Isolating CMake configuration options to the bson and mongoc libraries is out-of-scope.

#### External Dependencies

The following executables are **new** required external dependencies or **stricter** than present mongo-c-driver
  requirements:

- `cmake`: 3.25 or newer.
- `cargo`: 1.85 or newer (2024 Edition)
- `cbindgen`: 1.85 or newer.
- `patchelf`: Linux only.
- C Compiler (header validation only): C99 or newer, see `CMakeLists.txt`.
- C++ Compiler (tests only): C++14 or newer, see `CMakeLists.txt`.

All Rust crate dependencies are automatically obtained by `cargo`.
This is similar to how CMake obtains Catch2 and `uv` obtains Python packages.

The C compiler is required for `CMAKE_VERIFY_INTERFACE_HEADERS`, but is not strictly required to build the mongoac
  library (handled entirely by Cargo).
The C++ compiler is only required to build the Catch2 test suite (same C++14 requirement as in the C++ Driver).
The stricter C/C++ toolchain and CMake version requirements are expected to be acceptable for users given the
  comparatively more-demanding Rust toolchain requirements.

> [!TIP]
> - [Why CMake 3.25?](#why-cmake-325)
> - [Why patchelf?](#why-patchelf-soname)
> - [Why list dependencies explicitly?](#why-list-dependencies-explicitly)

#### Cargo Integration

CMake defines two `INTERFACE` targets:

| Target | Type | Compile Definition |
|---|---|---|
| `mongoac::shared` | `cdylib` | *(none)* |
| `mongoac::static` | `staticlib` | `MONGOAC_STATIC` |

CMake invokes `cargo rustc` via `add_custom_command`, passing `--target-dir` inside the CMake binary directory.
The custom target directory is essential to support both single-config and multi-config CMake generators.

The environment variables `MONGOAC_BSON_SHARED_LIBRARY_FILENAME` and `MONGOAC_BSON_STATIC_LIBRARY_FILENAME` direct
  `build.rs` to use the correct linkage with the appropriate bson2 library, as detected and configured by the parent
  CMake build configuration (for consistency with how mongoc links with bson, or how mongocxx links with bsoncxx).
`build.rs` then emits `cargo:rustc-link-lib=bson2` (shared) or `cargo:rustc-link-lib=static=bson2` (static)
  accordingly.

The mongoac library links with bson2 at **link-time only**.
No bson2 library symbols are embedded in either the mongoac shared or static libraries: all symbols remain unresolved.
However, the shared library correctly records the bson2 link dependency (e.g. via ELF `NEEDED`) to inform linkers where
  and how to find the bson2 library.

To avoid unnecessarily coupling mongo-c-driver specific CMake configuration patterns to mongoac, CMake and pkg-config
  package config files are generated using the same CMake generation pattern as in the C++ Driver
  (`mongoacConfig.cmake.in`, `mongoac.pc.in`), but result in the same installation directory structure as the C Driver.

> [!NOTE]
> Unlike mongoc, mongoac does **not** require `ENABLE_STATIC` to build tests.

> [!TIP]
> - [Why link bson2 into the Rust crate?](#why-link-bson2-into-rust)
> - [Why explicit environment variables for shared vs static bson2?](#why-explicit-bson-link-vars)
> - [Why not statically embed bson2 into the cdylib?](#rejected-downstream-bson-link)

#### Build Configuration

The CMake directory structure is largely the same as the existing mongo-c-driver for source, build, and install.
However, the mongoac subproject is completely independent from the mongoc subproject: `ENABLE_MONGOC=OFF` and
  `ENABLE_MONGOAC=ON` are a valid combination of CMake options.
This impacts the `test` subdirectory (where the Catch2 test suite is implemented) the most due to complete isolation
  from existing mongoc-based test utilities and helpers, including the unified spec test runner.
Unified spec format tests are expected to follow a similar structure, potentially duplicating the list of spec test
  files under `src/libmongoac/test/json` (mirroring `src/libmongoc/test/json`).

Library file naming and versioning also mirrors existing patterns in mongo-c-driver.
However, due to the library files being generated by Cargo rather than CMake, CMake must manually create and install
  shared library symlinks.
CMake must also use `patchelf` when available (on Linux environments only, per CMake's `LINUX` variable) to correctly
  set the SONAME in the shared library file.
When `patchelf` is not found, CMake emits a warning, but does not error.

The `OUT_DIR` env var specified by CMake ensures Cargo uses a directory consistent with the current CMake build
  configuration and regardless of the CMake generator being used (via CMake generator expressions).
Configuration options are forwarded to `build.rs` using environment variables (e.g. `MONGOAC_CMAKE_BUILD_TYPE` and
  `MONGOAC_LIBRARY_TYPE`).

> [!TIP]
> - [Why patchelf?](#why-patchelf-soname)
> - [Why skip cbindgen outside CMake?](#why-build-rs-warn)

#### Crate Headers

Header files generated via `cbindgen` are referred to as "crate headers" by this specification.

The `build.rs` crate (automatically invoked by Cargo) is responsible for invoking `cbindgen` as well as generating the
  `config.rs` crate according to the specific build configuration being requested by CMake.
Two headers are defined manually: `export.h` (a simplified equivalent to the export header generated by CMake) and
  `version.h` (declaring constants dependent on CMake and functions dependent on `config.rs`).
All other mongoac headers are generated via `cbindgen`.

> [!TIP]
> - [Why CMake for version.h instead of cbindgen?](#why-cmake-version-header)

#### Documentation

API documentation for crate headers are under `src/libmongoac/doc/`, for symmetry with bson and mongoc docs.

```bash
uv run --frozen cmake -S . -B <build> -DENABLE_HTML_DOCS=ON
uv run --frozen cmake --build <build> --target mongoac-doc
```

### Test Infrastructure

Tests are organized into two layers (reflecting the two-layer FFI architecture described below):

- **Rust Tests**: executed via `cargo test`, written in Rust.
- **Catch2 Tests**: executed via CTest or Catch2 executable, written in C++.

Rust tests permit test coverage of internal API without requiring conditional exports (e.g. `*_EXPORT_CDECL_TESTING` in
  the C++ Driver) or requiring static library linkage (e.g. `test-libmongoc` requiring `ENABLE_STATIC=ON`).
However, most tests should primarily be written as Catch2 test cases in order to cover the actual public API layer where
   input validation (e.g. `safe_*!()`) and return value conversions take place.

A `PATCH_COMMAND` during `FetchContent_Declare()` adds support for registering Catch2 test cases with CTest, where a
  `TEST_CASE` has a duplicate name but unique tags (supported by Catch2, but not by `catch_discover_tests()`).

```cpp
// CTest: "mongoac/a/b/c/example"
TEST_CASE("example", "[a][b][c]") { ... }
```

Special tags (e.g. `[!serial]`, `[!mayfail]`, etc.) are excluded in the CTest unique test name.
However, they are still registered with CTest as labels (e.g. `!serial`, `!mayfail`, etc.).

> [!TIP]
> - [Why Catch2?](#why-catch2)
> - [Why dual testing layers?](#why-dual-testing-layers)
> - [Why custom discovery?](#why-custom-test-discovery)

### Rust FFI Design

#### Two Layers

The Rust FFI uses a two-layer architectural design:

- **Layer 1 (Public API):** exported symbols prefixed with `mongoac_`.
  Functions use "safety macros" to consistently handle conversions between unsafe C and safe Rust, validating only what
    is needed to enforce FFI safety (e.g. null pointer checks, UTF-8 validation, etc.) and translating return values (or
    errors) into their C representations (e.g. `Box::into_raw()`, `safe_error!()`, etc.).
  This layer is tested by Catch2 tests.
- **Layer 2 (Internal Rust):**: safe Rust implementations of corresponding public API symbols.
  Functions are defined as methods of the corresponding `struct` being operated on.
  No `unsafe` blocks are present in Layer 2: all unsafe input validation and C representation conversions are handled by
    Layer 1.
  This layer is tested by Catch2 tests (via Layer 1) and via `cargo test`.

This results in the following general pattern for a given `example.rs` crate:

```c
// example.h (generated)
typedef struct mongoac_example_t mongoac_example_t;

mongoac_example_t* mongoac_example_new();
void mongoac_example_destroy(mongoac_example_t *example);
mongoac_future_t* mongoac_example_async(const char *input, mongoac_error_t *error);
```

```rs
// example.rs

use crate::error::ErrorT;   // mongoac_error_t
use crate::future::FutureT; // mongoac_future_t

// `mongoac_example_t`: renamed by cbindgen via build.rs.
pub struct ExampleT {
  inner: mongodb::Example, // Underlying Rust API struct.
  // ...
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_example_new() -> *mut ExampleT {
  Box::into_raw(Box::new(ExampleT::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_example_destroy(example: *mut ExampleT) {
  // if !example.is_null() {
  //     unsafe { drop(Box::from_raw(example)) }
  // }
  safe_drop!(example)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_example_async(
  example: *mut ExampleT, input: *const c_char, error: *mut ErrorT
) -> *mut FutureT {
  // Layer 1: optional pointer.
  // let error = match unsafe { error.as_mut() } {
  //     Some(e) => { e.clear(); Some(e) },
  //     None    => None,
  // }
  let error = safe_optional_error_as_mut!(error);

  // Layer 1: required parameter.
  // let example = match unsafe { example.as_mut() } {
  //     Some(r) => r,
  //     None => {
  //         $crate::private::safety::invalid_argument(
  //             $error,
  //             concat!(stringify!(example), ": must not be null"),
  //         );
  //         return Default::default();
  //     }
  // }
  let example = safe_as_mut_with_error!(example);

  // Layer 1: required UTF-8 string.
  // let input = {
  //     if input.is_null() {
  //         $crate::private::safety::invalid_argument(
  //             $error,
  //             concat!(stringify!(input), ": must not be null"),
  //         );
  //         return Default::default();
  //     }
  //     match unsafe { std::ffi::CStr::from_ptr(input) }.to_str() {
  //         Ok(s) => s.to_string(),
  //         Err(_) => {
  //             $crate::private::safety::invalid_argument(
  //                 $error,
  //                 concat!(stringify!(input), ": must be valid UTF-8"),
  //             );
  //             return Default::default();
  //         }
  //     }
  // }
  let input = safe_cstr_from_ptr_with_error!(input);

  // Layer 1: error handling.
  // let future = match example.async(input) {
  //     Ok(val) => val,
  //     Err(err) => {
  //         if let Some(e) = error {
  //             *e = ::std::convert::Into::into(err);
  //         }
  //         return Default::default();
  //     }
  // }
  let future = safe_error!(example.async(input));

  // Layer 1: returning an owning pointer.
  Box::into_raw(Box::new(future))
}

// Layer 2: methods and helper functions.
impl ExampleT {
  fn new() -> ExampleT {
    // Layer 2: safe Rust implementation.
  }

  // `drop()` is unnecessary in safe Rust.

  fn async(str: String) -> FutureT {
    // Layer 2: safe Rust implementation.
  }
}
```

#### bson2

The mongoac library deliberately reuses the existing BSON C library (`bson2`) in its API.

Unfortunately, `unsafe` blocksc are required when invoking bson API (currently the only expected use of `unsafe` that
  is not in Layer 1).
A zero-sized opaque `bson_t` is required, as otherwise reimplementing the `bson_t` struct definition may lead to many
  undesirable complications such as One Definition Rule violations, redefinitions errors in cbindgen-generated headers,
  complex special-casing in `build.rs`, and more.
Instead, `u32::from_le_bytes()` and `from_raw_parts()` are used to extract the length data member (due to the lack of
  a `bson_get_len()` function); all other features are utilized using public C API functions.
Accordingly, as a **library-wide precondition**, all `bson_t*` handled by mongoac MUST contain a valid embedded BSON
  length.
This precondition is the same as what is used by the new `bsoncxx::v1` API in the C++ Driver.

`*mut bson_t` is assumed to be an owning pointer.
`*const bson_t` is assumed to be a read-only, non-owning pointer.
This convention applies both to pointers given to mongoac and pointers returned from mongoac.
This design specification proposes using return values instead of using out-parameters whenever possible, e.g.:

```c
bson_t *bson = mongoac_future_get_bson(future, error);
if (mongoac_error_code(error) != MONGOAC_ERROR_CODE_OK) { /* error handling */ }
else { use(bson); bson_destroy(bson); }
```

However, this may exclude some opportunities to use `bson_static_init()` to initialize a `*mut bson_t` out-parameter,
  which can avoid allocations and internal copying of BSON data.
We may consider extending the API in the future with `bson_non_owning()` API if the cost of (de)allocating owning
  return values is measurably resulting in significant overhead, e.g.:

```c
bson_t bson;
if (!mongoac_future_get_bson_non_owning(future, &bson, error)) { /* error handling */ }
else { use(&bson); } // Non-owning: `bson_destroy(doc)` not required.
```

#### Opaque Pointers

All structs owned and returned by mongoac are opaque, including `bson_t` objects.
Safety macros enforce not-null requirements as graceful errors returned via `mongoac_error_t *error` out-params.
Owning pointers to mongoac structs are returned using `Box::into_raw()` and destroyed by `drop(Box::from_raw(ptr))`
  within a dedicated `mongoac_*_destroy()` function using `safe_drop!(ptr)`.

All public structs are defined as `pub struct ExampleT`, which are renamed to `mongoac_example_t` by cbindgen via
  build.rs.
The `T` suffix in Rust mirrors the `_t` suffix in C and prevents ambiguity with existing Rust structs and traits (e.g.
  `ErrorT` vs. `std::error::Error`, `ClientT` vs. `mongodb::Client`, etc.).
Private structs (under `src/libmongoac/private/`) do not need to follow this naming convention.

The Rust API treats most non-options classes such as `Client`, `Database`, and `Collection` as immutable
  post-construction.
This means most public API operating on these objects are logically-const.
Therefore, with the exception of `mongoac_*_destroy()`, most functions `mongoac_example_*(example, ...)` may accept
  `const mongoac_example_t *example` when `ExampleT` corresponds to an immutable struct.
The mongoac library internally uses `Arc<Mutex<T>>` for consistency with the Rust Driver's thread-safety model.
This is conceptually analogous to `std::shared_ptr<std::pair<std::mutex, T>>` in C++.

Because (nearly) all Rust API `*Options` structs support deserialization via
  [serde](https://docs.rs/serde/latest/serde/), the initial design specification proposes consistently using a single
  `options: *const bson_t` optional (nullable) parameter for all options by default to keep the Rust FFI as "thin" as
  possible.
This avoids needing to implement a large number of `*OptionsT` structs and accessor API in the initial Rust FFI
  implementation (several hundred functions in total when accounting for all the various options structs).
The API may be extended as-needed in the future to support typed options structs by adding `*_with_options()` variants
  to the existing API (e.g. see `mongoac_client_new_with_options()` for `mongoac_client_options_t`).
This pattern is also consistent with the Rust API's use of `*_with_options()` functions.

> [!TIP]
> - [Why use a single `bson_t` for options structs?](#why-bson-options)
> - [Should unrecognized fields be warned about?](#unrecognized-bson-fields)

> [!NOTE]
> `ClientSession` is a notable exception to the "non-options structs are immutable" pattern.

<!-- Audit Progress -->

#### Input Validation

The `private/safety.rs` crate provides macros which encapsulate unsafe C-to-Rust input validation.
All safety macros are defined to avoid runtime panics, both during validation (error handling) and after validation
  (in safe Layer 2 internal Rust code).
The `*_with_error` variants ensure the optional `mongoac_error_t *error` parameter is always cleared (when not null) and
  set when an input validation error occurs.
The macros which handle string-like arguments additionally validate the string is UTF-8 for Rust API compatibility:
  this is also [a language-wide invariant](https://doc.rust-lang.org/book/ch08-02-strings.html).
These macros are expected to be used exclusively in Layer 1 (Public API) function definitions.

#### Client Options

Client options that cannot be expressed through the connection string URI are
configured via a separate opaque `mongoac_client_options_t` handle. This handle
is passed to `mongoac_client_new_with_options()` and is **not** retained after
construction.

**Fields supported in this phase:**

| Field | Setter | Type |
|---|---|---|
| Command event capture | `mongoac_client_options_set_capture_command_events(opts, bool)` | `bool` toggle (`false` default) |
| CMAP/SDAM event capture toggles | `mongoac_client_options_set_capture_cmap_events(opts, bool)` / `mongoac_client_options_set_capture_sdam_events(opts, bool)` | `bool` toggle (`false` default; **not yet wired to event handlers**) |
| `server_api` | `mongoac_client_options_set_server_api(opts, bson_t*, error)` | BSON document (`bson_t*`; `NULL` clears) |

The `server_api` document is parsed and validated at setter time. It follows a
fixed schema (`apiVersion` required; `apiStrict`, `apiDeprecationErrors`
optional) that the Rust driver deserializes via serde. Unrecognized keys are
silently ignored.

> [!TIP]
> - [Why typed ClientOptionsT when operation options use BSON?](#why-typed-client-options)
> - [Other purely programmatic fields](#deferred-client-options-fields)

#### Error Model

Errors are reported through an opaque `mongoac_error_t` out-parameter with category, code, and message fields. Four categories are defined: `MONGOAC_ERROR_CATEGORY_NONE` (no error), `MONGOAC_ERROR_CATEGORY_MONGOAC` (mongoac-internal errors), `MONGOAC_ERROR_CATEGORY_BSON` (BSON deserialization errors), and `MONGOAC_ERROR_CATEGORY_RUST` (Rust driver errors, including server errors). Three synthetic codes cover common mongoac failures (unknown category, invalid argument, runtime error). Server and Rust driver error codes pass through as raw integers. Lifecycle is `mongoac_error_new()`/`mongoac_error_destroy()`; accessors return safe defaults on `NULL` input.

**Return value convention:** Every function that accepts an `error` out-parameter returns its result value directly (not via an out-parameter). The return value doubles as a suitable default on error (0, `NULL`, etc.). The canonical way to test success vs. failure is `mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK` — the `bool` return type is not used for this purpose. A future exception may be non-owning BSON out-params (e.g., cursor document borrowing via `bson_init_static`), where an out-parameter would be required because the caller borrows rather than owns the data; the current cursor getter returns an owning `bson_t*` instead.

> [!TIP]
> - [Why opaque errors?](#why-opaque-error-handle)
> - [Why raw integer codes?](#why-raw-error-codes)
> - [Why #define macros?](#why-define-macros)
> - [Why return-value-with-error convention?](#why-return-value-with-error)

#### Async Runtime

Each `mongoac_client_t` owns a dedicated single-thread Tokio runtime, created during client construction and destroyed with the client.

`mongoac_client_new()` creates a `RuntimeT` via `RuntimeT::new()`, which internally builds
a `tokio::runtime::Runtime` with `Builder::new_current_thread().enable_all()`:
- URI parsing: blocks on `ClientOptions::parse(uri_str).await`
- Client construction: `Client::with_options(options)` runs within the runtime
- Future operations: deferred to later phases on this runtime

The runtime is stored inside `ClientT` as `RuntimeT`, which wraps the underlying
`Arc<tokio::runtime::Runtime>` together with a `progress_lock` (`Arc<Mutex<()>>`)
that serializes synchronous access.

> [!TIP]
> - [Why per-client runtime?](#why-per-client-runtime)
> - [Why current_thread?](#why-current-thread)

#### Runtime Handle

A `mongoac_runtime_t` handle can be extracted from any client (via `mongoac_client_get_runtime()`) and outlives the client because `RuntimeT` is `Clone`-derived — both handles share the same underlying `Arc<Runtime>` and `Arc<Mutex<()>>`. Runtime functions accept `NULL` gracefully.

> [!TIP]
> - [Why RuntimeT as a separate handle?](#why-runtime-t-separate)
> - [Why parking_lot?](#why-parking-lot)

#### Async Operations

Async operations return an opaque `mongoac_future_t*`. The C caller creates one via an `*_async()` operation function, drives it to completion with a `mongoac_runtime_t*` `block_on*()` function, and destroys it with `mongoac_future_destroy()`. `mongoac_future_clone()` creates an additional handle that shares the same underlying result. All functions accept `NULL` gracefully.

##### Driving Futures

`mongoac_future_t` is a read-only, cloneable receipt for an async operation. The only mutable operations on the underlying handle are performed internally by `mongoac_runtime_t` `block_on*()` functions; the C caller does not poll or wait directly on the future.

`mongoac_runtime_block_on(runtime, future, error)` drives a single future to completion by acquiring `progress_lock` and entering the Tokio runtime. It is self-contained — no external worker thread is required. `NULL` is accepted safely. The future must have been created from the same runtime (or a clone of it); otherwise an error is written to the `error` out-parameter.

`mongoac_runtime_block_on_any(runtime, futures, count, error)` drives the provided futures concurrently and returns a `const mongoac_future_t*` to the first one that resolves. The returned pointer is one of the input pointers; the future handle is not mutated. If no future resolves (e.g., all inputs are `NULL` or empty count), it returns `NULL`. Runtime association is validated for each non-`NULL` future; on mismatch an error is written to `error`.

`mongoac_runtime_block_on_all(runtime, futures, count, error)` drives all provided futures to completion. `NULL` entries in the array are skipped. Runtime association is validated for each non-`NULL` future; on mismatch an error is written to `error`.

`mongoac_runtime_make_progress()` enters the per-client Tokio runtime, yields once to other spawned tasks, and returns. It is designed for a **worker thread** that drives fire-and-forget tasks and background runtime work. A per-client `progress_lock` serializes progress calls; `make_progress()` returns `false` if another thread is already driving progress. The same lock guards synchronous `block_on*()` calls, preventing concurrent `block_on*()` on a `current_thread` runtime (whose IO/timer driver `Core` is single-owner).

`mongoac_runtime_make_progress_with_timeout(runtime, timeout_ms)` wraps the yield in `tokio::time::timeout` to bound wall-clock time.

`mongoac_runtime_wait(runtime)` blocks (parks) the calling thread until work is available on the per-client runtime, then returns without driving the runtime. It is intended for a **worker thread** that would otherwise spin between `make_progress()` calls. Work availability is signaled by `RuntimeT::spawn()` when a new task is queued; the worker thread typically follows `wait()` with `make_progress()` to advance the runtime. Does not call `make_progress()` internally. Returns no value; the only meaningful exit is work having become available. `NULL` is accepted safely and returns immediately.

`mongoac_runtime_wait_with_timeout(runtime, timeout_ms)` blocks (parks) until work is available or the wall-clock `timeout_ms` expires. Returns `true` if work became available, `false` on timeout. Uses the same condvar-backed signal as `wait()`. `NULL` is accepted safely and returns `false`.

`mongoac_runtime_request_stop(runtime)` signals any thread parked on `mongoac_runtime_wait()` or `mongoac_runtime_wait_with_timeout()` for this runtime to wake and exit. It sets a persistent stop flag and notifies all waiters. A worker thread typically checks `mongoac_runtime_stop_requested()` after each `wait()` to decide whether to stop driving the runtime. `NULL` is accepted safely. The stop request is not automatically cleared; a new runtime handle is required if the caller wants to resume worker threads after stopping.

`mongoac_runtime_stop_requested(runtime)` returns `true` if `mongoac_runtime_request_stop()` has been called on this runtime, otherwise `false`. `NULL` is accepted safely and returns `false`.

**Thread-safety model:** All `mongoac_future_t` functions (`clone`, `is_ready`, `get_*`) are read-only and thread-safe across clones of the same underlying future. `mongoac_future_destroy()` must be the last call on a given handle; concurrent `destroy` with any other operation is not safe. `mongoac_runtime_t` `*mut` functions (`make_progress*`, `block_on*`, `wait*`, `request_stop`) are **NOT** thread-safe on the same runtime handle or its clones.

> [!TIP]
> - [Why timeout granularity?](#why-timeout-granularity)

##### Result Extraction

After `is_ready()` returns `true` or a `block_on*()` function has driven the future to completion, the caller extracts the result using a typed getter matching the operation's result category. Each getter returns the result value directly and writes error details to the `error` out-parameter. To distinguish a real result from a default/sentinel value, the caller checks `mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK`. Calling a getter with a mismatched result type sets `error` to `MONGOAC_ERROR_CODE_RUNTIME_ERROR` and returns a default value. Calling a getter before the future is ready sets `error` to `MONGOAC_ERROR_CODE_RUNTIME_ERROR` and returns a default value.

##### Fire-and-Forget

Operations that do not require a result are spawned onto the per-client runtime (via `RuntimeT::spawn()`) without returning a future handle. The C side cannot await or cancel them.

> [!TIP]
> - [Why single opaque future?](#why-single-opaque-future)
> - [Why are all driving operations on RuntimeT?](#why-driving-on-runtime)
> - [Why is FutureT cloneable?](#why-future-cloneable)
> - [Why single-yield make_progress?](#why-single-yield-make-progress)
> - [Why defer cancellation?](#why-defer-cancellation)

### Supported Features

> [!NOTE]
> This section documents both features that are **implemented** in the current proof-of-concept and features that are **planned** for later phases. Subsections explicitly state their current status where applicable.

#### Connection Strings (URI)

Mongoac accepts connection strings directly in `mongoac_client_new()`. There is no separate URI type.

##### Construction

`mongoac_client_new(uri_string, error)` and `mongoac_client_new_with_options(uri_string, options, error)` accept a null-terminated UTF-8 connection string, an optional `mongoac_client_options_t*` (pass `NULL` for defaults), and an optional error out-parameter. They return an opaque handle on success, `NULL` on failure. Construction **blocks** the caller for URI parsing (including DNS SRV/TXT for `mongodb+srv://`) but does not connect to the server.

> [!TIP]
> - [Why no separate URI type?](#why-no-uri-type)
> - [Why block for DNS?](#why-client-new-blocks)

##### URI Options

URI options are parsed by `ClientOptions::parse()`. All supported options configure the Rust driver's internal behavior; no C API exposes the runtime state. Supported categories include DNS seedlist, SRV polling, SDAM, server selection, read preference, read concern, compression, load balancers, retryable reads/writes, backpressure, connection pool sizing, auth (SCRAM, X509, GSSAPI, PLAIN, AWS, OIDC), and write concern. Unsupported options (`waitQueueTimeoutMS`, `serverSelectionTryOnce`) are silently ignored. `socketTimeoutMS` is rejected by the Rust driver.

> [!TIP]
> - [Why no URI option getters?](#why-no-uri-getters)
> - [Why are connection-string features URI-only?](#why-connection-string-features-uri-only)
> - [Why no callback-based auth API?](#why-no-callback-auth)

#### Client Metadata

The initial connection handshake is performed automatically by the Rust driver. mongoac does not expose `hello` or legacy `isMaster` commands to C callers.

The handshake metadata sent to the server contains:

| Field | Source | Notes |
|---|---|---|
| `client.application.name` | URI `appName` option | Maps to Rust `ClientOptions::app_name` |
| `client.driver.name` | Rust driver + mongoac | Rust base name is `"mongo-rust-driver"`; mongoac appends `"mongoac"` |
| `client.driver.version` | Rust driver + mongoac | Rust base version + mongoac version |
| `client.os.*` | Rust driver | Detected from `std::env::consts` |
| `client.platform` | Rust driver + mongoac | Rust platform string + `\|`-delimited C build metadata (`<build><link>`) |
| `client.env.*` | Rust driver | Detected from environment variables |

mongoac injects C driver identity into the handshake by setting `ClientOptions::driver_info` during `mongoac_client_new()`. The metadata uses the `|` delimiter required by the [Driver Handshake spec](https://github.com/mongodb/specifications/blob/master/source/mongodb-handshake/handshake.md). A two-character suffix is appended to the Rust `platform` string encoding build type (`d`/`r`/`u`) and linkage (`h`/`t`).

C callers may optionally append wrapping-library metadata via `mongoac_client_append_metadata(client, name, version, platform, error)`.

> [!NOTE]
> `mongoac_client_append_metadata()` validates immediate FFI safety (non-null client) and UTF-8 encoding for each non-`NULL` string argument. The [Driver Handshake spec](https://github.com/mongodb/specifications/blob/master/source/mongodb-handshake/handshake.md) requires `name` to be present, rejects `|` in driver-info strings, and limits the metadata document to 512 bytes; these spec-level checks are **delegated to the Rust driver**, not the FFI layer.
>
> [!TIP]
> - [Why append C build metadata to the platform field?](#why-build-platform-metadata)

#### Event API

Events use an **index-based** API rather than C callbacks: each implemented category has `count`, `get`, and `clear` functions polling a client-owned `VecDeque` buffer. The `get` functions return an owning `*mut bson_t` via `bson_new_from_data`; the caller must `bson_destroy()` the returned pointer. Ring buffer storage avoids the O(N) `memmove` that `Vec` would require, but `clear(N)` is still O(N) due to per-element drop.

One event category is currently implemented — **command** events. Capture toggles for **CMAP** (connection pool) and **SDAM** (server discovery and monitoring) are present on `mongoac_client_options_t` but reserved for future use (see [client options](#client-options)). All capture toggles default to `false` and must be explicitly enabled before client construction.

> [!NOTE]
> The PoC implements only command event capture for reference. The CMAP/SDAM capture toggles on `mongoac_client_options_t` are accepted but are **not yet wired to event handlers** in the current implementation.

Events are the Rust driver's serde-serialized event types, returned as BSON documents without transformation. The Rust driver uses `#[serde(untagged)]` on its `CommandEvent` and `SdamEvent` enums — serialized BSON documents contain no type-discriminant field. C callers distinguish event types by inspecting the presence of BSON document fields that are unique to each variant (e.g., `command` for `CommandStartedEvent` vs. `durationMS` for `CommandSucceededEvent` vs. `failure` for `CommandFailedEvent`).

**Known spec-level divergence:** Four SDAM event types (opening/closed) omit `topologyId` from their serialized BSON due to `#[serde(skip)]` on the Rust driver structs. Two events (`ServerDescriptionChanged`, `TopologyDescriptionChanged`) include it. The three heartbeat events (`ServerHeartbeatStarted`, `ServerHeartbeatSucceeded`, `ServerHeartbeatFailed`) have no `topologyId` field at all.

> [!TIP]
> - [Why index-based events?](#why-index-based-events)
> - [Why ring buffer storage?](#why-ring-buffer-storage)
> - [Why is event capture opt-in?](#why-capture-all-events-by-default)
> - [Why owning event document access?](#why-owning-event-doc-access)
> - [Why pass through Rust driver event shapes?](#why-pass-through-event-shapes)
> - [Why not callback-based events?](#rejected-callback-events)

#### Server Discovery, Selection & Operations

Server discovery, selection, retry, client backpressure, and connection resilience are handled internally by the Rust driver. C callers configure them through URI options and observe only operation-level outcomes.

##### Server Discovery and Monitoring

SDAM runs inside the Rust driver. mongoac does not expose topology state or server descriptions.

##### Server Selection

The Rust driver selects a server automatically for every operation. Client-specific options (`serverSelectionTimeoutMS`, `localThresholdMS`) are URI-only. Non-client-specific options (`readPreference`, `maxStalenessSeconds`, `readPreferenceTags`) map to `SelectionCriteria` and can be configured at any level by passing a `bson_t*` document with fields `mode`, `tagSets`, and `maxStalenessSeconds`. These options are deserialized when creating database/collection handles, but no CRUD operations currently consume them.

> [!TIP]
> - [Why are client-specific server selection options URI-only?](#why-uri-only-client-server-selection)
> - [Why are database/collection/operation read preferences passed as BSON documents?](#why-bson-read-preference)

##### Retryable Reads & Writes

Retryable reads and writes are enabled by default and controlled by URI options `retryReads` and `retryWrites`. The Rust driver automatically retries eligible operations once. No per-operation retry flags are exposed.

##### Connection Resilience (Step-Down)

The Rust driver preserves connections across replica set step-downs on wire version 8+. This requires no C API.

> [!TIP]
> - [Why defer typed read-preference structs?](#deferred-typed-read-preference)

#### Enumerate Databases

Two client-level async operations, distinguished by result format:

- **`list_databases`** returns a BSON array of `DatabaseSpecification` documents (`{ name, sizeOnDisk, empty, shards? }`).
- **`list_database_names`** returns a BSON array of name strings — both via `mongoac_future_get_bson()`.

Options (`authorizedDatabases`, `comment`, `filter`) as `*const bson_t` deserialized into `ListDatabasesOptions`; `NULL` = defaults. `nameOnly` is not a valid option — the Rust driver determines it internally per entry point, so two separate C functions avoid ambiguity. Targets `admin`; runs on primary. `totalSize` is not exposed.

#### Enumerate Collections

> [!NOTE]
> Not yet implemented in the current proof-of-concept. The `mongoac_cursor_t` handle exists, but no database operation currently returns it.

Two database-level async operations are planned, following the same result-type split as enumerate databases:

- **`list_collections`** will return a cursor (`mongoac_cursor_t`) with `*const bson_t` views of `CollectionSpecification` documents. The `type` field is a string (no dedicated C enum).
- **`list_collection_names`** will return a BSON array of name strings.

Options (`filter`, `batchSize`, `comment`, `authorizedCollections`) as `*const bson_t` deserialized into `ListCollectionsOptions`; `NULL` = defaults. `nameOnly` is not a valid option (same rationale as enumerate databases). `authorizedCollections` only affects `list_collection_names`.

#### Read Concern & Write Concern

> [!NOTE]
> Not yet implemented in the current proof-of-concept. `DatabaseOptions` can be deserialized from a `bson_t*`, but no operation currently consumes read concern or write concern settings.

Read concern and write concern will follow the [Options Deserialization](#why-bson-options) pattern: passed as BSON fields within the `*const bson_t options` parameter. Key casing matches the containing options struct — `camelCase` for most structs, `snake_case` for `DatabaseOptions` (both accepted due to FFI normalization). `NULL` (or omitting the field) inherits from the parent level.

> [!WARNING]
> **Empty ReadConcern for server-default reset** is not currently expressible — see [Rejected Ideas](#rejected-empty-read-concern-hack).

> [!TIP]
> - [Why use BSON documents for read/write concern?](#why-bson-options)
> - [Why defer typed concern structs?](#deferred-typed-read-preference)

<a id="crud-operations"></a>
#### CRUD Operations

> [!NOTE]
> Not yet implemented in the current proof-of-concept. The `mongoac_cursor_t` type and the async/runtime machinery exist, but no CRUD functions are exposed.

CRUD operations will follow the general async pattern with these conventions:

- **Session parameter:** Nullable `mongoac_client_session_t *session` (second param; `NULL` = implicit). See [Sessions](#sessions).
- **Sequence parameters:** `insert_many` and `aggregate` will accept C arrays of `bson_t*` with explicit count.
- **Result BSON encoding:** `camelCase` field names per CRUD spec.
- **Cursor:** Single `mongoac_cursor_t` type for `find`, `aggregate`, `run_cursor_command`, wrapping `Cursor<T>` or `SessionCursor<T>`. Session embedded; iteration functions have no session parameter. The cursor implementation currently returns an owning `bson_t*` copy of the current document via `mongoac_cursor_get_document()`; a future non-owning view variant may be added.
- **Cursor iteration:** Sync `cursor_next()` (blocks). Async variants return a future via `cursor_next_async()`. Document retrieval after async resolution via `cursor_get_document()`. A future `cursor_next_with_timeout()` may be added for tailable cursors.
- **Cursor lifecycle:** `mongoac_cursor_destroy()` will trigger `killCursors` via `AsyncDropToken`; call `make_progress()` to flush pending killCursors.
- **Sync find:** `mongoac_collection_find()` will return a cursor directly via `runtime.block_on()`.
- **Per-getMore options:** `batchSize` and `maxTimeMS` fixed at cursor creation.
- **Deferred (Database-level):** `aggregate` on `Database` and client-level `bulkWrite` (MongoDB 8.0+). `run_command` and `run_cursor_command` will be provided as database-level operations (see [Run Command](#run-command-database-level)).

> [!TIP]
> - [Why a single `mongoac_cursor_t` type?](#why-single-cursor-type)

##### Run Command (Database-level)

`run_command` and `run_cursor_command` are database-level operations on `mongoac_database_t`, following the same async and options patterns as collection-level CRUD.

Contracts: non-retryable; no `readConcern`/`writeConcern`; read preference follows `SelectionCriteria`; `$db` and Stable API fields set automatically; session is a dedicated pointer parameter (not a BSON field); `run_cursor_command` reuses `mongoac_cursor_t`.

##### Cursor Advance Execution Model

The cursor is backed by the Rust driver's `Cursor<T>` (implicit session) or `SessionCursor<T>` (explicit session), wrapped in the `mongoac`-internal `CursorT`. `advance()` has a fast path (buffer has documents — resolves in a single poll with no yield) and a slow path (buffer empty — issues a `getMore` command, yielding at the TCP send/receive boundary). Alternative first-yield points include server topology changes, connection pool wait, or registered event handlers.

> [!TIP]
> - [Why are all driving operations on RuntimeT?](#why-driving-on-runtime)
> - [Why single-yield make_progress?](#why-single-yield-make-progress)
> - [Why timeout granularity?](#why-timeout-granularity)

#### Collation

> [!NOTE]
> Not yet implemented in the current proof-of-concept. No CRUD or index operations are exposed.

Collation will be passed as a BSON sub-document inside existing `*const bson_t options` parameters. The BSON document follows the Rust `Collation` struct fields (`locale` required, plus optional `strength`, `caseLevel`, `caseFirst`, `numericOrdering`, `alternate`, `maxVariable`, `normalization`, `backwards`). Collation will be supported on all CRUD operations except `estimated_document_count`, `insert_one`, and `insert_many`. mongoac will not check `maxWireVersion < 5` for collation; the Rust driver handles server incompatibility.

> [!TIP]
> - [Why no maxWireVersion check?](#why-no-maxwireversion-check)
> - [Why are opcode-based writes not a concern?](#why-opcode-non-issue)

#### Collection Management

> [!NOTE]
> Partially implemented in the current proof-of-concept. Only `create_collection_async`, `drop_collection_async`, and `drop_database_async` are exposed; the remaining behavior is planned.

Mongoac provides async create and drop operations for collection lifecycle management. All resolve to void via `mongoac_future_get_void()`.

- **`create_collection_async`** — database-level, accepts `CreateCollectionOptions` as a BSON document (capped, validator, `viewOn`/`pipeline` for views, collation, timeseries, clusteredIndex, `encryptedFields`, and other `CreateCollectionOptions` fields). Only the async form is available.
- **`drop_collection_async`** — collection-level, accepts `DropCollectionOptions` as BSON (currently accepted but unused in the initial implementation).
- **`drop_database_async`** — database-level, drops the entire database.

`rename_collection` is not exposed — the Rust driver has no dedicated API. View creation uses `create_collection_async` with `viewOn`+`pipeline`; no separate create-view function. All operations are async-only.

<a id="sessions"></a>
#### Sessions

> [!NOTE]
> Partially implemented in the current proof-of-concept. Only `mongoac_client_start_session()`, `mongoac_client_start_session_async()`, and `mongoac_client_session_destroy()` are exposed. Sessions can be passed to `list_databases` and `list_database_names`, but transaction accessors and causal-consistency accessors are not yet implemented.

Session support follows the [Driver Sessions specification](https://github.com/mongodb/specifications/blob/master/source/sessions/driver-sessions.rst). The Rust driver manages server session lifetime internally; the FFI layer exposes explicit session handles for C callers.

`mongoac_client_start_session()` (synchronous, via `block_on()`) creates a session and returns the handle directly. An async variant `mongoac_client_start_session_async()` is also provided as part of the public API. Session options are deserialized via the [standard BSON option pattern](#why-bson-options) into Rust's `SessionOptions`. Validation (`causal_consistency` + `snapshot` conflict) is delegated to the Rust driver. Sessions are destroyed with `mongoac_client_session_destroy()`, dropping the backing `ClientSession` which returns the server session to the pool. If a transaction is in-progress at destroy time, the Rust driver's `Drop` impl fires an async abort task unawaited (matching Rust driver conventions).

The session type wraps `Arc<tokio::sync::Mutex<ClientSession>>`, providing thread safety across the two-thread polling model. All session access — both async spawned tasks and synchronous accessors — goes through the mutex. Multiple tasks queued for the same session yield on contention via `lock().await`; no deadlock occurs.

Planned synchronous accessors (`get_id`, `get_cluster_time`, `get_snapshot_time`, `advance_cluster_time`, `advance_operation_time`) will acquire the mutex via `blocking_lock()`. The Rust driver auto-populates `snapshot_time` from the server response after the first find, aggregate, or distinct operation on a snapshot session — the server picks a time (or returns `cursor.atClusterTime`), and the driver stores it in the session so all subsequent reads use the same snapshot.

Every CRUD operation accepting an explicit session will take a nullable `mongoac_client_session_t*` as its second parameter. `NULL` selects an implicit session. Cursor-creating operations will clone the `Arc` into the cursor so the session outlives the user's handle.

<a id="transactions"></a>
##### Transactions

> [!NOTE]
> Not yet implemented in the current proof-of-concept. No transaction functions are exposed.

Transaction support will follow the [Driver Transactions specification](https://github.com/mongodb/specifications/blob/master/source/transactions/transactions.rst). Transactions build on Driver Sessions (minimum server 4.0 for replica sets, 4.2 for sharded clusters).

Each transaction operation (`start_transaction`, `commit_transaction`, `abort_transaction`) will be provided in two forms: async (`*_async()`) returning a `mongoac_future_t*`, and sync (no suffix) blocking via `runtime.block_on()` with `progress_lock`. The sync variants must not be called from within a `make_progress()` context, matching the sync session accessor convention.

`TransactionOptions` will use the standard BSON deserialization pattern, supporting `readConcern`, `writeConcern`, `readPreference` (as a `SelectionCriteria` sub-document), and `maxCommitTimeMS` (`timeoutMS` is [deferred](#deferred-transaction-timeoutms)). `NULL` for options uses session-level defaults set via `defaultTransactionOptions` at session creation; per-call options override those defaults — the Rust driver handles the inheritance chain.

Transaction state machine validation (`None → Starting → InProgress → Committed → Aborted`) will be delegated to the Rust driver, which detects invalid transitions synchronously and propagates them through the error out-parameter. Error labels (`"TransientTransactionError"`, `"UnknownTransactionCommitResult"`) are accessible via `mongoac_error_has_label()`, which delegates directly to the Rust driver's `contains_label()` on the preserved original error.

The Rust driver's `and_run()` retry-loop convenience will not be exposed; C callers will implement their own retry logic around the explicit API. If a session is destroyed while a transaction is `InProgress`, the Rust driver's `Drop` impl fires a fire-and-forget async abort task — callers that need a clean abort should call `abort_transaction` explicitly before `destroy()`.

<a id="causal-consistency"></a>
##### Causal Consistency

> [!NOTE]
> Not yet implemented in the current proof-of-concept. No session accessors are exposed.

Causal consistency will follow the [Driver Causal Consistency specification](https://github.com/mongodb/specifications/blob/master/source/causal-consistency/causal-consistency.rst). It is **enabled by default** for explicit sessions (via `causalConsistency: true` in session options) and **not available** for implicit sessions. Causal consistency and snapshot reads are mutually exclusive — validation is delegated to the Rust driver.

The Rust driver tracks `operationTime` from every server response (including errors) and injects `afterClusterTime` into the `readConcern` of subsequent causally-consistent commands. Cluster time (`$clusterTime`) gossipping is fully automatic.

Synchronous accessors (`get_operation_time`, `advance_operation_time`, `get_cluster_time`, `advance_cluster_time`, `get_causal_consistency`) will exist for cross-session token propagation and acquire the session mutex via `blocking_lock()`.

> [!NOTE]
> **Known upstream limitation for `get_causal_consistency`:** `ClientSession::causal_consistency()` is `pub(crate)` in the Rust driver — a historical accident, not deliberate. Implementation will require either an upstream PR to make the getter `pub`, or storing the resolved value at session creation time. The other four accessors delegate to already-public `ClientSession` methods.

> [!NOTE]
> **Known limitation:** Due to an upstream Rust driver gap, `afterClusterTime` is not sent on write commands in causally-consistent sessions outside transactions. The `operationTime` from write responses is still captured, so subsequent reads carry the correct value, but the server cannot enforce causal ordering via oplog waiting on writes. This cannot be fixed in the FFI layer.

Explicit sessions are required for causal consistency — operations without a session parameter are non-causally-consistent.

## Rationale

> [!IMPORTANT]
> This section documents **why** specific design decisions were made.

### Dependencies

<a id="why-list-dependencies-explicitly"></a>
#### Why list dependencies explicitly?

Documenting the mongoac-specific and stricter base build requirements up-front lets users verify they have everything needed beyond or above the base `mongo-c-driver` toolchain. It also clarifies which components are "bring your own" versus fetched automatically (e.g., Catch2 via `FetchContent`, Rust crates via Cargo).

<a id="why-not-bundle-cbindgen-patchelf"></a>
#### Why not bundle cbindgen or patchelf?

cbindgen and patchelf are build-time tools with their own release cadences and platform availability. Bundling them would add vendoring overhead, delay security fixes, and conflict with system package managers. Requiring the host to provide them is consistent with normal distribution practice.

### Build System

<a id="why-cmake-325"></a>
#### Why CMake 3.25?

The following CMake features are newer than the current 3.15+ requirement:

- `cmake_path()` for filename extraction and path manipulation. (3.20)
- Generator expressions in `add_custom_command(OUTPUT)` and `add_custom_command(BYPRODUCTS)` (3.20)
- `target_sources(FILE_SET HEADERS)` (3.23)
- `CMAKE_VERIFY_INTERFACE_HEADER_SETS` (3.24)
- `FetchContent_Declare(... SYSTEM)` (3.25)

<a id="why-build-rs-warn"></a>
#### Why skip cbindgen outside CMake?

`MONGOAC_INCLUDE_DIR` is set by CMake and is only needed for cbindgen output. Treating its absence as an error would block `cargo check`, rust-analyzer, and other Rust-only tooling. Skipping gracefully preserves the separation between Rust compilation and C header generation.

<a id="why-cmake-version-header"></a>
#### Why CMake for version.h?

Version macros (`MONGOAC_VERSION_MAJOR`, etc.) cannot be defined from Rust.
`build.rs` is only responsible for defining **internal** constants as-minimally-needed to implement exported functions.
Using `configure_file()` follows the same pattern used by mongo-c-driver and ensures CMake is the source of truth
  without overcomplicating `build.rs`.

<a id="why-link-bson2-into-rust"></a>
#### Why link bson2 into the Rust crate?

Reusing bson2 avoids reimplementing `bson_t` layout in Rust. Dynamic linking over static embedding prevents duplicate bson2 symbol instances in multi-component processes (e.g., when libmongoc is also loaded), avoiding one-definition-rule violations for `bson_t` internals.

<a id="why-explicit-bson-link-vars"></a>
#### Why explicit environment variables for shared vs static bson2?

`build.rs` cannot know which crate type or bson2 variant CMake produced, so CMake passes the expected library explicitly as an absolute path, avoiding ambiguity when both variants are present in the same directory.

<a id="why-patchelf-soname"></a>
#### Why `patchelf`?

Cargo currently does not support setting custom SONAME for cdylib: see [rust-lang/cargo#5045](https://github.com/rust-lang/cargo/issues/5045).

On Linux, CMake owns packaging, versioning, and install rules. Using `patchelf` as a post-link step keeps the SONAME fix in the packaging layer without touching the Rust crate or Cargo.

Attempting to handle this within the `build.rs` script would require detecting platform shared-library naming
  conventions from the bson2 filename, then emitting `cargo:rustc-cdylib-link-arg` to pass `-Wl,-soname,...` to
  the linker.

### Test Infrastructure

<a id="why-catch2"></a>
#### Why Catch2?

Catch2 provides CMake integration via `catch_discover_tests`, standard `TEST_CASE` macros, and native CTest parallelization. It is the natural choice for a modern C++ test suite.

> [!TIP]
> - [Why not mongoc's TestSuite?](#rejected-mongoc-testsuite)

<a id="why-custom-test-discovery"></a>
#### Why custom discovery?

Catch2's `catch_discover_tests()` permits duplicate `TEST_CASE` names with different tags, but does not register these
  as unique tests in CTest.
Instead, only one test of many tests are registered with CTest, with the other tests silently discarded.

Rather than forcing unique `TEST_CASE` names (conflicting with native Catch2 design patterns), patching the internal
  `CatchAddTest.cmake` file to support unique test registration with tags is the most direct approach with minimal
  complexity.

<a id="why-dual-testing-layers"></a>
#### Why dual testing layers?

Rust tests exercise internal logic without cbindgen/C compilation overhead. C++ tests validate the public ABI: header syntax, opaque-pointer contracts, linking, and behavior visible to C callers.

### Rust FFI Design

<a id="why-bson-options"></a>
#### Why use a single `bson_t` for options structs?

Because (nearly) all Rust API `*Options` structs support deserialization via
  [serde](https://docs.rs/serde/latest/serde/), the initial design specification proposes consistently using a single
  `options: *const bson_t` optional (nullable) parameter for all options by default to keep the Rust FFI as "thin" as
  possible.
This avoids needing to implement a large number of `*OptionsT` structs and accessor API in the initial Rust FFI
  implementation: over a hundred functions in total when accounting for all the various options structs whose accessors
  must be kept in sync with every individual options field.
The API may be extended as-needed in the future to support typed options structs by adding `*_with_options()` variants
  to the existing API (e.g. see `mongoac_client_new_with_options()` for `mongoac_client_options_t`).
This pattern is also consistent with the Rust API's use of `*_with_options()` functions.

> [!IMPORTANT]
> BSON deserialization via serde automatically handles mapping `camelCase` BSON document fields to `snake_case` fields
>   in options structs.
> However, as a notable exception, `DatabaseOptions` is currently missing `#[serde(rename_all = "camelCase")]`.
> The Rust FFI will need to implement its own support for mapping `camelCase` BSON document fields to `DatabaseOptions`
>   fields for consistency.

<a id="why-typed-client-options"></a>
#### Why typed `ClientOptionsT` when operation options use `bson_t`?

Programmatic client options not settable via URI are few (three event toggles,
`server_api`) and stable, so a typed handle with named setters is more
discoverable and avoids a serde round-trip for trivial booleans. `server_api`
takes a `bson_t*` because its fixed schema is already serde-deserializable,
making a parallel C struct unnecessary.

<a id="why-opaque-error-handle"></a>
#### Why opaque errors?

Opaque handles provide ABI stability (internal layout can change without breaking callers) and avoid truncation — unlike mongoc's 504-byte inline `bson_error_t`, a heap-allocated `CString` supports arbitrary-length messages.

> [!TIP]
> - [Why not Box<dyn Error>?](#rejected-box-dyn-error)

<a id="why-raw-error-codes"></a>
#### Why raw integer codes for server/driver errors?

Server and Rust driver codes are external values not owned by mongoac. Only a small, stable taxonomy of synthetic mongoac errors (invalid argument, NULL pointer, etc.) justifies named constants.

<a id="why-define-macros"></a>
#### Why #define macros instead of C enums?

`#define` gives explicit `int32_t` width (C99 enum underlying types are compiler-chosen) and additive constant safety (does not change type size). The named values are backed by a Rust `#[repr(i32)]` enum with explicit discriminants, so duplicate values are caught at compile time before the C header is generated. cbindgen also emits simpler, more portable headers with `#define`.

> [!TIP]
> - [Why not C enums?](#rejected-c-enums)

<a id="why-return-value-with-error"></a>
#### Why return-value-with-error convention?

Returning the result value directly avoids forcing callers to declare extra variables. The `error` out-parameter carries structured diagnostics that a `bool` return cannot. The return value doubles as a safe default on error (0, `NULL`), so accessing it unconditionally is safe — the caller checks the error code only to distinguish a real result from a sentinel. Non-owning BSON out-params are the one exception: `bson_init_static` initializes an existing `bson_t` in place, requiring a pointer rather than a return value.

> [!TIP]
> - [Error-handling transparency trade-off](#error-handling-transparency)

<a id="why-per-client-runtime"></a>
#### Why per-client runtime?

Isolating each client's async context avoids `Send`/`Sync` requirements on futures crossing the FFI boundary. It also simplifies resource accounting: when the client is destroyed, its runtime is destroyed with it.

<a id="why-current-thread"></a>
#### Why current_thread?

A `current_thread` runtime is sufficient for a single-client context and avoids spawning an OS thread pool per client. The Rust driver's internal connection pool and topology monitoring already handle concurrency.

<a id="why-driving-on-runtime"></a>
#### Why are all driving operations on RuntimeT?

Mutable operations on a `FutureT` (poll, wait, block-on) require `Pin<&mut Self>` on the underlying Tokio `JoinHandle` chain and must serialize with the `current_thread` runtime's single-owner driver core. Centralizing all such operations on `RuntimeT` lets `FutureT` be a read-only, cloneable receipt. C callers cannot accidentally poll or block on the same future from multiple threads, and the thread-safety model collapses to "read-only getters are safe, everything else is runtime-owned."

> [!TIP]
> - [Why not waker-based integration?](#rejected-waker-integration)

<a id="why-future-cloneable"></a>
#### Why is FutureT cloneable?

Cloning lets multiple C contexts hold a reference to the same async result without coordinating ownership. Because the underlying `FutureValue` is reference-counted, all clones observe the same resolved state and share the same typed result. Cloning is cheap: it increments an `Arc` and a `RuntimeT` reference count.

> [!TIP]
> - [Why not separate future types?](#rejected-separate-futures)

<a id="why-single-opaque-future"></a>
#### Why single opaque future?

A single handle type minimizes API surface. The typed-getter pattern is idiomatic in C libraries, and Rust dispatches internally via an enum not exposed in the C ABI.

> [!TIP]
> - [Why not separate future types?](#rejected-separate-futures)

<a id="why-single-yield-make-progress"></a>
#### Why single-yield make_progress?

Yielding exactly once per call advances the runtime without monopolizing the worker thread. The caller controls pacing by choosing how often to call `make_progress()` from the worker thread. For futures with results, `block_on*()` is used instead; `make_progress()` is reserved for fire-and-forget tasks and background runtime work.

> [!TIP]
> - [Why not yield until idle?](#rejected-exhaustive-progress)

<a id="why-runtime-wait"></a>
#### Why runtime wait?

A dedicated worker thread that repeatedly calls `make_progress()` would otherwise spin or sleep in a busy loop between yields. A condvar-backed wait lets the worker thread park until a new task is actually queued via `RuntimeT::spawn()`, then wake only when there is work to advance. The timeout variant keeps the same non-spinning behavior while allowing the caller to bound blocking time.

<a id="why-timeout-granularity"></a>
##### Why timeout granularity?

On a `current_thread` runtime, Tokio's scheduler checks the timer wheel only after every `event_interval` spawned-task polls (default 61). A non-yielding spawned task blocks the single thread — the timer cannot advance until the task completes. As a result, `timeout_ms` is a **soft upper bound**: actual elapsed time can exceed it by up to `event_interval` polls' worth of execution.

This granularity is inherent to the single-threaded, non-preemptive model. Callers requiring tighter bounds should reduce `timeout_ms` proportionally.

<a id="why-defer-cancellation"></a>
#### Why defer cancellation?

Cancellation is deferred because opaque handles allow it to be added later as an ABI-compatible extension.

> [!TIP]
> - [Why not add cancellation now?](#rejected-cooperative-cancellation)

<a id="why-runtime-t-separate"></a>
#### Why RuntimeT as a separate handle?

`RuntimeT` is `Clone`-derived — both handles share the same `Arc<Runtime>` and `progress_lock` via reference-counting, so a handle extracted from a client outlives the client. Event-loop integrations and multi-threaded callers may need to drive progress from a context that does not own the client. Decoupling runtime from client avoids requiring the full `ClientT` where only yielding is needed.

<a id="why-arc-runtime"></a>
#### Why Arc<Runtime> instead of borrowed references?

`RuntimeT` encapsulates an `Arc<Runtime>` (and `Arc<Mutex<()>>` for `progress_lock`), so both handles reference-count the same underlying allocations. C callers do not understand Rust borrow semantics. A `RuntimeT` must remain valid even if the originating `ClientT` is destroyed, so `Arc<Runtime>` is required.

> [!TIP]
> - [Why not borrowed references?](#rejected-borrowed-references)

<a id="why-parking-lot"></a>
#### Why parking_lot?

`parking_lot::{Condvar, Mutex}` powers the `progress_lock` that serializes synchronous runtime entry and the `spawn_mutex` that pairs with the `spawn_cv` condvar. The work-available signal itself is an `AtomicBool` (`spawn_flag`) so that `RuntimeT::spawn()` can signal without acquiring a lock. No spurious `Condvar` wakeups and no poisoning across FFI make it safe to use directly on the C boundary. Note: session operations use `tokio::sync::Mutex` instead (see [Sessions](#sessions)) because `tokio::sync::MutexGuard` is `Send` and must be held across `.await` points inside spawned tasks.

<a id="why-single-cursor-type"></a>
#### Why a single `mongoac_cursor_t` type?

The Rust driver's dual-type design is a borrow-checker artifact that cannot be enforced at compile time across the C FFI boundary. A single mongoac type embeds the session via `Arc<tokio::sync::Mutex<ClientSession>>` (not `parking_lot::Mutex` — `tokio::sync::MutexGuard` is `Send` and safe to hold across `.await` points), provides non-owning document access via `bson_init_static`, and simplifies the API with uniform destroy and iteration patterns.

<a id="why-dedicated-session-parameter"></a>
#### Why a dedicated `mongoac_session_t *session` parameter instead of a BSON field?

A dedicated pointer locks the ABI from day one: callers pass `NULL` until sessions arrive (see the [Sessions](#sessions) specification), without requiring an options-BSON migration. Embedding session as a free-form BSON key would be less discoverable for callers and harder to deprecate later. The dedicated pointer also matches the Rust driver's explicit-session API, where `&mut ClientSession` is passed as a separate argument to operation builders.

<a id="why-bson-string-cursortype"></a>
#### Why BSON string for CursorType?

Consistency with the existing [options deserialization pattern](#why-bson-options): the BSON options document is deserialized directly into the Rust struct via serde, which handles the string-to-enum mapping. An integer enum would require a parallel C `#define` set and manual conversion code that duplicates serde's work.

### Supported Features

#### Connection Strings (URI)

<a id="why-no-uri-type"></a>
##### Why no separate URI type?

The `mongodb` crate's `ClientOptions::parse()` is the authoritative parser. Calling it directly from the FFI layer avoids duplicating parse logic and eliminates a C-only two-phase construction model. URI inspection can be added later without breaking existing API.

> [!TIP]
> - [Why not a full URI type?](#deferred-full-uri-type)
> - [Why not a read-only URI type?](#deferred-read-only-uri-type)

<a id="why-client-new-blocks"></a>
##### Why block for DNS?

`ClientOptions::parse()` is async because `mongodb+srv://` requires DNS lookups. Blocking the caller with `runtime.block_on()` avoids exposing an async parse to C. URI parsing is one-time and typically completes in milliseconds. Using the client's own runtime avoids creating a temporary one.

<a id="why-no-uri-getters"></a>
##### Why no URI option getters?

`ClientOptions` fields are consumed during `Client::with_options()`. Exposing them back would require storing a copy inside `mongoac_client_t` for rarely-accessed data. The common workflow (connect, operate, disconnect) does not need post-construction URI inspection.

<a id="why-connection-string-features-uri-only"></a>
##### Why are connection-string features URI-only?

The Rust driver's `ClientOptions::parse()` is the authoritative parser for connection strings. It validates and applies options for authentication, SDAM, server selection, compression, load balancing, max staleness, retryable reads, retryable writes, pool sizing, and SRV behavior internally. Typed C APIs for each feature would duplicate the Rust driver's model, increase the FFI surface, and require ongoing maintenance to track `#[non_exhaustive]` option fields. URI-only configuration keeps the API surface minimal and avoids leaking Rust-internal topology, pool, and monitoring state across the FFI boundary.

> [!TIP]
> - [Why not expose a credential type?](#rejected-credential-type)
> - [Why not expose typed C APIs for standard connection-string features?](#rejected-connection-string-feature-apis)

<a id="why-no-callback-auth"></a>
##### Why no callback-based auth API?

Rust-to-C callbacks would require the Rust driver to invoke caller-supplied C functions during async authentication on the per-client Tokio runtime, creating re-entrancy, cancellation, and lifetime hazards across the FFI boundary. The built-in OIDC environment integrations (Azure, GCP, k8s) and standard URI-driven mechanisms cover the common cases without callbacks.

> [!TIP]
> - [Why not expose a credential type?](#rejected-credential-type)
> - [Why not callback-based auth?](#rejected-callback-auth)

<a id="why-build-platform-metadata"></a>
#### Client Metadata

mongoac appends the C build configuration to the handshake `client.platform` field so support and diagnostics can distinguish Debug vs. Release and shared vs. static builds. The compact two-letter suffix keeps the existing `platform` field readable and the handshake document within the 512-byte limit. Additional single-character fields may be added in the future to capture other build or distribution attributes.

<a id="why-index-based-events"></a>
#### Event API

##### Why index-based events?

Index-based access keeps the FFI boundary one-directional: C pulls BSON documents from Rust-owned buffers on the same thread that drives the runtime. No C callbacks are invoked from internal tokio threads, avoiding re-entrancy, lifetime, and synchronization hazards. The BSON documents are produced by serde-serializing the Rust driver events directly, avoiding custom wrapper types and keeping the FFI implementation minimal.

> [!TIP]
> - [Why not callback-based events?](#rejected-callback-events)
> - [Event buffer limits](#event-buffer-limits)

<a id="why-ring-buffer-storage"></a>
##### Why ring buffer storage?

`VecDeque` advances the head pointer on `drain(..N)` rather than `Vec`'s O(N) `memmove`, avoiding the cost of shifting remaining elements. However, per-element drop is still O(N). Push, index lookup, and length queries remain O(1).

> [!TIP]
> - [Event buffer limits](#event-buffer-limits)

<a id="why-capture-all-events-by-default"></a>
##### Why is event capture opt-in rather than default?

Event capture imposes serialization and memory overhead on every operation. Making it opt-in avoids paying this cost for callers that do not consume events.

<a id="why-owning-event-doc-access"></a>
##### Why owning event document access?

Events are serialized to BSON at `get()` time and returned as an owning `*mut bson_t` via `bson_new_from_data`. The caller must `bson_destroy()` the returned pointer. This avoids the complexity of pinning pre-serialized buffers across `clear()` and client destruction, and keeps the event storage format (unserialized Rust driver types) decoupled from the C API return format. Pre-serialized storage with non-owning views remains a potential future optimization ([deferred](#deferred-bson-conversion-efficiency)), as it would require managing the lifetime of pinned `RawDocumentBuf` entries across `clear()` operations.

<a id="why-pass-through-event-shapes"></a>
##### Why pass through Rust driver event shapes?

Consistent with the [partially-transparent error-handling approach](#error-handling-transparency): the FFI layer validates pointer safety and encoding, but defers semantic choices (including event serialization format) to the Rust driver. Normalizing event shapes to spec-expected conventions would require duplicating or transforming Rust driver internals, adding complexity and risk of divergence from the upstream crate. Known spec-level divergences (e.g., `topologyId` serde skip, untagged serialization, duration subdocuments, string `failure`, `connectionId` and `databaseName` shapes) are accepted as-is and documented in the investigation reports.

<a id="why-uri-only-client-server-selection"></a>
##### Why are client-specific server selection options URI-only?

`serverSelectionTimeoutMS` and `localThresholdMS` are client-level settings in the Rust driver (`ClientOptions`). They affect every operation on the client and are naturally expressed through the URI. Because they are client-specific and the Rust driver already parses them from the URI, a typed C API would duplicate the Rust driver's model without providing new capabilities.

<a id="why-bson-read-preference"></a>
##### Why are database/collection/operation read preferences passed as BSON documents?

Unlike client-specific server selection options, `readPreference`, `maxStalenessSeconds`, and `readPreferenceTags` are meaningful at the database, collection, and operation levels in the Rust driver. The `mongodb` crate exposes `SelectionCriteria` on `DatabaseOptions`, `CollectionOptions`, and per-operation option structs such as `FindOptions`. A `bson_t*` document deserializes directly into `SelectionCriteria` through the Rust driver's serde, reusing the same validation and shape without introducing a new C type.

> [!TIP]
> - [Typed C API for read preference](#deferred-typed-read-preference)

<a id="why-internal-no-api"></a>
##### Why do SDAM, retry, and step-down resilience require no C API?

These behaviors are managed entirely inside the Rust driver, which exposes no public API to read topology state, toggle per-operation retry, or manually clear connection pools. Because the C caller cannot influence them through any Rust API, there is no C API surface to expose.

#### Collation

<a id="why-no-maxwireversion-check"></a>
##### Why no maxWireVersion check?

The Rust driver sends collation unconditionally. mongoac defers spec-level validation to the Rust API per the ["partially-transparent" error-handling approach](#error-handling-transparency). This is a known spec divergence.

<a id="why-opcode-non-issue"></a>
##### Why are opcode-based writes not a concern?

The Rust driver uses `OP_MSG` exclusively — the opcode-based restriction is inapplicable.

#### Sessions

<a id="why-tokio-sync-mutex"></a>
##### Why `tokio::sync::Mutex` instead of `parking_lot::Mutex` for session state?

`parking_lot::MutexGuard` and `std::sync::MutexGuard` are `!Send` — they cannot be held across `.await` points inside a spawned task. `tokio::sync::MutexGuard` is `Send` and designed for this pattern.

> [!TIP]
> - [Why not parking_lot or std::sync::Mutex for session state?](#rejected-tokio-mutex-blocking-lock)

<a id="why-session-mutex-held-across-await"></a>
##### Why hold the session mutex across the entire operation `.await`?

The Rust driver's action builders consume `&mut ClientSession` for the operation's full duration. There is no intermediate point to release the lock before `.await` completes.

<a id="why-progress-lock-extended"></a>
##### Why extend `progress_lock` to guard synchronous `block_on()` calls?

`RuntimeT::block_on()` acquires `progress_lock` internally before every `runtime.block_on()` call, so callers do not need to manage the lock manually. On `current_thread` runtimes, the IO/timer driver `Core` is single-owner (`AtomicCell`). Two threads cannot enter `block_on()` simultaneously without one losing driver access. The shared lock prevents this.

<a id="why-session-accessors-block-on-mutex"></a>
##### Why do synchronous session accessors block on the mutex instead of using `try_lock()`?

`try_lock()` would return `WouldBlock` on contention, forcing callers to retry — an unfamiliar pattern for synchronous accessors and inconsistent with the rest of mongoac.

<a id="why-fire-and-forget-abort"></a>
##### Why fire-and-forget async abort on session destroy?

This is inherent to the Rust driver's `ClientSession::Drop`. Diverging would require a synchronous abort path absent from the upstream driver.

<a id="why-defer-and-run"></a>
##### Why defer the `and_run()` transaction retry convenience?

Exposing `and_run()` would require Rust-to-C callback invocation — re-entrancy, cancellation, and lifetime hazards that every other mongoac feature deliberately avoids.

> [!TIP]
> - [Why not callback-based auth?](#rejected-callback-auth)
> - [Why not callback-based events?](#rejected-callback-events)

<a id="why-session-drop-sends-endsessions"></a>
##### Why is no explicit `endSessions` C API needed?

The Rust driver's `Client::Drop` impl handles this automatically — sending pooled session IDs (in chunks of 10,000) during client destruction.

<a id="why-causal-consistency-automatic"></a>
##### Why is causal consistency mostly automatic (no explicit C API)?

The Rust driver's executor handles `operationTime` capture and `afterClusterTime` injection automatically — the C caller only needs to create a session with `causalConsistency: true`. Manual accessors are only needed for cross-session token propagation (`advance_operation_time`, `advance_cluster_time`), a rare use case.

<a id="why-causal-consistency-writes-known-limitation"></a>
##### Why is the `afterClusterTime`-on-writes limitation documented as a known limitation rather than worked around?

The upstream Rust driver's executor gates `afterClusterTime` injection on `op.read_concern().supported()`, and write operations return `Feature::NotSupported`. The FFI layer cannot override this — the Rust executor controls the wire protocol. Documenting the limitation is the only viable option. Filing an upstream issue is recommended.

#### Transactions

<a id="why-both-async-sync-transaction"></a>
##### Why both async and sync transaction variants?

Transaction operations are typically called in sequence by a single thread with no concurrent work to drive — forcing every call through the future poll loop adds boilerplate without concurrency benefit. The sync variant uses `runtime.block_on()` with `progress_lock`, matching the existing pattern of `mongoac_client_start_session()` and synchronous cursor iteration. Callers who need non-blocking I/O use the `*_async()` variants.

<a id="why-rust-transaction-state"></a>
##### Why rely on Rust for transaction state validation?

The Rust driver validates all state transitions synchronously (before any async I/O) behind the `Arc<Mutex<ClientSession>>` guard — there is no async hop for pure state errors. Duplicating the state machine on the C side would add drift risk as the Rust driver's state machine evolves, with no measurable performance benefit.

<a id="why-both-default-txn-options"></a>
##### Why support both session-level and per-call default transaction options?

The Rust driver's inheritance chain (session-level defaults overridden by per-call values) is handled entirely on the Rust side — no C-side storage of default options is needed. Supporting both mechanisms gives C callers full flexibility while keeping the FFI boundary stateless.

<a id="why-error-t-enum"></a>
##### Why refactor `ErrorT` to an enum storing original errors?

The previous flat struct discarded error labels, wire version, server response, and source chain. An enum where each variant stores the original error (`Rust(mongodb::error::Error)`, `Bson(bson::error::Error)`, etc.) preserves all metadata naturally and delegates `mongoac_error_has_label()` directly to the Rust driver's `contains_label()` without a separate label-tracking collection.

## Rejected Ideas

> [!IMPORTANT]
> Ideas considered but not pursued during foundation design.

### Build System

<a id="rejected-manual-c-verification"></a>
#### Dedicated C-only header verification targets

Manual `OBJECT` targets were investigated to verify generated headers as C. Rejected because `FILE_SET HEADERS` on `INTERFACE` targets already integrates with CMake's `CMAKE_VERIFY_INTERFACE_HEADER_SETS` option. Manual `OBJECT` targets would duplicate this functionality and require fragile `add_dependencies` logic for generated headers.

<a id="rejected-downstream-bson-link"></a>
#### Static embedding of bson2 into the cdylib

Rejected: statically embedding bson2 into the shared library would create duplicate symbol instances in processes that also load libmongoc, violating the one-definition rule for `bson_t` internals.


### Test Infrastructure

<a id="rejected-mongoc-testsuite"></a>
#### Reusing mongoc's TestSuite

Rejected: `TestSuite` is C-only and sync-centric, with `fork()`-based isolation and compile-time test registration. It has no awareness of C++ lifetimes or async polling. Adapting it for C++ and async futures exceeds the value of reuse.

### Rust FFI Design

#### bson2

<a id="rejected-new-bson-type"></a>
##### New Rust BSON type for FFI

Rejected: reimplementing `bson_t` layout in Rust risks UB from alignment/packing divergence. C users would need to learn a new type.

<a id="rejected-flat-structs"></a>
#### Flat value-type structs

Rejected: `#[repr(C)]` structs with public fields break ABI on every layout change. Inline `char message[512]` arrays force truncation and waste stack space.

<a id="rejected-box-dyn-error"></a>
#### Box<dyn std::error::Error> opaque handle

Rejected: trait objects discard category/code metadata. C callers need integer codes to classify errors, not just Display strings.

<a id="rejected-c-enums"></a>
#### C enumerations for category/code

Rejected: C99 allows the compiler to choose any compatible integer type for enum underlying types. Even a `Sentinel = INT32_MAX` enumerator only influences the heuristic; it is not mandated. Pre-C23 compilers also erase enum types to `typedef int32_t`.

<a id="rejected-manual-match"></a>
#### Manual match impl blocks for enum conversions

Rejected: violates single responsibility. Adding one error code would require editing 4–5 locations. At scale, manual match blocks risk drift and have no compile-time verification.

<a id="rejected-pure-macros"></a>
#### macro_rules! for conversions and messages

Rejected: cbindgen does not expand general `macro_rules!` macros, so `pub const` items and enum variants would still need separate hand-written lists. At scale, macro tables become unwieldy and produce poor diagnostics.

<a id="rejected-build-time-codegen"></a>
#### Build-time code generation from a master .def file

Rejected: a code generator adds build infrastructure overhead disproportionate for a proof-of-concept. Generated files create pre-commit friction. Deferrable: derive macros (`num_enum` + `strum`) suffice.

<a id="rejected-inline-char"></a>
#### Inline char arrays for strings

Rejected: fixed-size arrays truncate long messages and rigidify ABI. Changing array size breaks binary compatibility.

<a id="rejected-waker-integration"></a>
#### Waker-based event-loop integration

A waker callback inverts control: Rust decides when to notify the caller rather than the caller polling. This imposes a mandatory thread-safe synchronization contract on all consumers, including those without an event loop, and waker vtables are complex to implement correctly across language boundaries.

<a id="rejected-separate-futures"></a>
#### Separate future types per result category

Separate future types per result category would require a distinct Rust struct, module, and generated header for each category, duplicating `poll`, `destroy`, and any extension functions. C provides no strong type checking for opaque pointers, so the compile-time safety benefit is limited.

<a id="rejected-exhaustive-progress"></a>
#### Exhaustive make_progress until idle

Looping until all spawned tasks report idle would reduce boundary crossings, but the runtime cannot guarantee termination. A single long-running task could monopolize the calling thread indefinitely.

<a id="rejected-cooperative-cancellation"></a>
#### Cooperative cancellation for Phase 0

Cooperative cancellation requires wiring an `AbortHandle` or oneshot channel into every operation and testing race conditions during normal resolution and client shutdown. The bookkeeping and test coverage required are significant for an initial API.

<a id="rejected-borrowed-references"></a>
#### Borrowed `&'r Runtime` references in RuntimeT

Borrowed references are sufficient when Rust controls lifetimes and callers are single-threaded. C callers do not understand borrow semantics, and a `RuntimeT` may outlive its originating `ClientT`, so `Arc<Runtime>` inside `RuntimeT` is the only safe choice. `RuntimeT` stores `Arc<Runtime>` and `Arc<Mutex<()>>` (for `progress_lock`), and is `Clone`-derived so callers can cheaply share handles.

<a id="rejected-temporary-runtime"></a>
##### Temporary runtime for URI parse

Rejected: the per-client runtime already exists at parse time. Using it eliminates overhead and preserves DNS cache between parse and connection.

<a id="rejected-separate-cursor-types"></a>
#### Exposing separate C cursor types for implicit and explicit sessions

The Rust driver's dual-cursor type is a borrow-checker artifact. C lacks Rust's lifetime system, so the distinction cannot be enforced at compile time. A single cursor type with an embedded session provides the same capabilities with a simpler API and fewer opportunities for caller error.

### Supported Features

#### Connection Strings (URI)

<a id="rejected-connection-string-feature-apis"></a>
##### Exposing typed C APIs for standard connection-string features

The Rust driver already parses, validates, and applies URI options internally. A parallel C API would duplicate that model and require ongoing maintenance for `#[non_exhaustive]` option fields without providing new capabilities.

<a id="rejected-credential-type"></a>
##### Exposing a `mongoac_credential_t` type with typed setters

A dedicated credential type would create a parallel construction path alongside the URI without adding functionality for standard auth mechanisms. It would also not enable callback-based auth, which the FFI deliberately avoids.

<a id="rejected-callback-auth"></a>
##### Callback-based authentication API

Rust-to-C callbacks during async authentication introduce re-entrancy, cancellation, and lifetime hazards across the FFI boundary. Standard URI-driven mechanisms and the Rust driver's built-in OIDC integrations cover common cases.

#### Read Concern, Write Concern & Read Preference

<a id="rejected-empty-read-concern-hack"></a>

##### Raw BSON readConcern: {} at the FFI boundary

The CRUD spec permits sending `readConcern: {}` to reset a parent-level concern to server default. The Rust driver's public API cannot produce this — `ReadConcern.level` is not `Option<ReadConcernLevel>`, conflating "not set" with "explicit server default". A raw-BSON workaround would need to replicate the driver's serde and wire-protocol serialization, creating a fragile internal divergence.

<a id="rejected-callback-events"></a>
#### Event API

##### Callback-based event handlers

Rust-to-C callbacks from internal tokio threads would require `Send + Sync`, re-entrancy safety, and non-blocking guarantees — a complex cross-language concurrency contract. Index-based buffers avoid invoking C code from Rust threads entirely.

<a id="rejected-event-shape-normalization"></a>

##### Event shape normalization

Post-processing serialized BSON events to match spec-expected conventions (renaming fields to camelCase, converting durations to flat integers, lowercasing reason strings, injecting `databaseName`) would require duplicating or transforming Rust driver internals — whether through a custom serde wrapper, BSON post-processing, or an event-bridge layer. All three approaches create maintenance burden and risk of divergence from the upstream crate that mongoac does not control.

<a id="rejected-list-database-names-string-getter"></a>

##### Dedicated string array getter for listDatabaseNames

An earlier design called for a `mongoac_future_get_strings(future, &data, &len, &error)` getter returning `const char**` and `size_t`, with a companion `mongoac_future_free_strings()` function. While more ergonomic for C callers (no BSON parsing to extract strings), this was rejected in favor of returning a BSON array via the existing `mongoac_future_get_bson()` getter. The BSON array pattern keeps the API consistent between `listDatabases` and `listDatabaseNames` and avoids adding a new result-extraction function for a single operation.

<a id="rejected-list-databases-dedicated-result"></a>

##### Dedicated mongoac_list_databases_result_t type

Creating a dedicated `mongoac_list_databases_result_t` opaque handle with count, index-access, and destroy methods was considered. This would provide a cleaner iteration API for C callers but was rejected in favor of reusing the existing BSON array pattern. Adding a dedicated type later is an additive change that does not break ABI.

#### Enumerate Collections

<a id="rejected-collection-type-enum"></a>

##### Dedicated C enum for CollectionType

A dedicated `mongoac_collection_type_t` enum with constants for each variant (`COLLECTION`, `VIEW`, `TIMESERIES`, `BUCKET`) was considered for type safety and discoverability. Rejected in favor of a string field in the BSON document — the user confirmed that C callers parsing the `type` field as a string is sufficient, and a parallel enum would add maintenance overhead without clear benefit.

<a id="rejected-cursor-types-per-result"></a>

##### Separate cursor types for CollectionSpecification vs Document

Distinct cursor types for `listCollections` results (`Cursor<CollectionSpecification>`) and `find` results (`Cursor<Document>`) were considered. Rejected in favor of a single `mongoac_cursor_t` that exposes raw BSON bytes per entry regardless of the underlying Rust type — the user confirmed this approach. The C caller sees a non-owning `*const bson_t` view in both cases.

#### Collection Management

<a id="rejected-rename-collection"></a>

##### Exposing rename_collection

Exposing `rename_collection` as either a client-level or database-level function was considered. The user rejected exposure entirely: if the Rust driver does not natively support it (no dedicated `rename_collection` API — must use `admin.run_command()` with a manual command document), the Rust FFI should not expose it either.

<a id="rejected-fire-and-forget-admin"></a>

##### Fire-and-forget variants for admin operations

Fire-and-forget variants (no future object) for `create_collection`, `drop_collection`, and `drop_database` were considered. Rejected — the user confirmed these admin operations should remain async-only with future handles, consistent with the rest of mongoac.

<a id="rejected-separate-create-view"></a>

##### Separate create_view function

A dedicated `mongoac_database_create_view_async` function for discoverability was considered. Rejected in favor of expressing view creation through `create_collection` with `viewOn` and `pipeline` fields in the BSON options — consistent with the Rust driver and the existing BSON-options pattern.

<a id="rejected-collection-type-enum-mgmt"></a>

##### Dedicated C enum for collection lifecycle options

A dedicated struct with typed setters for `CreateCollectionOptions` fields (similar to `mongoac_client_options_t`) was considered. Rejected — all options are passed as BSON documents following the existing deserialization pattern, consistent with other operation options.

#### Sessions

<a id="rejected-sync-only-sessions"></a>

##### Sync-only session API (no async session operations)

Restricting session parameters to synchronous API functions only was considered to avoid the thread-safety complexity of `Arc<Mutex<>>`. Rejected — the approach is viable, and sync-only sessions would force callers to choose between explicit sessions and non-blocking I/O.

<a id="rejected-per-op-mutex"></a>

##### Per-operation Mutex lock-then-drop pattern (don't hold across `.await`)

Locking the session mutex only to configure the action builder before `.await` was considered. Rejected: the Rust driver's `.session()` method does not return a separately configured builder — the `&mut` reference is consumed by `.await` and held internally for the operation's full duration. There is no point to release the lock early.

<a id="rejected-tokio-mutex-blocking-lock"></a>

##### Using `std::sync::Mutex` or `parking_lot::Mutex` for session state

Both produce `!Send` guards, which cannot be held across `.await` in a spawned task (`tokio::spawn` requires `Send`). `tokio::sync::Mutex` is the only standard choice producing a `Send` guard.

<a id="rejected-callback-and-run"></a>

##### Callback-based `and_run()` transaction retry API

Exposing the Rust driver's `and_run()` / `and_run2()` retry-loop convenience as a C callback was considered but rejected. Rust-to-C callbacks from inside async code introduce re-entrancy, cancellation, and lifetime hazards across the FFI boundary that every other mongoac feature deliberately avoids. C callers implement their own retry loop using explicit `start`/`commit`/`abort`.

<a id="rejected-async-only-transaction"></a>

##### Async-only transaction API

Requiring every `start`/`commit`/`abort` to go through the future poll loop was considered, but transaction operations are typically called in a tight sequence by a single thread with no concurrent work — the poll loop adds boilerplate and yield overhead without concurrency benefit.

<a id="rejected-c-side-state-machine"></a>

##### Duplicating the transaction state machine on the C side

Tracking `TransactionState` on the C side was considered, but the Rust driver already validates all state transitions synchronously (before any async I/O) behind the `Arc<Mutex<ClientSession>>` guard. A parallel C-side state machine creates drift risk whenever the Rust driver's state machine evolves, with no measurable performance benefit — the Rust validation path is lock-call-return with no async hop.

<a id="rejected-vec-cstring-error-labels"></a>

##### Storing error labels as `Vec<CString>` in `ErrorT`

A separate `Vec<CString>` of error labels was considered, but it creates a duplicate-tracking problem every time an error crosses the FFI boundary — the label collection drifts from the underlying `mongodb::error::Error` whenever one is reconstructed or which lacks the original's `contains_label()` method.

## Deferred Features

> [!IMPORTANT]
> Features intentionally excluded from the current scope. Each item can be added later without breaking the existing API or ABI unless noted otherwise.

### bson2

<a id="deferred-bson-conversion-efficiency"></a>

#### Avoiding deep-copies when converting between `bson_t` and `Document`/`RawDocument`/`RawDocumentBuf`

The `mongoac_future_get_bson()` getter serializes a `Document` to bytes then passes them to `bson_new_from_data`, which always copies. Avoiding the copy in the `Document` → owning `*mut bson_t` path could use `bson_new_from_buffer` to transfer ownership of the serialized bytes directly into the `bson_t`.

Options deserialization (`*const bson_t` → `Document` → `*Options`) goes through a full `IndexMap` parse because the Rust driver's `*Options` structs implement `DeserializeOwned`, not `Deserialize<'a>`. A non-owning `&RawDocument` → `*Options` path is not expressible without changes to the upstream `mongodb` crate. This is deferrable without public API or ABI impact.

### Test Infrastructure

<a id="deferred-dagger-integration"></a>

#### Dagger integration for local CI testing

Deferred to a later phase. The existing CMake + `uv` + Cargo workflow is sufficient for the proof-of-concept.

### Supported Features

#### Connection Strings (URI)

<a id="deferred-full-uri-type"></a>
##### Full mongoac_uri_t with typed getters and setters

A full URI type would require a parallel option store because `ClientOptions` is read-only after construction. This creates maintenance coupling with the `mongodb` crate's `#[non-exhaustive]` fields.

<a id="deferred-read-only-uri-type"></a>
##### Read-only mongoac_uri_t with getters only

A read-only wrapper adds little value over a future `mongoac_client_get_uri_string()`. The same diagnostics/logging use case is served without introducing opaque-handle lifecycle.

#### Client Options

<a id="deferred-client-options-fields"></a>

##### Other purely programmatic `ClientOptions` fields

The following `ClientOptions` fields cannot be expressed through the connection
string but are **deferred** from the initial `mongoac_client_options_t` API:

| Field | Why deferred |
|---|---|
| `driver_info` | `mongoac_client_append_metadata()` already enables post-construction metadata injection. A separate setter would duplicate this existing API. |
| `tracing_max_document_length_bytes` | Feature-gated (`tracing-unstable`) in the Rust driver. Deferred until the tracing feature is stable. |
| `tracing` (opentelemetry) | Feature-gated (`opentelemetry`) in the Rust driver. Requires establishing a tracing strategy for mongoac first. |

Adding these fields later is an additive change: new setter functions on
`mongoac_client_options_t` that do not break existing C API or ABI.

#### Read Concern, Write Concern & Read Preference

<a id="deferred-typed-read-preference"></a>

##### Typed C API for read preference, read concern, and write concern

`SelectionCriteria`, `ReadConcern`, and `WriteConcern` are currently passed as BSON fields within `*const bson_t options` documents. A future `*_with_options()` extension (analogous to `mongoac_client_new_with_options()`) could accept dedicated C structs, providing stronger type checking and enabling configurations not expressible via plain BSON deserialization — such as an empty `ReadConcern { }` for server-default reset (the Rust driver's public API cannot construct this). Deferred because the BSON options pattern covers all common cases, and adding typed structs later is an additive change.



#### Logging

Logging is deferred. No log callback, level constants, or default logger are exposed in this phase.

#### Client Metadata

<a id="post-construction-app-name"></a>
##### Post-construction application name setter

The handshake spec permits setting the application name on the `MongoClient` before any connection is established. The Rust driver only accepts `app_name` through `ClientOptions` at construction time; there is no `Client::set_app_name` API. A post-construction setter in mongoac would require rebuilding the underlying `Client` (and its runtime) or upstream Rust driver support. Deferred until the Rust driver exposes this capability or the use case justifies the complexity.

#### Collation

##### Separate `mongoac_collation_t` type

A dedicated collation handle with typed setters for each field would provide discoverability and type safety, following the same pattern as the deferred [typed read-preference/concern structs](#deferred-typed-read-preference). The existing BSON options pattern covers all common cases. Adding a `mongoac_collation_t` later is an additive change.

##### Bulk write collation helper

Deferred to the bulk write feature design.

##### Change stream collation documentation

The Rust driver's `ChangeStreamOptions.collation` is `#[serde(skip_serializing)]`. This constraint should be documented when change streams are designed.

##### Index collation and dropIndex

Verification of `DropIndexOptions` collation support is deferred to the index management feature design.

#### Enumerate Collections

<a id="deferred-list-collection-names-streaming"></a>

##### Streaming variant for listCollectionNames

The current `list_collection_names` implementation eagerly collects all results into a `Vec<String>` before returning. A streaming variant returning a cursor of name strings would reduce memory overhead for databases with very large numbers of collections. Deferred — the `list_collections` cursor path already provides a streaming alternative, and adding a streaming names variant later is an additive change.

#### Async Operations

<a id="deferred-transaction-retry-callback"></a>

##### Callback-based transaction retry callback (`and_run`)

Exposing `and_run()` would require Rust-to-C callback invocation, which is deliberately avoided elsewhere in mongoac. Deferred — C callers use explicit `start`/`commit`/`abort`.

<a id="deferred-transaction-timeoutms"></a>

##### `timeoutMS` support in the transaction API

The Driver Transactions specification deprecates `maxCommitTimeMS` in favor of `timeoutMS`, but mongoac has not yet introduced `timeoutMS` for any operation. Deferred until `timeoutMS` is added to the broader mongoac API — `maxCommitTimeMS` via `TransactionOptions` BSON is sufficient for initial support, and adding `timeoutMS` later is a non-breaking additive change.

## Open Issues

> [!IMPORTANT]
> Items that warrant discussion but do not yet have a concrete resolution.

### Build System

<a id="version-current-strategy"></a>
#### Version strategy: independent vs. root VERSION_CURRENT

Should mongoac use its own `VERSION_CURRENT` or share the root `VERSION_CURRENT` from the `mongo-c-driver` repository?

- **Independent:** allows separate release cadence; mongoac is experimental (v0.x) while mongoc is stable (v2.x).
- **Unified:** simplifies packaging and release notes; all three libraries ship together.

Current approach: **independent** — mongoac maintains its own `src/libmongoac/VERSION_CURRENT` (`0.1.0-dev`), separate from the root `VERSION_CURRENT` (`2.3.1`). Switching to unified later is a packaging change, not an ABI break, but it is disruptive for release tooling and downstream consumers expecting a separate version line.

### Rust FFI Design

<a id="error-code-strategy"></a>
#### Error code strategy: stable constants vs. raw pass-through

Should server and Rust driver error codes be exposed as named `#define` constants or as raw integer pass-through values?

- **Stable constants for all codes:** every error code (including server and Rust driver) is mapped to a named `#define`. This enables `#ifdef` feature-gating and gives C callers a stable vocabulary for classification. However, mapping an open-ended set of server codes requires a version-locked table that is costly to maintain.
- **Raw pass-through for external codes:** server and Rust driver codes are passed through as raw integers; only mongoac-internal codes (invalid argument, NULL pointer, etc.) use named constants. This avoids premature commitment while the taxonomy evolves, and named mongoac constants can still be added later without breaking ABI. However, C callers lose the ability to classify server errors with `#ifdef` or symbolic names.

Current approach: **raw pass-through** — only mongoac-internal category constants and synthetic codes use named `#define` constants; server and Rust driver codes pass through as raw integers. This choice is deferrable: stable constants can be added later without breaking ABI.

<a id="error-code-extraction"></a>
##### Error code extraction from `mongodb::error::Error`

`ErrorT::code()` maps an internal `mongodb::error::ErrorKind` to an `ErrorCodeT` integer. Only four `ErrorKind` variants carry server error codes: `Command`, `Write`, `InsertMany`, and `BulkWrite`. Codes across all four draw from the same server error-code namespace and are not disambiguated by the integer value alone (e.g., code `91` can appear in both `CommandError.code` and `WriteConcernError.code`). All other variants return `ErrorCodeT::Unknown(i32::MIN)`, indicating the error exists but no numeric code is available.

For multi-error variants (`InsertMany`, `BulkWrite`), write concern errors are preferred over individual write errors when both are present. This priority is deferrable: changing it does not break the C API or ABI.

#### String API: C strings vs. string views

Which representation should mongoac use for all string parameters and return values that cross the FFI boundary?

- **`const char*`:** the de facto standard for C libraries; Rust's `CStr`/`CString` handle it safely. However, embedded NUL bytes are ambiguous, every call requires `strlen()`, and retrofitting to a view-based API later is an ABI-breaking change.
- **`const char* + size_t`:** avoids embedded NUL issues and removes `strlen()`. However, every string parameter becomes two parameters, which is verbose and inconsistent with struct-based API patterns elsewhere in mongoac.
- **`mongoac_string_view_t`:** a `#[repr(C)]` struct encapsulating pointer + length; consistent with modern C APIs, avoids embedded NUL issues, and avoids `strlen()`. However, C callers must construct a struct for every string argument, adding friction for the common case.

This is an all-or-nothing decision at design time: once `const char*` is in the public API, it cannot be replaced without breaking ABI.

Current approach: **`const char*`** — all public APIs currently use null-terminated byte strings. This choice is effectively irreversible: switching to `const char* + size_t` or `mongoac_string_view_t` would be an ABI-breaking change.

<a id="error-handling-transparency"></a>
#### Error-handling transparency

Should the FFI layer defer all error semantics to the Rust API, or implement spec-compliant validation on top of it? `mongoac_client_append_metadata()` is a representative example: the Driver Handshake spec requires `name` to be present, forbids `|` in driver-info strings, and limits the metadata document to 512 bytes, but the underlying Rust `Client::append_metadata()` does not enforce these constraints.

- **Fully-transparent:** perform only FFI conversions and rely on the Rust API for all validation. This minimizes FFI code and avoids divergence from Rust behavior. However, invalid UTF-8 is silently replaced with `U+FFFD` by `CStr::to_string_lossy()`, corrupting caller data, and C callers may see server-side errors or non-compliant handshake metadata that mongoac could have caught earlier.
- **Partially-transparent:** validate immediate FFI safety (non-null required pointers) and encoding preconditions (valid UTF-8), but defer semantic/spec-level validation to the Rust API. This catches common C caller mistakes at the boundary while avoiding duplication of Rust driver logic. However, spec violations such as `|` in metadata or oversized documents pass through silently.
- **Opaque (spec-compliant FFI layer):** implement the full specification on the Rust side regardless of whether the Rust API enforces it. This gives C callers consistent, spec-compliant behavior independent of Rust driver version. However, it duplicates validation logic, requires ongoing maintenance to track spec changes, and risks divergence if the Rust driver later enforces different rules.

Current approach: **partially-transparent** — the FFI layer validates pointer safety and UTF-8 encoding, rejecting invalid UTF-8 with `MONGOAC_ERROR_CODE_INVALID_ARGUMENT`. Spec-level constraints (e.g., `|` in metadata, 512-byte limit) are delegated to the Rust API. This decision is deferrable: moving more or less validation to the boundary is a behavior change that does not break C API or ABI, though it may change which errors callers observe.

<a id="unrecognized-bson-fields"></a>
#### Unrecognized fields in BSON options deserialization

Should BSON deserialization warn on unrecognized fields?

- **Warn on unrecognized fields:** Emit a WARN-level log for any field not present in the corresponding Rust `*Options` struct. This helps callers catch typos early. However, it requires serde's unknown-field detection or a maintained allow-list per options type, adds log volume, and may produce warnings for intentionally ignored fields (e.g., forward-compatibility options).
- **Silently ignore:** Rely on serde's default behavior. This is simple, matches the Rust driver, and avoids log noise. However, callers may not realize they are passing misspelled or unsupported fields.
- **Reject unrecognized fields:** Treat unknown fields as an error and return `MONGOAC_ERROR_CODE_INVALID_ARGUMENT`. This gives the strongest validation but breaks forward compatibility and may reject documents with extra fields the caller does not control.

Current approach: **silently ignore** — the FFI layer relies on serde's default behavior. This decision is deferrable: adding warnings or strict rejection later is a behavior change that does not break the C API or ABI, though it may change runtime logs and which documents are accepted.

#### Async Operations

<a id="array-result-representation"></a>
##### Array-like result representation: bson_t vs ptr+len vs dedicated array type

Operations that return lists of values (e.g., `listDatabases`, `listDatabaseNames`, `listCollectionNames`, `listIndexes`) need a way to return array-like results across the FFI boundary. (Note: `listCollections` uses a cursor for its result, not a list result — see the CRUD Operations section for cursor conventions.) Three representations are under consideration:

- **BSON array via `bson_t` (current approach):** The result is encoded as a BSON array stored in the existing `bson_t*` returned by `mongoac_future_get_bson()`. The caller iterates elements using standard bson2 macros (`bson_iter_init_find` + `bson_iter_recurse`). Pros: reuses existing types and getter, no new API surface, consistent with BSON interchange format. Cons: C callers must know bson2 iteration APIs; elements of homogeneous type (e.g., all strings) incur BSON encoding overhead for a single concrete type; BSON arrays are stored as `{"0": ..., "1": ...}` internally, which may be surprising for callers expecting a flat C array.

- **Pointer+length out-params:** A getter returning pointers with an explicit count, e.g. `mongoac_future_get_str_array(future, &data, &len, &error)` returning `const char**` for string arrays, or `mongoac_future_get_doc_array(future, &data, &len, &error)` returning `const bson_t**` for document arrays. Pros: familiar C idiom, avoids BSON parsing for simple types (strings), explicit length and pointer make iteration natural. Cons: introduces a family of getter functions, ownership semantics must be defined per variant (does the caller free elements individually? is the array contiguous?), and heterogeneous arrays cannot be represented without a tag.

- **Dedicated `mongoac_array_t` opaque type:** An opaque handle with accessors like `mongoac_array_count(array, &error)` and `mongoac_array_get(array, index, &error)` returning a `bson_t*` element, plus a `mongoac_array_get_string(array, index, &error)` for string elements. Inspired by the cursor iteration pattern. Pros: type-safe, discoverable, extensible with typed accessors, single ownership convention (destroy the array). Cons: adds another opaque handle lifecycle (create, access, destroy) with its own memory management, requires index-based accessor functions per element type, heavier API surface for simple use cases.

Current approach: **BSON array via `bson_t`** — the existing `Bson` variant of `FutureValue` is reused for `listDatabases`, `listDatabaseNames`, and `listCollectionNames`, with the caller iterating the BSON array using bson2 macros. `listCollections` uses the `Cursor` variant of `FutureValue` instead, consistent with the CRUD cursor pattern. This choice is **not deferrable**: the `FutureValue` variant assigned to each operation determines which typed getter is valid. Changing an operation from one variant to another (e.g., from `Bson` to a dedicated array type) would break existing compiled callers, because `mongoac_future_get_bson()` on a future that no longer stores a `Bson` variant returns a runtime error. The chosen representation for each operation is locked at implementation time. Adding *new* operations with a different representation is always possible, but retrofitting an existing operation is an ABI break.

### Supported Features

<a id="event-buffer-limits"></a>
#### Event API

##### Should event buffers have a size limit?

Command, SDAM, and CMAP events will be stored in unbounded per-client buffers. Long-lived clients or heavy monitoring workloads could consume unbounded memory if the caller does not clear events promptly.

- **Unbounded buffers:** simple, no event loss, no configuration API. Con: unbounded memory growth if the caller forgets to clear.
- **Bounded buffers with tail eviction:** caps memory by dropping oldest events. Con: silently loses events; the limit is an arbitrary tuning parameter.
- **Bounded buffers with caller notification:** returns an error when full. Con: requires backpressure or an error signal that complicates the API.

Current approach: **unbounded buffers** — events are stored until the caller clears them. This decision is deferrable: adding a size limit later does not change the C function signatures or ABI; it only changes runtime behavior.
