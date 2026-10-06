use std::ffi::c_void;

use libbitcoinkernel_sys::{
    btck_LogCategory, btck_LogCategory_ALL, btck_LogCategory_BENCH, btck_LogCategory_BLOCKSTORAGE,
    btck_LogCategory_COINDB, btck_LogCategory_KERNEL, btck_LogCategory_LEVELDB,
    btck_LogCategory_LOCK, btck_LogCategory_MEMPOOL, btck_LogCategory_PRUNE, btck_LogCategory_RAND,
    btck_LogCategory_REINDEX, btck_LogCategory_TXPACKAGES, btck_LogCategory_UNKNOWN,
    btck_LogCategory_VALIDATION, btck_LogEntry, btck_LogLevel, btck_LogLevel_DEBUG,
    btck_LogLevel_ERROR, btck_LogLevel_INFO, btck_LogLevel_TRACE, btck_LogLevel_WARNING,
    btck_LoggingConnection, btck_logging_connection_create, btck_logging_connection_destroy,
    btck_logging_set_min_level,
};

use crate::{ffi::c_helpers, KernelError};

/// A function for handling log messages produced by the kernel library.
pub trait Log {
    fn log(&self, message: &str);
}

unsafe extern "C" fn log_callback<T: Log + 'static>(
    user_data: *mut c_void,
    entry: *const btck_LogEntry,
) {
    if entry.is_null() {
        return;
    }
    let entry = &*entry;
    let message = c_helpers::to_string(entry.message, entry.message_len);
    let log = user_data as *mut T;
    (*log).log(&message);
}

unsafe extern "C" fn destroy_log_callback<T>(user_data: *mut c_void) {
    if !user_data.is_null() {
        let _ = Box::from_raw(user_data as *mut T);
    }
}

/// Delivers kernel log entries through a user callback.
///
/// Attach the logger to a [`crate::ContextBuilder`] with
/// [`crate::ContextBuilder::logger`] so context work receives entries.
/// The connection must outlive every context that uses it.
pub struct Logger {
    pub(crate) inner: *mut btck_LoggingConnection,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl Drop for Logger {
    fn drop(&mut self) {
        unsafe {
            btck_logging_connection_destroy(self.inner);
        }
    }
}

impl Logger {
    /// Create a new Logger with the specified callback.
    pub fn new<T: Log + 'static>(log: T) -> Result<Logger, KernelError> {
        let log_ptr = Box::into_raw(Box::new(log));

        let inner = unsafe {
            btck_logging_connection_create(
                log_callback::<T>,
                log_ptr as *mut c_void,
                Some(destroy_log_callback::<T>),
            )
        };

        if inner.is_null() {
            unsafe {
                let _ = Box::from_raw(log_ptr);
            }
            return Err(KernelError::Internal(
                "Failed to create new logging connection.".to_string(),
            ));
        }

        Ok(Logger { inner })
    }

    /// Set the minimum log level for this connection. Entries below this level
    /// are not delivered. Defaults to Info.
    pub fn set_min_level(&self, level: LogLevel) {
        unsafe {
            btck_logging_set_min_level(self.inner, level.into());
        }
    }
}

/// Logging categories for Bitcoin Kernel messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LogCategory {
    All = btck_LogCategory_ALL,
    Bench = btck_LogCategory_BENCH,
    BlockStorage = btck_LogCategory_BLOCKSTORAGE,
    CoinDb = btck_LogCategory_COINDB,
    LevelDb = btck_LogCategory_LEVELDB,
    Mempool = btck_LogCategory_MEMPOOL,
    Prune = btck_LogCategory_PRUNE,
    Rand = btck_LogCategory_RAND,
    Reindex = btck_LogCategory_REINDEX,
    Validation = btck_LogCategory_VALIDATION,
    Kernel = btck_LogCategory_KERNEL,
    TxPackages = btck_LogCategory_TXPACKAGES,
    Lock = btck_LogCategory_LOCK,
    Unknown = btck_LogCategory_UNKNOWN,
}

