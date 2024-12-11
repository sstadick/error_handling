use anyhow::{self, Context};
use std::{
    error::Error,
    fs::File,
    io::{self, Read},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    IoError { source: io::Error, file: String },
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type DynError = Box<dyn Error>;
type DynResult<T> = Result<T, DynError>;

/// The most explicit errors, you must define all the variants of the errors and unify them up your stack yourself.
/// If you are able to `#[from]` most sources, this isn't too bad.
/// But is it worth the effort? In what scenarios are we recovering a program based on the type of the error?
pub fn thiserror_function(file: &str) -> Result<(), DataStoreError> {
    let mut fh = File::open(file).map_err(|e| DataStoreError::IoError {
        source: e,
        file: file.to_string(),
    })?;
    let mut contents = String::new();
    fh.read_to_string(&mut contents)
        .map_err(|e| DataStoreError::IoError {
            source: e,
            file: file.to_string(),
        })?;
    println!("{:?}", contents);
    Ok(())
}

/// The anyhow type is very much like a Box<dyn Error> in that it will convert any error types to an [`anyhow::Error`].
/// However, it adds helper methods like `.context`, and `anyhow!` for creating error messages on the fly.
/// Additionally, as shown above, thiserror can source anyhow errors.
/// Lastly, this serves as a universal error type, you don't have to have a hierarchy of errors.
pub fn anyhow_function(file: &str) -> anyhow::Result<()> {
    let mut fh = File::open(file).with_context(|| format!("Failed to open {}", file))?;
    let mut contents = String::new();
    fh.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read {}", file))?;
    println!("{:?}", contents);
    Ok(())
}

/// With this method it is much harder to return additional context.
/// In the very simple case, where you have a [`DataSource`] error variant that is `#[from] io::Error`, then yes there is no change needed.
/// But if you need to map to struct fields for file names and such, you still have to make changes.
/// Lastly, you lose information doing it this way, the "history" of the error is erased
pub fn dyn_function(file: &str) -> DynResult<()> {
    let mut fh = File::open(file)
        .map_err(|e| io::Error::new(e.kind(), format!("Failed to open {}", file)))?;
    let mut contents = String::new();
    fh.read_to_string(&mut contents)
        .map_err(|e| io::Error::new(e.kind(), format!("Failed to read {}", file)))?;
    println!("{:?}", contents);
    Ok(())
}

fn main() -> DynResult<()> {
    // Thiserror error: IoError { source: Os { code: 2, kind: NotFound, message: "No such file or directory" }, file: "myfile.txt" }
    if let Err(e) = thiserror_function("myfile.txt") {
        println!("Thiserror error: {:?}", e);
    }
    // Anyhow error: Failed to open myfile.txt

    // Caused by:
    //     No such file or directory (os error 2)
    if let Err(e) = anyhow_function("myfile.txt") {
        println!("Anyhow error: {:?}", e);
    }
    // BoxDyn error: Custom { kind: NotFound, error: "Failed to open myfile.txt" }
    if let Err(e) = dyn_function("myfile.txt") {
        println!("BoxDyn error: {:?}", e);
    }
    Ok(())
}
