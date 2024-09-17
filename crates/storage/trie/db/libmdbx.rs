use std::{marker::PhantomData, sync::Arc};

use crate::error::StoreError;
use libmdbx::{
    orm::{table, Database, Table},
    table_info,
};

/// Libmdbx implementation for the TrieDB trait, with get and put operations.
pub struct LibmdbxTrieDB<T: Table> {
    db: Arc<Database>,
    phantom: PhantomData<T>,
}

use super::TrieDB;

impl<T> LibmdbxTrieDB<T>
where
    T: Table<Key = Vec<u8>, Value = Vec<u8>>,
{
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            phantom: PhantomData,
        }
    }
}

impl<T> TrieDB for LibmdbxTrieDB<T>
where
    T: Table<Key = Vec<u8>, Value = Vec<u8>>,
{
    fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, StoreError> {
        let txn = self.db.begin_read().map_err(StoreError::LibmdbxError)?;
        txn.get::<T>(key).map_err(StoreError::LibmdbxError)
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StoreError> {
        let txn = self
            .db
            .begin_readwrite()
            .map_err(StoreError::LibmdbxError)?;
        txn.upsert::<T>(key, value)
            .map_err(StoreError::LibmdbxError)?;
        txn.commit().map_err(StoreError::LibmdbxError)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::LibmdbxTrieDB;
    use crate::trie::{test_utils::new_db, Trie};
    use libmdbx::{
        orm::{table, Database, Table},
        table_info,
    };
    table!(
        /// NodeHash to Node table
        ( Nodes )  Vec<u8> => Vec<u8>
    );

    use crate::trie::TrieDB;

    #[test]
    fn simple_addition() {
        let inner_db = new_db::<Nodes>();
        let db = LibmdbxTrieDB::<Nodes>::new(inner_db);
        assert_eq!(db.get("hello".into()).unwrap(), None);
        db.put("hello".into(), "value".into());
        assert_eq!(db.get("hello".into()).unwrap(), Some("value".into()));
    }

    #[test]
    fn different_tables() {
        table!(
            /// vec to vec
            ( TableA ) Vec<u8> => Vec<u8>
        );
        table!(
            /// vec to vec
            ( TableB ) Vec<u8> => Vec<u8>
        );
        let tables = [table_info!(TableA), table_info!(TableB)]
            .into_iter()
            .collect();

        let inner_db = Arc::new(Database::create(None, &tables).unwrap());
        let db_a = LibmdbxTrieDB::<TableA>::new(inner_db.clone());
        let db_b = LibmdbxTrieDB::<TableB>::new(inner_db.clone());
        db_a.put("hello".into(), "value".into());
        assert_eq!(db_b.get("hello".into()).unwrap(), None);
    }
}
