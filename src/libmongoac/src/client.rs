use crate::database::DatabaseT;
use crate::database_options::DatabaseOptionsT;
use crate::private::macros::*;

use crate::bson::BsonT;
use crate::client_options::ClientOptionsT;
use crate::client_session::ClientSessionT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::list_databases_options::ListDatabasesOptionsT;
use crate::runtime::RuntimeT;
use crate::session_options::SessionOptionsT;
use crate::string::StringViewT;
use crate::version::{MONGOAC_BUILD_PLATFORM, MONGOAC_VERSION_FULL};
use crate::{op_with_session, spawn};

use mongodb::Client;
use mongodb::bson::{
    Bson, Document, RawDocumentBuf, serialize_to_document, serialize_to_raw_document_buf,
};
use mongodb::event::EventHandler;
use mongodb::event::command::CommandEvent;
use mongodb::options::ListDatabasesOptions;
use mongodb::options::{ClientOptions, DriverInfo, SessionOptions};
use mongodb::results::DatabaseSpecification;

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone)]
pub struct ClientT {
    inner: Client,
    runtime: RuntimeT,
    command_events: Option<Arc<Mutex<VecDeque<CommandEvent>>>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_destroy(client: *mut ClientT) {
    safe_drop!(client);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_clone(client: *const ClientT) -> *mut ClientT {
    Box::into_raw(Box::new(safe_as_ref!(client).clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_new(conn_str: StringViewT, error: *mut ErrorT) -> *mut ClientT {
    let error = safe_optional_error_as_mut!(error);
    let conn_str = safe_string_view_with_error!(conn_str, error);

    Box::into_raw(Box::new(safe_error!(ClientT::new(conn_str), error)))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_new_with_options(
    options: *const ClientOptionsT,
    error: *mut ErrorT,
) -> *mut ClientT {
    let error = safe_optional_error_as_mut!(error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(safe_error!(
        ClientT::new_with_options(options),
        error
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_get_runtime(client: *const ClientT) -> *mut RuntimeT {
    Box::into_raw(Box::new(safe_as_ref!(client).get_runtime()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_get_database(
    client: *const ClientT,
    name: StringViewT,
    options: *const DatabaseOptionsT,
    error: *mut ErrorT,
) -> *mut DatabaseT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let name = safe_string_view_with_error!(name, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(DatabaseT::new(
        client,
        name,
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_append_metadata(
    client: *mut ClientT,
    name: StringViewT,
    version: StringViewT,
    platform: StringViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_mut_with_error!(client, error);

    let name = safe_optional_string_view_with_error!(name, error).unwrap_or_default();
    let version = safe_optional_string_view_with_error!(version, error);
    let platform = safe_optional_string_view_with_error!(platform, error);

    safe_error!(client.append_metadata(name, version, platform), error);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_count_command_events(client: *const ClientT) -> usize {
    safe_as_ref!(client).count_command_events()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_get_command_event(client: *const ClientT, index: usize) -> BsonT {
    let client = safe_as_ref!(client);

    match client.get_command_event(index) {
        Some(doc) => doc.into(),
        None => BsonT::default(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_clear_command_events(client: *mut ClientT, n: usize) {
    safe_as_mut!(client).clear_command_events(n);
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

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_start_session_async(
    client: *const ClientT,
    options: *const SessionOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(
        client.start_session_async(options.map(Into::into)),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_start_session(
    client: *const ClientT,
    options: *const SessionOptionsT,
    error: *mut ErrorT,
) -> *mut ClientSessionT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(safe_error!(
        client.start_session(options.map(Into::into)),
        error
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_databases_async(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const ListDatabasesOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(
        client.list_databases_async(session, options.map(Into::into)),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_databases(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const ListDatabasesOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        client.list_databases(session, options.map(Into::into)),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_database_names_async(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const ListDatabasesOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(
        client.list_database_names_async(session, options.map(Into::into)),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_list_database_names(
    client: *const ClientT,
    session: *mut ClientSessionT,
    options: *const ListDatabasesOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        client.list_database_names(session, options.map(Into::into)),
        error
    )
    .into()
}

impl ClientT {
    fn new(conn_str: &str) -> Result<ClientT, ErrorT> {
        let runtime = make_runtime()?;

        let client = runtime.block_on(async move {
            let mut opts = ClientOptions::parse(conn_str).await?;

            opts.driver_info = Some(build_driver_info());

            Client::with_options(opts)
        })?;

        Ok(ClientT {
            runtime,
            inner: client,
            command_events: None,
        })
    }

    fn new_with_options(options: Option<&ClientOptionsT>) -> Result<ClientT, ErrorT> {
        let runtime = make_runtime()?;

        let command_events: Option<Arc<Mutex<VecDeque<CommandEvent>>>> =
            if options.is_some_and(|o| o.capture_command_events) {
                Some(Arc::new(Mutex::new(VecDeque::new())))
            } else {
                None
            };

        // Save user-provided metadata to append after mongoac's metadata.
        let user_driver_info = options
            .map(|o| o.inner.driver_info.clone())
            .unwrap_or_default();

        let client = {
            let command_event_handler = command_events.as_ref().map(Arc::clone);
            let mut options: ClientOptions = options.map(Into::into).unwrap_or_default();

            // Append mongoac's metadata before user-provided metadata.
            options.driver_info = Some(build_driver_info());

            if let Some(handler) = command_event_handler {
                options.command_event_handler =
                    Some(EventHandler::callback(move |ev: CommandEvent| {
                        handler.lock().push_back(ev);
                    }));
            }

            // Required to associate background tasks spawned by `Client::with_options()` with this runtime.
            let _guard = runtime.get_runtime().enter();

            Client::with_options(options)?
        };

        // Append user-provided metadata after mongoac's metadata.
        if let Some(driver_info) = user_driver_info {
            client.append_metadata(driver_info)?;
        }

        Ok(ClientT {
            runtime,
            inner: client,
            command_events,
        })
    }

    fn append_metadata(
        &mut self,
        name: &str,
        version: Option<&str>,
        platform: Option<&str>,
    ) -> Result<(), mongodb::error::Error> {
        let driver_info = DriverInfo::builder()
            .name(name)
            .version(version.map(Into::into))
            .platform(platform.map(Into::into))
            .build();

        self.inner.append_metadata(driver_info)
    }

    pub(crate) fn get_runtime(&self) -> RuntimeT {
        self.runtime.clone()
    }

    pub(crate) fn inner(&self) -> &Client {
        &self.inner
    }

    fn count_command_events(&self) -> usize {
        self.command_events.as_ref().map_or(0, |q| q.lock().len())
    }

    fn get_command_event(&self, index: usize) -> Option<RawDocumentBuf> {
        let guard = self.command_events.as_ref()?;
        let ev = guard.lock().get(index).cloned()?;
        serialize_to_raw_document_buf(&ev).ok()
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
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let res = op_with_session!(client.list_databases().with_options(options), session)?;
            specs_to_bson_array(&res)
        })
    }

    fn list_databases(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res = op_with_session!(self.inner.list_databases().with_options(options), session)?;
            specs_to_bson_array(&res)
        })
    }

    fn list_database_names_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> FutureT {
        let client = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let res =
                op_with_session!(client.list_database_names().with_options(options), session)?;
            strings_to_bson(&res)
        })
    }

    fn list_database_names(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListDatabasesOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res = op_with_session!(
                self.inner.list_database_names().with_options(options),
                session
            )?;
            strings_to_bson(&res)
        })
    }

    fn start_session_async(&self, options: Option<SessionOptions>) -> FutureT {
        let client = self.inner.clone();

        spawn!(self, ClientSession, async move {
            client
                .start_session()
                .with_options(options)
                .await
                .map(ClientSessionT::new)
                .map_err(ErrorT::from)
        })
    }

    fn start_session(&self, options: Option<SessionOptions>) -> Result<ClientSessionT, ErrorT> {
        self.runtime.block_on(async {
            self.inner
                .start_session()
                .with_options(options)
                .await
                .map(ClientSessionT::new)
                .map_err(Into::into)
        })
    }

    fn shutdown_async(&self) -> FutureT {
        let client = self.inner.clone();

        spawn!(self, Void, async move {
            client.shutdown().await;
            Ok(())
        })
    }

    fn shutdown(&self) -> Result<(), ErrorT> {
        let client = self.inner.clone();

        self.runtime.block_on(async {
            client.shutdown().await;
            Ok(())
        })
    }
}

fn make_runtime() -> Result<RuntimeT, ErrorT> {
    RuntimeT::new().map_err(|e| {
        ErrorT::from_mongoac(
            crate::error::ErrorCodeT::RuntimeError,
            &format!("runtime creation failed: {e}"),
        )
    })
}

fn build_driver_info() -> DriverInfo {
    DriverInfo::builder()
        .name("mongoac".to_string())
        .version(Some(MONGOAC_VERSION_FULL.into()))
        .platform(Some(MONGOAC_BUILD_PLATFORM.into()))
        .build()
}

fn specs_to_bson_array(specs: &[DatabaseSpecification]) -> Result<RawDocumentBuf, ErrorT> {
    let mut doc = Document::new();

    for (i, spec) in specs.iter().enumerate() {
        doc.insert(
            i.to_string(),
            Bson::Document(serialize_to_document(spec)?), // Deep-copy!
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
