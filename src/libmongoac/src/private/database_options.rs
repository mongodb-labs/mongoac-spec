use mongodb::options::{
    DatabaseOptions, ReadConcern, ReadPreference, SelectionCriteria, WriteConcern,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct DatabaseOptionsT {
    #[serde(alias = "readConcern")]
    read_concern: Option<ReadConcern>,

    #[serde(alias = "readPreference")]
    read_preference: Option<ReadPreference>,

    #[serde(alias = "writeConcern")]
    write_concern: Option<WriteConcern>,
}

impl From<DatabaseOptionsT> for DatabaseOptions {
    fn from(opts: DatabaseOptionsT) -> Self {
        DatabaseOptions::builder()
            .read_concern(opts.read_concern)
            .selection_criteria(opts.read_preference.map(SelectionCriteria::ReadPreference))
            .write_concern(opts.write_concern)
            .build()
    }
}
