use crate::private::macros::*;

use crate::client_options::ClientOptionsT;
use crate::client_session::ClientSessionT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::{BsonT, ConstBsonT, bson_t};
use crate::runtime::RuntimeT;
use crate::spawn;
use crate::version::{MONGOAC_BUILD_PLATFORM, MONGOAC_VERSION_FULL};

use mongodb::bson::{Document, RawDocumentBuf};
use mongodb::{
    bson::{self, Bson},
    options::ListDatabasesOptions,
};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::ffi::c_char;
use std::sync::Arc;

pub struct ClientT {
    inner: mongodb::Client,
    runtime: RuntimeT,
    command_events: Option<Arc<Mutex<VecDeque<mongodb::event::command::CommandEvent>>>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_new(conn_str: *const c_char, error: *mut ErrorT) -> *mut ClientT {
    let error = safe_optional_error_as_mut!(error);
    let conn_str = safe_cstr_from_ptr_with_error!(conn_str, error);

    let client = safe_error!(ClientT::new(conn_str), error);
    Box::into_raw(Box::new(client))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_new_with_options(
    conn_str: *const c_char,
    options: *const ClientOptionsT,
    error: *mut ErrorT,
) -> *mut ClientT {
    let error = safe_optional_error_as_mut!(error);
    let conn_str = safe_cstr_from_ptr_with_error!(conn_str, error);
    let options = safe_optional_as_ref!(options);

    let client = safe_error!(ClientT::new_with_options(conn_str, options), error);
    Box::into_raw(Box::new(client))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_get_runtime(client: *const ClientT) -> *mut RuntimeT {
    let client = safe_as_ref!(client);
    Box::into_raw(Box::new(client.get_runtime()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_append_metadata(
    client: *mut ClientT,
    name: *const c_char,
    version: *const c_char,
    platform: *const c_char,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);

    let name = safe_optional_cstr_from_ptr_with_error!(name, error).unwrap_or_default();
    let version = safe_optional_cstr_from_ptr_with_error!(version, error);
    let platform = safe_optional_cstr_from_ptr_with_error!(platform, error);

    safe_error!(client.append_metadata(name, version, platform), error);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_count_command_events(client: *const ClientT) -> usize {
    let client = safe_as_ref!(client);
    client.count_command_events()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_get_command_event(
    client: *const ClientT,
    index: usize,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref!(client);

    match client.get_command_event(index) {
        Some(doc) => safe_error!(BsonT::try_from(&doc), error).into_raw(),
        None => Default::default(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_clear_command_events(client: *mut ClientT, n: usize) {
    let client = safe_as_mut!(client);

    client.clear_command_events(n);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_destroy(client: *mut ClientT) {
    safe_drop!(client);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_shutdown_async(
    client: *const ClientT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);

    Box::into_raw(Box::new(client.shutdown_async()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_shutdown(client: *const ClientT, error: *mut ErrorT) {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);

    safe_error!(client.shutdown(), error);
}

fn parse_session_options(
    bson: Option<ConstBsonT>,
) -> Result<Option<mongodb::options::SessionOptions>, ErrorT> {
    let bson = match bson {
        Some(r) => r,
        None => return Ok(None),
    };

    let opts =
        mongodb::bson::deserialize_from_reader(bson.as_bytes()).map_err(Into::<ErrorT>::into)?;
    Ok(Some(opts))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_start_session_async(
    client: *const ClientT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);

    let options = safe_optional_const_bson!(options);
    let session_opts = safe_error!(parse_session_options(options), error);

    let future = client.start_session_async(session_opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_start_session(
    client: *const ClientT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut ClientSessionT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);

    let options = safe_optional_const_bson!(options);
    let session_opts = safe_error!(parse_session_options(options), error);

    let session = safe_error!(client.start_session(session_opts), error);
    Box::into_raw(Box::new(session))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_databases_async(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let opts =
        safe_optional_bson_opts_with_error!(mongodb::options::ListDatabasesOptions, options, error);

    let future = client.list_databases_async(session, opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_databases(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let opts =
        safe_optional_bson_opts_with_error!(mongodb::options::ListDatabasesOptions, options, error);

    let docs = safe_error!(client.list_databases(session, opts), error);
    safe_error!(BsonT::try_from(&docs), error).into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_database_names_async(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let opts =
        safe_optional_bson_opts_with_error!(mongodb::options::ListDatabasesOptions, options, error);

    let future = client.list_database_names_async(session, opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_database_names(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let opts =
        safe_optional_bson_opts_with_error!(mongodb::options::ListDatabasesOptions, options, error);

    let names = safe_error!(client.list_database_names(session, opts), error);
    safe_error!(BsonT::try_from(&names), error).into_raw()
}

impl ClientT {
    fn new(conn_str: String) -> Result<ClientT, ErrorT> {
        let runtime = RuntimeT::new().map_err(|e| {
            ErrorT::from_mongoac(
                crate::error::ErrorCodeT::RuntimeError,
                &format!("runtime creation failed: {e}"),
            )
        })?;

        let command_events: Arc<Mutex<VecDeque<mongodb::event::command::CommandEvent>>> =
            Arc::new(Mutex::new(VecDeque::new()));
        let events_for_handler = Arc::clone(&command_events);

        let client = runtime.block_on(async move {
            let mut opts = mongodb::options::ClientOptions::parse(conn_str).await?;

            opts.driver_info = Some(
                mongodb::options::DriverInfo::builder()
                    .name("mongoac".to_string())
                    .version(MONGOAC_VERSION_FULL.to_string())
                    .platform(MONGOAC_BUILD_PLATFORM.to_string())
                    .build(),
            );

            opts.command_event_handler = Some(mongodb::event::EventHandler::callback(
                move |ev: mongodb::event::command::CommandEvent| {
                    events_for_handler.lock().push_back(ev);
                },
            ));

            mongodb::Client::with_options(opts)
        })?;

        Ok(ClientT {
            runtime,
            inner: client,
            command_events: Some(command_events),
        })
    }

    fn new_with_options(
        conn_str: String,
        options: Option<&ClientOptionsT>,
    ) -> Result<ClientT, ErrorT> {
        let options = match options {
            Some(o) => o,
            None => return Self::new(conn_str),
        };

        let runtime = RuntimeT::new().map_err(|e| {
            ErrorT::from_mongoac(
                crate::error::ErrorCodeT::RuntimeError,
                &format!("runtime creation failed: {e}"),
            )
        })?;

        let command_events: Option<Arc<Mutex<VecDeque<mongodb::event::command::CommandEvent>>>> =
            if options.capture_command_events() {
                Some(Arc::new(Mutex::new(VecDeque::new())))
            } else {
                None
            };

        let events_for_handler = command_events.as_ref().map(Arc::clone);

        let client = runtime.block_on(async move {
            let mut opts = mongodb::options::ClientOptions::parse(conn_str).await?;

            opts.driver_info = Some(
                mongodb::options::DriverInfo::builder()
                    .name("mongoac".to_string())
                    .version(MONGOAC_VERSION_FULL.to_string())
                    .platform(MONGOAC_BUILD_PLATFORM.to_string())
                    .build(),
            );

            if let Some(handler) = events_for_handler {
                opts.command_event_handler = Some(mongodb::event::EventHandler::callback(
                    move |ev: mongodb::event::command::CommandEvent| {
                        handler.lock().push_back(ev);
                    },
                ));
            }

            if let Some(server_api) = options.server_api() {
                opts.server_api = Some(server_api.clone());
            }

            mongodb::Client::with_options(opts)
        })?;

        Ok(ClientT {
            runtime,
            inner: client,
            command_events,
        })
    }

    fn append_metadata(
        &self,
        name: String,
        version: Option<String>,
        platform: Option<String>,
    ) -> Result<(), mongodb::error::Error> {
        let driver_info = mongodb::options::DriverInfo::builder()
            .name(name)
            .version(version)
            .platform(platform)
            .build();

        self.inner.append_metadata(driver_info)
    }

    pub(crate) fn get_runtime(&self) -> RuntimeT {
        self.runtime.clone()
    }

    pub(crate) fn inner(&self) -> &mongodb::Client {
        &self.inner
    }

    fn count_command_events(&self) -> usize {
        self.command_events.as_ref().map_or(0, |q| q.lock().len())
    }

    fn get_command_event(&self, index: usize) -> Option<RawDocumentBuf> {
        let guard = self.command_events.as_ref()?;
        let ev = guard.lock().get(index).cloned()?;
        mongodb::bson::serialize_to_raw_document_buf(&ev).ok()
    }

    fn clear_command_events(&mut self, n: usize) {
        if let Some(q) = &self.command_events {
            let mut guard = q.lock();
            let count = n.min(guard.len());
            guard.drain(..count);
        }
    }

    fn list_databases_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> FutureT {
        let client = self.inner.clone();
        let session_arc = session.map(|s| s.state.clone());

        spawn!(&self, Bson, async move {
            if let Some(ref session_arc) = session_arc {
                let mut guard = session_arc.lock().await;
                let specs = client
                    .list_databases()
                    .with_options(options)
                    .session(&mut *guard)
                    .await?;
                specs_to_bson_array(&specs)
            } else {
                let specs = client.list_databases().with_options(options).await?;
                specs_to_bson_array(&specs)
            }
        })
    }

    fn list_databases(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        let session_arc = session.map(|s| s.state.clone());
        self.runtime.block_on(async move {
            if let Some(ref session_arc) = session_arc {
                let mut guard = session_arc.lock().await;
                let specs = self
                    .inner
                    .list_databases()
                    .with_options(options)
                    .session(&mut *guard)
                    .await?;
                specs_to_bson_array(&specs)
            } else {
                let specs = self.inner.list_databases().with_options(options).await?;
                specs_to_bson_array(&specs)
            }
        })
    }

    fn list_database_names_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> FutureT {
        let client = self.inner.clone();
        let session_arc = session.map(|s| s.state.clone());

        spawn!(&self, Bson, async move {
            if let Some(ref session_arc) = session_arc {
                let mut guard = session_arc.lock().await;
                let names = client
                    .list_database_names()
                    .with_options(options)
                    .session(&mut *guard)
                    .await?;
                strings_to_bson(&names)
            } else {
                let names = client.list_database_names().with_options(options).await?;
                strings_to_bson(&names)
            }
        })
    }

    fn list_database_names(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        let session_arc = session.map(|s| s.state.clone());
        self.runtime.block_on(async move {
            if let Some(ref session_arc) = session_arc {
                let mut guard = session_arc.lock().await;
                let names = self
                    .inner
                    .list_database_names()
                    .with_options(options)
                    .session(&mut *guard)
                    .await?;
                strings_to_bson(&names)
            } else {
                let names = self
                    .inner
                    .list_database_names()
                    .with_options(options)
                    .await?;
                strings_to_bson(&names)
            }
        })
    }

    fn start_session_async(&self, options: Option<mongodb::options::SessionOptions>) -> FutureT {
        let client = self.inner.clone();

        spawn!(&self, ClientSession, async move {
            let mut builder = client.start_session();
            if let Some(opts) = options {
                builder = builder.with_options(opts);
            }
            builder.await.map(ClientSessionT::new)
        })
    }

    fn start_session(
        &self,
        options: Option<mongodb::options::SessionOptions>,
    ) -> Result<ClientSessionT, mongodb::error::Error> {
        self.runtime.block_on(async {
            let mut builder = self.inner.start_session();
            if let Some(opts) = options {
                builder = builder.with_options(opts);
            }
            let session = builder.await?;
            Ok(ClientSessionT::new(session))
        })
    }

    fn shutdown_async(&self) -> FutureT {
        let client = self.inner.clone();

        spawn!(&self, Void, async move {
            client.shutdown().await;
            Ok::<(), ErrorT>(())
        })
    }

    fn shutdown(&self) -> Result<(), ErrorT> {
        let client = self.inner.clone();

        self.runtime.block_on({
            async {
                client.shutdown().await;
                Ok(())
            }
        })
    }
}

fn specs_to_bson_array(
    specs: &[mongodb::results::DatabaseSpecification],
) -> Result<RawDocumentBuf, ErrorT> {
    let mut doc = Document::new();

    for (i, spec) in specs.iter().enumerate() {
        doc.insert(
            i.to_string(),
            Bson::Document(bson::serialize_to_document(spec)?),
        );
    }

    RawDocumentBuf::try_from(doc).map_err(Into::into)
}

pub(crate) fn strings_to_bson(names: &[String]) -> Result<RawDocumentBuf, ErrorT> {
    let mut doc = Document::new();

    for (i, name) in names.iter().enumerate() {
        doc.insert(i.to_string(), name);
    }

    RawDocumentBuf::try_from(doc).map_err(Into::into)
}
