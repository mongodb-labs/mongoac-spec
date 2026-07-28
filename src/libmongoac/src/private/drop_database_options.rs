use mongodb::options::{DropDatabaseOptions, WriteConcern};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DropDatabaseOptionsT {
    #[serde(alias = "writeConcern")]
    write_concern: Option<WriteConcern>,
}

impl From<DropDatabaseOptionsT> for DropDatabaseOptions {
    fn from(opts: DropDatabaseOptionsT) -> Self {
        DropDatabaseOptions::builder()
            .write_concern(opts.write_concern)
            .build()
    }
}
