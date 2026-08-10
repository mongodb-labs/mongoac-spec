use std::path::{Path, PathBuf};

macro_rules! includes {
    ($($h:expr),* $(,)?) => {
        vec![$($h.into()),*]
    };
}

// List include directives needed by each crate header.
fn configure(name: &str, mut config: cbindgen::Config) -> cbindgen::Config {
    // Rename structs from `ExampleT` to `mongoac_example_t`.
    for (from, to) in [
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
    ] {
        config
            .export
            .rename
            .insert(from.to_string(), to.to_string());
    }

    config.sys_includes = match name {
        "bson" => includes!["mongoac/export.h", "stdint.h"],
        "aggregate_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/read_concern.h",
            "mongoac/write_concern.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "sanity_check" => includes!["mongoac/export.h", "stdint.h"],
        "client" => includes![
            "mongoac/export.h",
            "mongoac/client_options.h",
            "mongoac/client_session.h",
            "mongoac/error.h",
            "mongoac/future.h",
            "mongoac/list_databases_options.h",
            "mongoac/session_options.h",
            "mongoac/runtime.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "client_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/server_api.h",
            "mongoac/read_concern.h",
            "mongoac/write_concern.h",
            "mongoac/read_preference.h",
            "mongoac/server_selector.h",
            "mongoac/tls.h",
            "mongoac/credential.h",
            "stdbool.h",
            "stdint.h",
        ],
        "client_session" => includes!["mongoac/export.h", "mongoac/future.h"],
        "collection" => includes![
            "mongoac/export.h",
            "mongoac/database.h",
            "mongoac/error.h",
            "mongoac/future.h",
            "mongoac/drop_collection_options.h",
            "mongoac/insert_one_options.h",
            "mongoac/insert_many_options.h",
            "mongoac/find_options.h",
            "mongoac/find_one_options.h",
            "mongoac/delete_options.h",
            "mongoac/replace_options.h",
            "mongoac/update_options.h",
            "mongoac/count_options.h",
            "mongoac/estimated_document_count_options.h",
            "mongoac/distinct_options.h",
            "mongoac/aggregate_options.h",
            "mongoac/collection_options.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "create_collection_options" => {
            includes!["mongoac/export.h", "mongoac/error.h", "mongoac/bson.h"]
        }
        "collection_options" => includes![
            "mongoac/export.h",
            "mongoac/read_concern.h",
            "mongoac/write_concern.h",
            "mongoac/read_preference.h",
            "mongoac/server_selector.h",
        ],
        "credential" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "cursor_type" => includes!["stdint.h"],
        "cursor" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/future.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "database" => includes![
            "mongoac/export.h",
            "mongoac/client.h",
            "mongoac/client_session.h",
            "mongoac/cursor.h",
            "mongoac/error.h",
            "mongoac/future.h",
            "mongoac/list_collections_options.h",
            "mongoac/drop_database_options.h",
            "mongoac/database_options.h",
            "mongoac/create_collection_options.h",
            "mongoac/run_command_options.h",
            "mongoac/run_cursor_command_options.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "database_options" => includes![
            "mongoac/export.h",
            "mongoac/read_concern.h",
            "mongoac/write_concern.h",
            "mongoac/read_preference.h",
            "mongoac/server_selector.h",
        ],
        "drop_collection_options" => includes!["mongoac/export.h", "mongoac/write_concern.h"],
        "delete_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/write_concern.h",
            "mongoac/bson.h",
        ],
        "replace_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/write_concern.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "update_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/write_concern.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "count_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/read_concern.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "estimated_document_count_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/read_concern.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "distinct_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/read_concern.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "drop_database_options" => includes!["mongoac/export.h", "mongoac/write_concern.h"],
        "error" => includes![
            "mongoac/export.h",
            "mongoac/string.h",
            "stdbool.h",
            "stdint.h"
        ],
        "find_options" => includes!["mongoac/export.h", "mongoac/error.h", "mongoac/bson.h"],
        "find_one_options" => includes!["mongoac/export.h", "mongoac/error.h", "mongoac/bson.h"],
        "future" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "insert_many_options" => includes![
            "mongoac/export.h",
            "mongoac/write_concern.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "insert_one_options" => includes![
            "mongoac/export.h",
            "mongoac/write_concern.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "list_collections_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "list_databases_options" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/bson.h",
            "stdbool.h",
        ],
        "read_concern" => includes!["mongoac/export.h", "mongoac/error.h", "stdint.h"],
        "run_command_options" => includes![
            "mongoac/export.h",
            "mongoac/read_preference.h",
            "mongoac/server_selector.h",
        ],
        "run_cursor_command_options" => includes![
            "mongoac/export.h",
            "mongoac/cursor_type.h",
            "mongoac/read_preference.h",
            "mongoac/server_selector.h",
            "mongoac/error.h",
            "mongoac/bson.h",
            "stdint.h",
        ],
        "read_preference" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/bson.h",
            "stdbool.h",
            "stdint.h",
        ],
        "runtime" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "mongoac/future.h",
            "stdint.h"
        ],
        "server_api" => includes!["mongoac/export.h", "stdbool.h"],
        "server_info" => includes![
            "mongoac/export.h",
            "mongoac/bson.h",
            "mongoac/string.h",
            "stdbool.h",
            "stdint.h",
        ],
        "server_selector" => includes![
            "mongoac/export.h",
            "mongoac/server_info.h",
            "stdbool.h",
            "stdint.h",
        ],
        "session_options" => includes![
            "mongoac/export.h",
            "mongoac/transaction_options.h",
            "stdbool.h",
            "stdint.h",
        ],
        "string" => includes!["mongoac/export.h", "stdint.h"],
        "tls" => includes!["mongoac/export.h", "mongoac/error.h", "stdbool.h"],
        "transaction_options" => includes![
            "mongoac/export.h",
            "mongoac/read_concern.h",
            "mongoac/write_concern.h",
            "mongoac/read_preference.h",
            "stdint.h",
        ],
        "write_concern" => includes![
            "mongoac/export.h",
            "mongoac/error.h",
            "stdbool.h",
            "stdint.h",
        ],
        _ => vec![],
    };
    config
}

fn generate_crate_header(crate_path: &Path, src_dir: &Path, include_dir: &Path) {
    let file_stem = crate_path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("invalid UTF-8");

    // These crates must not generate a header.
    // Keep synchronized with `skip_cargo_headers` in src/libmongoac/CMakeLists.txt.
    const SKIP_CARGO_HEADERS: &[&str] = &["lib", "mod", "version"];
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
    let config = cbindgen::Config {
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
        documentation: true,
        documentation_style: cbindgen::DocumentationStyle::C99,
        function: cbindgen::FunctionConfig {
            prefix: Some("MONGOAC_API".to_owned()),
            ..Default::default()
        },
        no_includes: true,
        ..Default::default()
    };

    // Apply per-crate configuration options.
    let config = configure(rel_str, config);

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
