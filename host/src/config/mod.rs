//! Config Store — profile/page/button (source of truth di Host).

pub mod store;

// Re-export API publik (sebagian dipakai editor/fase lanjut).
#[allow(unused_imports)]
pub use store::{
    clear_button_from_pages, Action, Button, Config, ConfigStore, GridSize, Page, PageType,
    Profile,
};
