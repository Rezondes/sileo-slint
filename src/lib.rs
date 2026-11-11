//! A physics-based toast component for [Slint](https://slint.dev).
//! Port of [Sileo](https://github.com/hiaaryan/sileo).
//!
//! The component itself is written in Slint and ships in the `ui` folder of this crate.
//! This crate does two things: [`library_paths`] tells the Slint compiler where that
//! folder is, and [`init!`] backs the toast list with Rust.
//!
//! # Setup
//!
//! Add the crate as a dependency and as a build dependency, next to `slint` (1.18 or
//! newer) and `slint-build`.
//!
//! `build.rs`:
//!
//! ```ignore
//! fn main() {
//!     let config = slint_build::CompilerConfiguration::new()
//!         .with_library_paths(sileo_slint::library_paths());
//!     slint_build::compile_with_config("ui/app.slint", config).unwrap();
//! }
//! ```
//!
//! `ui/app.slint`:
//!
//! ```slint,ignore
//! import { Sileo, SileoItem, SileoKind, SileoPosition, SileoToast, Toaster } from "@sileo";
//! // Rust needs these types, so export them from your main file.
//! export { Sileo, SileoItem, SileoKind, SileoPosition, SileoToast }
//!
//! export component App inherits Window {
//!     // your content, then the Toaster on top
//!     Toaster { }
//! }
//! ```
//!
//! `src/main.rs`:
//!
//! ```ignore
//! slint::include_modules!();
//!
//! fn main() -> Result<(), slint::PlatformError> {
//!     let ui = App::new()?;
//!     sileo_slint::init!(ui);
//!
//!     ui.global::<Sileo>().invoke_success("Saved".into(), "Your changes are stored.".into());
//!     ui.global::<Sileo>().invoke_show(SileoToast {
//!         title: "Update available".into(),
//!         kind: SileoKind::Action,
//!         button: "Install".into(),
//!         ..Default::default()
//!     });
//!
//!     ui.run()
//! }
//! ```
//!
//! The same calls work from Slint code: `Sileo.success("Saved", "")`.

use std::collections::HashMap;
use std::path::PathBuf;

/// The library path that makes `import { Toaster } from "@sileo";` work.
///
/// Pass it to `slint_build::CompilerConfiguration::with_library_paths` in your `build.rs`.
pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([(
        "sileo".to_string(),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/sileo.slint")),
    )])
}

/// Connects the `Sileo` global of a component to its toast list.
///
/// Call it once per window, right after creating the component. Without it the calls on
/// `Sileo` do nothing.
///
/// The Slint-generated types `Sileo`, `SileoItem` and `SileoPosition` must be in scope,
/// so re-export them from your main `.slint` file (see the crate documentation).
///
/// The macro is the whole runtime: the generated types live in your crate, so the code
/// that touches them has to be expanded there.
#[macro_export]
macro_rules! init {
    ($ui:expr) => {{
        use ::slint::{ComponentHandle as _, Model as _};

        let ui = &$ui;
        let model = ::std::rc::Rc::new(::slint::VecModel::<SileoItem>::default());
        let instance = ::std::rc::Rc::new(::std::cell::Cell::new(0i32));
        let global = ui.global::<Sileo>();
        global.set_items(::slint::ModelRc::from(model.clone()));
        global.set_items_reversed(::slint::ModelRc::new(::slint::ReverseModel::new(
            ::slint::ModelRc::from(model.clone()),
        )));

        // Keeps the ids the Toaster reads in step with the list.
        let sync = {
            let (weak, model) = (ui.as_weak(), model.clone());
            move || {
                let Some(ui) = weak.upgrade() else { return };
                let global = ui.global::<Sileo>();
                let latest = model.iter().filter(|item| !item.exiting).last();
                global.set_latest_id(latest.map(|item| item.toast.id).unwrap_or_default());
                let hovered = global.get_hovered_id();
                let alive = model
                    .iter()
                    .any(|item| !item.exiting && item.toast.id == hovered);
                if !hovered.is_empty() && !alive {
                    global.set_hovered_id(Default::default());
                }
            }
        };

        global.on_show({
            let (model, instance, sync) = (model.clone(), instance.clone(), sync.clone());
            move |mut toast| {
                if toast.id.is_empty() {
                    toast.id = "sileo-default".into();
                }
                let id = toast.id.clone();
                instance.set(instance.get() + 1);
                let live = model
                    .iter()
                    .position(|item| !item.exiting && item.toast.id == id);
                if let Some(row) = live {
                    // Same id: the visible toast morphs and stays where it is.
                    if toast.position == SileoPosition::Auto {
                        toast.position = model.row_data(row).unwrap().toast.position;
                    }
                    model.set_row_data(
                        row,
                        SileoItem {
                            toast,
                            instance: instance.get(),
                            exiting: false,
                        },
                    );
                } else {
                    // A toast with this id that is still on its way out makes room at once.
                    if let Some(row) = model.iter().position(|item| item.toast.id == id) {
                        model.remove(row);
                    }
                    model.push(SileoItem {
                        toast,
                        instance: instance.get(),
                        exiting: false,
                    });
                }
                sync();
                id
            }
        });

        global.on_dismiss({
            let (model, sync) = (model.clone(), sync.clone());
            move |id| {
                let live = model
                    .iter()
                    .position(|item| !item.exiting && item.toast.id == id);
                let Some(row) = live else { return };
                let mut item = model.row_data(row).unwrap();
                item.exiting = true;
                let instance = item.instance;
                model.set_row_data(row, item);
                sync();

                // Remove it once the exit animation is over. The instance pins the exact
                // toast, so a new toast with the same id is left alone.
                let (model, sync) = (model.clone(), sync.clone());
                ::slint::Timer::single_shot(::std::time::Duration::from_millis(600), move || {
                    let gone = model
                        .iter()
                        .position(|item| item.exiting && item.instance == instance);
                    if let Some(row) = gone {
                        model.remove(row);
                        sync();
                    }
                });
            }
        });

        global.on_clear({
            let (model, sync) = (model.clone(), sync.clone());
            move || {
                model.set_vec(::std::vec::Vec::new());
                sync();
            }
        });
    }};
}
