use std::path::{Path, PathBuf};

macro_rules! includes {
    ($($h:expr),* $(,)?) => {
        vec![$($h.into()),*]
    };
}

// Keep synchronized with `skip_cargo_headers` in src/libmongoac/CMakeLists.txt.
const SKIP_CARGO_HEADERS: &[&str] = &["lib", "mod", "version"];

// Keep synchronized with `skip_forward_headers` in src/libmongoac/CMakeLists.txt.
const SKIP_FORWARD_HEADERS: &[&str] = &["bson", "cursor_type", "string"];

// Rename structs from `ExampleT` to `mongoac_example_t`.
fn rename_structs() -> std::collections::HashMap<String, String> {
    let pairs: &[(&str, &str)] = &[
        ("AggregateOptionsT", "mongoac_aggregate_options_t"),
        ("BsonT", "mongoac_bson_t"),
        ("BsonViewT", "mongoac_bson_view_t"),
        ("ClientOptionsT", "mongoac_client_options_t"),
        ("ClientSessionT", "mongoac_client_session_t"),
        ("ClientT", "mongoac_client_t"),
        ("CollectionT", "mongoac_collection_t"),
        ("CollectionOptionsT", "mongoac_collection_options_t"),
        ("CountOptionsT", "mongoac_count_options_t"),
        ("CredentialT", "mongoac_credential_t"),
        (
            "CreateCollectionOptionsT",
            "mongoac_create_collection_options_t",
        ),
        ("CursorT", "mongoac_cursor_t"),
        ("CursorTypeT", "mongoac_cursor_type_t"),
        ("DatabaseT", "mongoac_database_t"),
        ("DatabaseOptionsT", "mongoac_database_options_t"),
        ("DeleteOptionsT", "mongoac_delete_options_t"),
        ("DistinctOptionsT", "mongoac_distinct_options_t"),
        ("DropDatabaseOptionsT", "mongoac_drop_database_options_t"),
        (
            "DropCollectionOptionsT",
            "mongoac_drop_collection_options_t",
        ),
        (
            "EstimatedDocumentCountOptionsT",
            "mongoac_estimated_document_count_options_t",
        ),
        ("ErrorT", "mongoac_error_t"),
        ("FindOptionsT", "mongoac_find_options_t"),
        ("FindOneOptionsT", "mongoac_find_one_options_t"),
        ("FutureT", "mongoac_future_t"),
        ("InsertManyOptionsT", "mongoac_insert_many_options_t"),
        ("InsertOneOptionsT", "mongoac_insert_one_options_t"),
        ("ListDatabasesOptionsT", "mongoac_list_databases_options_t"),
        (
            "ListCollectionsOptionsT",
            "mongoac_list_collections_options_t",
        ),
        ("ReadConcernT", "mongoac_read_concern_t"),
        ("ReadPreferenceT", "mongoac_read_preference_t"),
        ("ReplaceOptionsT", "mongoac_replace_options_t"),
        ("RunCommandOptionsT", "mongoac_run_command_options_t"),
        (
            "RunCursorCommandOptionsT",
            "mongoac_run_cursor_command_options_t",
        ),
        ("RuntimeT", "mongoac_runtime_t"),
        ("ServerApiT", "mongoac_server_api_t"),
        ("ServerInfoT", "mongoac_server_info_t"),
        ("ServerSelectorT", "mongoac_server_selector_t"),
        ("ServerTypeT", "mongoac_server_type_t"),
        ("SessionOptionsT", "mongoac_session_options_t"),
        ("StringT", "mongoac_string_t"),
        ("StringViewT", "mongoac_string_view_t"),
        ("TransactionOptionsT", "mongoac_transaction_options_t"),
        ("TlsOptionsT", "mongoac_tls_options_t"),
        ("UpdateOptionsT", "mongoac_update_options_t"),
        ("WriteConcernT", "mongoac_write_concern_t"),
    ];

    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// List include directives needed by each crate header.
fn configure(name: &str, config: &mut cbindgen::Config) {
    let headers = match name {
        "bson" => includes!["stdint.h"],
        "aggregate_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/read_concern-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "client" => includes![
            "mongoac/client_options-fwd.h",
            "mongoac/client_session-fwd.h",
            "mongoac/error-fwd.h",
            "mongoac/future-fwd.h",
            "mongoac/list_databases_options-fwd.h",
            "mongoac/session_options-fwd.h",
            "mongoac/runtime-fwd.h",
            "mongoac/bson.h",
            "mongoac/string.h",
            "stdint.h",
        ],
        "client_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/server_api-fwd.h",
            "mongoac/read_concern-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/read_preference-fwd.h",
            "mongoac/server_selector-fwd.h",
            "mongoac/tls_options-fwd.h",
            "mongoac/credential-fwd.h",
            "mongoac/string.h",
            "stdbool.h",
            "stdint.h",
        ],
        "client_session" => includes!["mongoac/error-fwd.h", "mongoac/future-fwd.h"],
        "collection" => includes![
            "mongoac/database-fwd.h",
            "mongoac/client_session-fwd.h",
            "mongoac/cursor-fwd.h",
            "mongoac/error-fwd.h",
            "mongoac/future-fwd.h",
            "mongoac/drop_collection_options-fwd.h",
            "mongoac/insert_one_options-fwd.h",
            "mongoac/insert_many_options-fwd.h",
            "mongoac/find_options-fwd.h",
            "mongoac/find_one_options-fwd.h",
            "mongoac/delete_options-fwd.h",
            "mongoac/replace_options-fwd.h",
            "mongoac/update_options-fwd.h",
            "mongoac/count_options-fwd.h",
            "mongoac/estimated_document_count_options-fwd.h",
            "mongoac/distinct_options-fwd.h",
            "mongoac/aggregate_options-fwd.h",
            "mongoac/collection_options-fwd.h",
            "mongoac/bson.h",
            "mongoac/string.h",
            "stdint.h",
        ],
        "create_collection_options" => includes!["mongoac/error-fwd.h", "mongoac/bson.h"],
        "collection_options" => includes![
            "mongoac/read_concern-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/read_preference-fwd.h",
            "mongoac/server_selector-fwd.h",
        ],
        "credential" => includes![
            "mongoac/error-fwd.h",
            "mongoac/bson.h",
            "mongoac/string.h",
            "stdint.h",
        ],
        "cursor_type" => includes!["stdint.h"],
        "cursor" => includes![
            "mongoac/error-fwd.h",
            "mongoac/future-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "database" => includes![
            "mongoac/client-fwd.h",
            "mongoac/client_session-fwd.h",
            "mongoac/cursor-fwd.h",
            "mongoac/error-fwd.h",
            "mongoac/future-fwd.h",
            "mongoac/list_collections_options-fwd.h",
            "mongoac/drop_database_options-fwd.h",
            "mongoac/database_options-fwd.h",
            "mongoac/create_collection_options-fwd.h",
            "mongoac/run_command_options-fwd.h",
            "mongoac/run_cursor_command_options-fwd.h",
            "mongoac/bson.h",
            "mongoac/string.h",
        ],
        "database_options" => includes![
            "mongoac/read_concern-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/read_preference-fwd.h",
            "mongoac/server_selector-fwd.h",
        ],
        "delete_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/bson.h",
        ],
        "replace_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "update_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "count_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/read_concern-fwd.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "estimated_document_count_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/read_concern-fwd.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "distinct_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/read_concern-fwd.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "drop_collection_options" => includes!["mongoac/write_concern-fwd.h"],
        "drop_database_options" => includes!["mongoac/write_concern-fwd.h"],
        "error" => includes!["mongoac/string.h", "stdbool.h", "stdint.h"],
        "find_options" => includes!["mongoac/error-fwd.h", "mongoac/bson.h"],
        "find_one_options" => includes!["mongoac/error-fwd.h", "mongoac/bson.h"],
        "future" => includes![
            "mongoac/error-fwd.h",
            "mongoac/runtime-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "insert_many_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "insert_one_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "list_collections_options" => includes![
            "mongoac/error-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "list_databases_options" => includes!["mongoac/error-fwd.h", "mongoac/bson.h", "stdbool.h"],
        "read_concern" => includes!["mongoac/error-fwd.h", "mongoac/string.h"],
        "run_command_options" => includes![
            "mongoac/read_preference-fwd.h",
            "mongoac/server_selector-fwd.h",
        ],
        "run_cursor_command_options" => includes![
            "mongoac/cursor_type.h",
            "mongoac/read_preference-fwd.h",
            "mongoac/server_selector-fwd.h",
            "mongoac/error-fwd.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "read_preference" => includes![
            "mongoac/error-fwd.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "runtime" => includes!["mongoac/error-fwd.h", "mongoac/future-fwd.h", "stdint.h"],
        "server_api" => includes!["stdbool.h"],
        "server_info" => includes![
            "mongoac/bson.h",
            "mongoac/string.h",
            "stdbool.h",
            "stdint.h",
        ],
        "server_selector" => includes!["mongoac/server_info-fwd.h", "stdbool.h"],
        "session_options" => {
            includes!["mongoac/transaction_options-fwd.h", "stdbool.h", "stdint.h"]
        }
        "string" => includes!["stdint.h"],
        "tls_options" => includes!["mongoac/error-fwd.h", "mongoac/string.h", "stdbool.h",],
        "transaction_options" => includes![
            "mongoac/read_concern-fwd.h",
            "mongoac/write_concern-fwd.h",
            "mongoac/read_preference-fwd.h",
            "stdint.h",
        ],
        "write_concern" => includes![
            "mongoac/error-fwd.h",
            "mongoac/string.h",
            "stdbool.h",
            "stdint.h",
        ],
        _ => vec![],
    };

    config.sys_includes.extend(headers);
}

