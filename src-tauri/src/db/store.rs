/// SQLite + SQLCipher store — stub.
///
/// Full implementation will use `rusqlite` with the `sqlcipher` feature flag.
/// The encryption key is derived from the OS keychain, never hardcoded.
///
/// Schema is managed via `schema.sql`; migrations run on startup.
pub struct Store;

impl Store {
    /// Open (or create) the encrypted database at the app's local data path.
    pub fn open(_path: &str) -> Result<Self, String> {
        // TODO: rusqlite::Connection::open_with_flags(path, ...)
        //       + PRAGMA key = '<keychain-derived-key>';
        Ok(Store)
    }

    /// Run all pending migrations from schema.sql.
    pub fn migrate(&self) -> Result<(), String> {
        // TODO: execute CREATE TABLE IF NOT EXISTS statements
        Ok(())
    }
}
