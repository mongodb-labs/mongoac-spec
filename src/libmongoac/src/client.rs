use crate::{
    safe_as_mut, safe_as_ref, safe_as_ref_with_error, safe_cstr_from_ptr_with_error, safe_drop,
    safe_optional_as_mut, safe_optional_as_ref, safe_optional_cstr_from_ptr_with_error,
    safe_optional_error_as_mut,
};

use crate::client_options::ClientOptionsT;
use crate::client_session::ClientSessionT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::bson_t;
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

impl ClientT {
    fn new(conn_str: String) -> Result<ClientT, mongodb::error::Error> {
        let runtime = RuntimeT::new().map_err(|e| {
            mongodb::error::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("runtime creation failed: {e}"),
            ))
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
    ) -> Result<ClientT, mongodb::error::Error> {
        let options = match options {
            Some(o) => o,
            None => return Self::new(conn_str),
        };

        let runtime = RuntimeT::new().map_err(|e| {
            mongodb::error::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("runtime creation failed: {e}"),
            ))
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

    pub(crate) fn client(&self) -> &mongodb::Client {
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
        let session_arc = session.map(|s| s.inner.clone());

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

    fn list_database_names_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> FutureT {
        let client = self.inner.clone();
        let session_arc = session.map(|s| s.inner.clone());

        spawn!(&self, Bson, async move {
            if let Some(ref session_arc) = session_arc {
                let mut guard = session_arc.lock().await;
                let names = client
                    .list_database_names()
                    .with_options(options)
                    .session(&mut *guard)
                    .await?;
                names_to_bson_array(&names)
            } else {
                let names = client.list_database_names().with_options(options).await?;
                names_to_bson_array(&names)
            }
        })
    }

    fn list_databases(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> Result<Document, mongodb::error::Error> {
        let session_arc = session.map(|s| s.inner.clone());
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

    fn list_database_names(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> Result<Document, mongodb::error::Error> {
        let session_arc = session.map(|s| s.inner.clone());
        self.runtime.block_on(async move {
            if let Some(ref session_arc) = session_arc {
                let mut guard = session_arc.lock().await;
                let names = self
                    .inner
                    .list_database_names()
                    .with_options(options)
                    .session(&mut *guard)
                    .await?;
                names_to_bson_array(&names)
            } else {
                let names = self
                    .inner
                    .list_database_names()
                    .with_options(options)
                    .await?;
                names_to_bson_array(&names)
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
            builder.await.map(|session| ClientSessionT::new(session))
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
}

fn specs_to_bson_array(
    specs: &[mongodb::results::DatabaseSpecification],
) -> Result<Document, mongodb::error::Error> {
    let mut doc = Document::new();

    for (i, spec) in specs.iter().enumerate() {
        doc.insert(
            i.to_string(),
            Bson::Document(bson::serialize_to_document(spec)?),
        );
    }

    Ok(doc)
}

fn names_to_bson_array(names: &[String]) -> Result<Document, mongodb::error::Error> {
    let mut doc = Document::new();

    for (i, name) in names.iter().enumerate() {
        doc.insert(i.to_string(), name);
    }

    Ok(doc)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_new(conn_str: *const c_char, error: *mut ErrorT) -> *mut ClientT {
    let error = safe_optional_error_as_mut!(error);
    let conn_str = safe_cstr_from_ptr_with_error!(conn_str, error);

    match ClientT::new(conn_str) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(err) => {
            if let Some(e) = error {
                *e = ErrorT::from_mongodb(&err);
            }
            Default::default()
        }
    }
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

    match ClientT::new_with_options(conn_str, options) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(err) => {
            if let Some(e) = error {
                *e = ErrorT::from_mongodb(&err);
            }
            Default::default()
        }
    }
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

    match client.append_metadata(name, version, platform) {
        Ok(_) => {}
        Err(err) => {
            if let Some(e) = error {
                *e = ErrorT::from_mongodb(&err);
            }
        }
    }
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
        Some(raw) => match bson_t::from_raw_document_buf(&raw) {
            Ok(ptr) => ptr,
            Err(err) => {
                if let Some(e) = error {
                    *e = ErrorT::from_bson(&err);
                }
                Default::default()
            }
        },
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

fn parse_session_options(
    bson: *const bson_t,
) -> Result<Option<mongodb::options::SessionOptions>, ErrorT> {
    let bson = match safe_optional_as_ref!(bson) {
        Some(r) => r,
        None => return Ok(None),
    };
    let doc = bson.to_document().map_err(|e| ErrorT::from_bson(&e))?;
    let opts = mongodb::bson::deserialize_from_document(doc).map_err(|e| ErrorT::from_bson(&e))?;
    Ok(Some(opts))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_start_session_async(
    client: *mut ClientT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);

    let session_opts = match parse_session_options(options) {
        Ok(opts) => opts,
        Err(err) => {
            if let Some(e) = error {
                *e = err;
            }
            return Default::default();
        }
    };

    let future = client.start_session_async(session_opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_start_session(
    client: *mut ClientT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut ClientSessionT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);

    let session_opts = match parse_session_options(options) {
        Ok(opts) => opts,
        Err(err) => {
            if let Some(e) = error {
                *e = err;
            }
            return Default::default();
        }
    };

    match client.start_session(session_opts) {
        Ok(session) => Box::into_raw(Box::new(session)),
        Err(err) => {
            if let Some(e) = error {
                *e = ErrorT::from_mongodb(&err);
            }
            Default::default()
        }
    }
}

fn parse_options(bson: *const bson_t) -> Result<Option<ListDatabasesOptions>, ErrorT> {
    let bson = match safe_optional_as_ref!(bson) {
        Some(r) => r,
        None => return Ok(None),
    };
    let doc = bson.to_document().map_err(|e| ErrorT::from_bson(&e))?;
    let opts = mongodb::bson::deserialize_from_document(doc).map_err(|e| ErrorT::from_bson(&e))?;
    Ok(Some(opts))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_databases_async(
    client: *mut ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let future = match options {
        Some(opts) => match parse_options(opts) {
            Ok(opts) => client.list_databases_async(session, opts),
            Err(err) => {
                if let Some(e) = error {
                    *e = err;
                }
                return Default::default();
            }
        },
        None => client.list_databases_async(session, None),
    };

    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_databases(
    client: *mut ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let opts = match options {
        Some(opts) => match parse_options(opts) {
            Ok(opts) => opts,
            Err(err) => {
                if let Some(e) = error {
                    *e = err;
                }
                return Default::default();
            }
        },
        None => None,
    };

    match client.list_databases(session, opts) {
        Ok(doc) => match bson_t::from_document(&doc) {
            Ok(ptr) => ptr,
            Err(err) => {
                if let Some(e) = error {
                    *e = ErrorT::from_bson(&err);
                }
                Default::default()
            }
        },
        Err(err) => {
            if let Some(e) = error {
                *e = ErrorT::from_mongodb(&err);
            }
            Default::default()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_database_names_async(
    client: *mut ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let future = match options {
        Some(opts) => match parse_options(opts) {
            Ok(opts) => client.list_database_names_async(session, opts),
            Err(err) => {
                if let Some(e) = error {
                    *e = err;
                }
                return Default::default();
            }
        },
        None => client.list_database_names_async(session, None),
    };

    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_database_names(
    client: *mut ClientT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let opts = match options {
        Some(opts) => match parse_options(opts) {
            Ok(opts) => opts,
            Err(err) => {
                if let Some(e) = error {
                    *e = err;
                }
                return Default::default();
            }
        },
        None => None,
    };

    match client.list_database_names(session, opts) {
        Ok(doc) => match bson_t::from_document(&doc) {
            Ok(ptr) => ptr,
            Err(err) => {
                if let Some(e) = error {
                    *e = ErrorT::from_bson(&err);
                }
                Default::default()
            }
        },
        Err(err) => {
            if let Some(e) = error {
                *e = ErrorT::from_mongodb(&err);
            }
            Default::default()
        }
    }
}