fn default_config() -> cbindgen::Config {
    cbindgen::Config {
        language: cbindgen::Language::C,
        style: cbindgen::Style::Type,
        autogen_warning: Some(
            "// Generated by src/libmongoac/cmake/generate-crate-headers/src/main.rs".to_owned(),
        ),
        include_version: true,
        cpp_compat: true,
        braces: cbindgen::Braces::NextLine,
        line_length: 120,
        tab_width: 3,
        documentation: false,
        documentation_style: cbindgen::DocumentationStyle::C99,
        export: cbindgen::ExportConfig {
            rename: rename_structs(),
            ..Default::default()
        },
        function: cbindgen::FunctionConfig {
            prefix: Some("MONGOAC_API".to_owned()),
            ..Default::default()
        },
        no_includes: true,
        ..Default::default()
    }
}

fn generate_forward_header(crate_path: &Path, rel_stem: &Path, include_dir: &Path) {
    let rel_str = rel_stem.to_str().expect("invalid UTF-8");

    // `path/to/crate` -> `MONGOAC_PATH_TO_CRATE_FWD_H`
    let include_guard = format!(
        "MONGOAC_{}_FWD_H",
        rel_str
            .replace(std::path::MAIN_SEPARATOR, "_")
            .to_uppercase()
    );

    // `path/to/crate` -> `<include_dir>/path/to/crate-fwd.h`
    // Parent directories are already created by `generate_crate_header()`.
    let header_path = include_dir.join(rel_stem).with_file_name(format!(
        "{}-fwd.h",
        rel_stem
            .file_name()
            .and_then(|s| s.to_str())
            .expect("invalid UTF-8")
    ));

    let mut config = default_config();

    // Only forward declarations of opaque structs.
    config.export.item_types = vec![cbindgen::ItemType::OpaqueItems];

    // Generate the forward header.
    cbindgen::Builder::new()
        .with_config(config)
        .with_include_guard(&include_guard)
        .with_src(crate_path)
        .generate()
        .expect("cbindgen failed")
        .write_to_file(&header_path);
}

