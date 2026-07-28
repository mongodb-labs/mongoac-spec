use mongodb::options::{DropCollectionOptions, WriteConcern};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct DropCollectionOptionsT {
    #[serde(alias = "writeConcern")]
    write_concern: Option<WriteConcern>,
}

impl From<DropCollectionOptionsT> for DropCollectionOptions {
    fn from(opts: DropCollectionOptionsT) -> Self {
        DropCollectionOptions::builder()
            .write_concern(opts.write_concern)
            // .encrypted_fields(opts.encrypted_fields)
            .build()
    }
}
