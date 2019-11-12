//! # About
//!
//! A small-footprint database implamentation, originally designed for the
//! [Zeno](https://gitlab.com/zeno-src/zeno) code editor. This database does not
//! accept duplicates and will not save a second identical item.
//!
//! Under the surface, tinydb uses a [HashSet]-based table that works in a similar
//! fashion to SQL-like/Grid based databases.
//!
//! # Disclaimer
//!
//! This project is not intended to be used inside of any critical systems due to
//! the nature of dumping/recovery. If you are using this crate as a temporary and
//! in-memory only database, it should preform at a reasonable speed (as it uses
//! [HashSet] underneith).

use std::collections::HashSet;
use std::fs::File;
use std::hash;
use std::io::prelude::*;
use std::path::PathBuf;

/// Database error enum
#[derive(Debug)]
pub enum DatabaseError {
    /// When the item queried for was not found
    ItemNotFound,

    /// A duplicate value was found when adding to the database with
    /// [Database::strict_dupes] allowed.
    DupeFound,
    /// When [Database::save_path] is required but is not found. This commonly
    /// happens when loading or dumping a database with [Database::save_path]
    /// being [Option::None].
    SavePathRequired,

    /// Misc [std::io::Error] that could not be properly handled.
    IOError(std::io::Error),
}

/// The primary database structure, allowing storage of a given generic.
///
/// The generic type used should primarily be structures as they resemble a
/// conventional database model and should implament [hash::Hash] and [Eq].
///
/// # Essential operations
///
/// - Create: [Database::new]   
/// - Query: [Database::query_item]
/// - Update: [Database::update_item]
/// - Delete: [Database::remove_item]
/// - Read all: [Database::read_db]
/// - Dump: [Database::dump_db]
/// - Load: [Database::load_db]
pub struct Database<T: hash::Hash + Eq> {
    pub label: String,
    pub save_path: Option<PathBuf>,
    pub strict_dupes: bool,
    items: HashSet<T>,
}

impl<T: hash::Hash + Eq> Database<T> {
    /// Creates a new database instance.
    pub fn new(label: String, save_path: Option<PathBuf>, strict_dupes: bool) -> Self {
        Database {
            label: label,
            save_path: save_path,
            strict_dupes: strict_dupes,
            items: HashSet::new(),
        }
    }

    /// Adds a new item to the in-memory database.
    pub fn add_item(&mut self, item: T) -> Result<(), DatabaseError> {
        if self.strict_dupes {
            if self.items.contains(&item) {
                return Err(DatabaseError::DupeFound);
            }
        }

        self.items.insert(item);
        return Ok(());
    }

    /// Removes an item from the database.
    /// 
    /// # Errors
    /// 
    /// Will return [DatabaseError::ItemNotFound] if the item that is attempting
    /// to be deleted was not found.
    pub fn remove_item(&mut self, item: T) -> Result<(), DatabaseError> {
        if self.items.remove(&item) {
            Ok(())
        } else {
            Err(DatabaseError::ItemNotFound)
        }
    }

    /// Query the database for a specific item.
    pub fn query_item(&mut self, item: T) -> Option<&T> {
        self.items.get(&item)
    }

    /// Loads all into database from a `.tinydb` file and **erases any current
    /// in-memory data**.
    ///
    /// # Loading path methods
    /// 
    /// The database will usually try to load `\[label\].tinydb` where `\[label\]`
    /// is the defined [Database::label] (path is reletive to where tinydb was
    /// executed).
    ///
    /// You can also overwrite this behaviour by defining a [Database::save_path]
    /// when generating the database inside of [Database::new].
    pub fn load_db(&self) {
        unimplemented!();
    }

    /// Dumps/saves database to a binary file.
    /// 
    /// # Saving path methods
    /// 
    /// The database will usually save as `\[label\].tinydb` where `\[label\]`
    /// is the defined [Database::label] (path is reletive to where tinydb was
    /// executed).
    ///
    /// You can also overwrite this behaviour by defining a [Database::save_path]
    /// when generating the database inside of [Database::new].
    pub fn dump_db(&self) -> Result<(), DatabaseError> {
        let u8_dump: &[u8] = unsafe { any_as_u8_slice(self) };

        let mut dump_file = self.open_db_path()?;

        io_to_dberror(dump_file.write_all(u8_dump))?;

        Ok(())
    }

    /// Opens the path given in [Database::save_path] or returns a [DatabaseError].
    fn open_db_path(&self) -> Result<File, DatabaseError> {
        let definate_path = path_to_dberror(self.save_path.as_ref())?;

        if definate_path.exists() {
            io_to_dberror(std::fs::remove_file(&definate_path))?;
        }

        io_to_dberror(File::create(&definate_path))
    }
}

/// Converts a possible [std::io::Error] to a [DatabaseError].
fn io_to_dberror<T>(io_res: Result<T, std::io::Error>) -> Result<T, DatabaseError> {
    match io_res {
        Ok(x) => Ok(x),
        Err(e) => Err(DatabaseError::IOError(e)),
    }
}

/// Converts an [Option]<[PathBuf]> into a [Result]<[PathBuf], [DatabaseError]>.
fn path_to_dberror(path: Option<&PathBuf>) -> Result<PathBuf, DatabaseError> {
    match path {
        None => Err(DatabaseError::SavePathRequired),
        Some(x) => Ok(x.to_owned()),
    }
}

/// Converts a [Sized] generic to a u8 slice.
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dummy struct to use inside of tests
    #[derive(Hash, Eq, PartialEq, Debug)]
    struct DemoStruct {
        name: String,
        age: i32,
    }

    /// Tests addition to in-memory db
    #[test]
    fn item_add() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(String::from("Adding test"), None, true);

        my_db.add_item(DemoStruct {
            name: String::from("John"),
            age: 16,
        })?;

        Ok(())
    }

    /// Tests removal from in-memory db
    #[test]
    fn item_remove() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(String::from("Adding test"), None, true);

        let testing_struct = DemoStruct {
            name: String::from("Xander"),
            age: 33,
        };

        my_db.add_item(&testing_struct)?;
        my_db.remove_item(&testing_struct)?;

        Ok(())
    }

    #[test]
    fn db_dump() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(
            String::from("Adding test"),
            Some(PathBuf::from("db/test.tinydb")),
            true,
        );

        for _ in 0..1 {
            let testing_struct = DemoStruct {
                name: String::from("Xander"),
                age: 33,
            };
            let other = DemoStruct {
                name: String::from("John"),
                age: 54,
            };
            my_db.add_item(testing_struct)?;
            my_db.add_item(other)?;
        }

        my_db.dump_db()?;

        Ok(())
    }
}