fn generate_crate_header(crate_path: &Path, src_dir: &Path, include_dir: &Path) {
    let file_stem = crate_path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("invalid UTF-8");

    if SKIP_CARGO_HEADERS.contains(&file_stem) {
        return;
    }

    // `src/libmongoac/src/path/to/crate.rs` -> `path/to/crate`
    let rel_path = crate_path
        .strip_prefix(src_dir)
        .expect("file not under src root");
    let rel_stem = rel_path.with_extension("");
    let rel_str = rel_stem.to_str().expect("invalid UTF-8");

    // `path/to/crate` -> `MONGOAC_PATH_TO_CRATE_H`
    let include_guard = format!(
        "MONGOAC_{}_H",
        rel_str
            .replace(std::path::MAIN_SEPARATOR, "_")
            .to_uppercase()
    );

    // `path/to/crate` -> `<include_dir>/path/to/crate.h`
    let header_path = include_dir.join(&rel_stem).with_extension("h");
    if let Some(parent) = header_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|_| panic!("failed to create include directory: {}", parent.display()));
    }

    // Default cbindgen configuration for all crates.
    let mut config = default_config();

    // All except `OpaqueItems` (which is declared in the forward header).
    config.export.item_types = vec![
        cbindgen::ItemType::Functions,
        cbindgen::ItemType::Typedefs,
        cbindgen::ItemType::Constants,
        cbindgen::ItemType::Enums,
        cbindgen::ItemType::Structs,
        cbindgen::ItemType::Unions,
    ];

    // Always include the component's forward header first.
    if !SKIP_FORWARD_HEADERS.contains(&file_stem) {
        generate_forward_header(crate_path, &rel_stem, include_dir);
        config.sys_includes.push(format!("mongoac/{rel_str}-fwd.h"));
    }

    // Normal headers typically export at least one symbol.
    config.sys_includes.push("mongoac/export.h".into());

    // Apply per-crate configuration options.
    configure(rel_str, &mut config);

    // Generate the crate header.
    cbindgen::Builder::new()
        .with_config(config)
        .with_include_guard(&include_guard)
        .with_src(crate_path)
        .generate()
        .expect("cbindgen failed")
        .write_to_file(&header_path);
}

// Recursively find all mongoac crates under the source directory which need a header.
fn find_crates(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("failed to read directory: {}", dir.display()))
    {
        let path = entry
            .map(|e| e.path())
            .unwrap_or_else(|_| panic!("failed to read directory entry in: {}", dir.display()));

        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "private") {
                continue; // Private crates do not need headers.
            }

            find_crates(&path, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
}

fn main() {
    let src_dir = std::env::var("MONGOAC_SRC_DIR")
        .expect("MONGOAC_SRC_DIR is required (the Rust source root directory)");
    let include_dir = std::env::var("MONGOAC_INCLUDE_DIR")
        .expect("MONGOAC_INCLUDE_DIR is required (the output directory for generated headers)");

    let src_path = Path::new(&src_dir);
    let include_path = Path::new(&include_dir);

    let mut files = Vec::new();
    find_crates(src_path, &mut files);

    // Generate crate headers in parallel.
    let mut handles = Vec::new();
    for path in files {
        let include_dir = include_path.to_path_buf();
        let src_dir = src_path.to_path_buf();
        handles.push(std::thread::spawn(move || {
            generate_crate_header(&path, &src_dir, &include_dir);
        }));
    }
    for handle in handles {
        handle.join().expect("header generation failed");
    }
}
