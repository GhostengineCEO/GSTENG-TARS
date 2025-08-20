pub mod cli;
pub mod server;
pub mod workspace;
pub mod extensions;

pub use cli::VSCodeCLI;
pub use server::VSCodeServer;
pub use workspace::{Workspace, WorkspaceManager};
pub use extensions::{Extension, ExtensionManager};
