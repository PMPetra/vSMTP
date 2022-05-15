//! vSMTP testing utilities

#![doc(html_no_source)]
#![deny(missing_docs)]
//
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
//
#![allow(clippy::doc_markdown)]

/// Config shortcut
pub mod config;

///
pub mod receiver;

///
pub mod get_tls_file;

#[cfg(test)]
mod tests;

const TESTS_ROOT_WORKSPACE: &str = "./tests_root_workspace";

/// holds metadata to run integration tests.
/// tests must be under `src/vsmtp/`
pub struct TestContext {
    /// the path to the workspace of the test. fs resources can be written
    /// in this path.
    workspace: std::path::PathBuf,
}

static INIT_LOGS: std::sync::Once = std::sync::Once::new();

impl TestContext {
    /// create a new test context.
    /// this sets the current directory to the workspace root.
    /// it also initializes logs if needed and creates a new workspace
    /// to work with.
    ///
    /// # Errors
    /// * failed to create a test workspace.
    ///
    /// # Panics
    /// * Could not set the current dir to the root of the workspace.
    pub fn new() -> std::io::Result<Self> {
        INIT_LOGS.call_once(|| {
            std::env::set_current_dir("../../../").unwrap();
            initialize_tests_logs();
        });

        Ok(Self {
            workspace: get_test_workspace()?,
        })
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.workspace).unwrap();
    }
}

/// create a workspace to make integration tests.
/// the path returned is the path to the created workspace.
/// the generated folder will be removed on test teardown.
///
/// # Errors
///
/// * failed to create a test workspace.
fn get_test_workspace() -> std::io::Result<std::path::PathBuf> {
    let path = std::path::PathBuf::from_iter([
        initialize_tests_root_workspace(),
        &std::iter::repeat_with(fastrand::alphanumeric)
            .take(20)
            .collect::<String>(),
    ]);

    std::fs::create_dir(&path)?;

    Ok(path)
}

/// initializes logs using the console appender from log4rs.
fn initialize_tests_logs() {
    use log4rs::{append, config, encode, Config};
    use vsmtp_common::re::log::LevelFilter;

    let config = Config::builder()
        .appender(
            config::Appender::builder().build(
                "stdout",
                Box::new(
                    append::console::ConsoleAppender::builder()
                        .encoder(Box::new(encode::pattern::PatternEncoder::new(
                            "{d(%Y-%m-%d %H:%M:%S%.f)} {h({l:<5} [{I}])} {t:<30} $ {m}{n}",
                        )))
                        .build(),
                ),
            ),
        )
        .build(
            config::Root::builder()
                .appender("stdout")
                .build(LevelFilter::Warn),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}

/// create the folder that will hold all tests workspaces if needed.
fn initialize_tests_root_workspace() -> &'static str {
    if !std::path::Path::new(TESTS_ROOT_WORKSPACE).exists() {
        std::fs::create_dir_all(TESTS_ROOT_WORKSPACE).unwrap();
    }

    TESTS_ROOT_WORKSPACE
}
