//! PROTOTYPE core: every part but command.
//! No PyO3: execution reaches Python through the Interpreter trait.
pub mod assertion;
pub mod dispatch;
pub mod execution;
pub mod reporting;
pub mod selection;