impl From<LogCategory> for btck_LogCategory {
    fn from(category: LogCategory) -> Self {
        category as btck_LogCategory
    }
}

#[allow(non_upper_case_globals)]
impl From<btck_LogCategory> for LogCategory {
    fn from(value: btck_LogCategory) -> Self {
        match value {
            btck_LogCategory_ALL => LogCategory::All,
            btck_LogCategory_BENCH => LogCategory::Bench,
            btck_LogCategory_BLOCKSTORAGE => LogCategory::BlockStorage,
            btck_LogCategory_COINDB => LogCategory::CoinDb,
            btck_LogCategory_LEVELDB => LogCategory::LevelDb,
            btck_LogCategory_MEMPOOL => LogCategory::Mempool,
            btck_LogCategory_PRUNE => LogCategory::Prune,
            btck_LogCategory_RAND => LogCategory::Rand,
            btck_LogCategory_REINDEX => LogCategory::Reindex,
            btck_LogCategory_VALIDATION => LogCategory::Validation,
            btck_LogCategory_KERNEL => LogCategory::Kernel,
            btck_LogCategory_TXPACKAGES => LogCategory::TxPackages,
            btck_LogCategory_LOCK => LogCategory::Lock,
            btck_LogCategory_UNKNOWN => LogCategory::Unknown,
            _ => panic!("Unknown log category: {}", value),
        }
    }
}

/// Logging levels for controlling message verbosity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LogLevel {
    Trace = btck_LogLevel_TRACE,
    Debug = btck_LogLevel_DEBUG,
    Info = btck_LogLevel_INFO,
    Warning = btck_LogLevel_WARNING,
    Error = btck_LogLevel_ERROR,
}

impl From<LogLevel> for btck_LogLevel {
    fn from(level: LogLevel) -> Self {
        level as btck_LogLevel
    }
}

#[allow(non_upper_case_globals)]
impl From<btck_LogLevel> for LogLevel {
    fn from(value: btck_LogLevel) -> Self {
        match value {
            btck_LogLevel_TRACE => LogLevel::Trace,
            btck_LogLevel_DEBUG => LogLevel::Debug,
            btck_LogLevel_INFO => LogLevel::Info,
            btck_LogLevel_WARNING => LogLevel::Warning,
            btck_LogLevel_ERROR => LogLevel::Error,
            _ => panic!("Unknown log level: {}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_category_conversions() {
        for cat in [
            LogCategory::All,
            LogCategory::Bench,
            LogCategory::BlockStorage,
            LogCategory::CoinDb,
            LogCategory::LevelDb,
            LogCategory::Mempool,
            LogCategory::Prune,
            LogCategory::Rand,
            LogCategory::Reindex,
            LogCategory::Validation,
            LogCategory::Kernel,
            LogCategory::TxPackages,
            LogCategory::Lock,
            LogCategory::Unknown,
        ] {
            let raw: btck_LogCategory = cat.into();
            let back: LogCategory = raw.into();
            assert_eq!(cat, back);
        }
    }

    #[test]
    fn test_log_level_conversions() {
        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warning,
            LogLevel::Error,
        ] {
            let raw: btck_LogLevel = level.into();
            let back: LogLevel = raw.into();
            assert_eq!(level, back);
        }
    }

    struct TestLog {
        messages: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl Log for TestLog {
        fn log(&self, message: &str) {
            self.messages.lock().unwrap().push(message.to_string());
        }
    }

    #[test]
    fn test_logger_creation() {
        let messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let test_log = TestLog {
            messages: messages.clone(),
        };
        let logger = Logger::new(test_log);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_logger_set_min_level() {
        let messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let test_log = TestLog {
            messages: messages.clone(),
        };
        let logger = Logger::new(test_log).unwrap();
        logger.set_min_level(LogLevel::Debug);
        logger.set_min_level(LogLevel::Info);
    }
}
