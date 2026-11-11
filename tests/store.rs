//! The toast list behind `Sileo`, driven through the real Slint-generated types.

use slint::Model as _;
use std::time::Duration;

slint::slint! {
    #[library_path(sileo) = "ui/sileo.slint"]
    import { Toaster } from "@sileo";
    export { Sileo, SileoItem, SileoKind, SileoPosition, SileoToast } from "@sileo";

    export component App inherits Window {
        Toaster { }
    }
}

fn ids(ui: &App) -> Vec<String> {
    ui.global::<Sileo>()
        .get_items()
        .iter()
        .map(|item| item.toast.id.to_string())
        .collect()
}

fn toast(id: &str, title: &str) -> SileoToast {
    SileoToast {
        id: id.into(),
        title: title.into(),
        ..Default::default()
    }
}

// One test function: the Slint platform belongs to a single thread.
#[test]
fn store() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = App::new().unwrap();
    sileo_slint::init!(ui);
    let sileo = ui.global::<Sileo>();

    // No id: one default toast that morphs.
    assert_eq!(
        sileo.invoke_success("Saved".into(), "".into()),
        "sileo-default"
    );
    sileo.invoke_show(SileoToast {
        title: "Left".into(),
        position: SileoPosition::BottomLeft,
        ..Default::default()
    });
    let first = sileo.get_items().row_data(0).unwrap();
    sileo.invoke_error("Failed".into(), "".into());
    let second = sileo.get_items().row_data(0).unwrap();
    assert_eq!(ids(&ui), ["sileo-default"]);
    assert_eq!(second.toast.kind, SileoKind::Error);
    assert_ne!(
        second.instance, first.instance,
        "showing again is a new instance"
    );
    assert_eq!(
        second.toast.position,
        SileoPosition::BottomLeft,
        "a morphing toast stays in place"
    );

    // Own ids stack, the newest one is the active one.
    sileo.invoke_show(toast("a", "A"));
    sileo.invoke_show(toast("b", "B"));
    assert_eq!(ids(&ui), ["sileo-default", "a", "b"]);
    assert_eq!(sileo.get_latest_id(), "b");
    let reversed = sileo.get_items_reversed();
    assert_eq!(reversed.row_data(0).unwrap().toast.id, "b");

    // Dismiss: exits first, is removed after the exit animation.
    sileo.set_hovered_id("b".into());
    sileo.invoke_dismiss("b".into());
    assert!(sileo.get_items().row_data(2).unwrap().exiting);
    assert_eq!(sileo.get_latest_id(), "a");
    assert_eq!(
        sileo.get_hovered_id(),
        "",
        "a hover on a leaving toast must not pause the rest"
    );
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(700));
    assert_eq!(ids(&ui), ["sileo-default", "a"]);

    // Shown again while leaving: the new toast survives the pending removal.
    sileo.invoke_dismiss("a".into());
    sileo.invoke_show(toast("a", "A again"));
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(700));
    assert_eq!(ids(&ui), ["sileo-default", "a"]);
    assert!(!sileo.get_items().row_data(1).unwrap().exiting);

    // Dismissing an unknown id does nothing.
    sileo.invoke_dismiss("nope".into());
    assert_eq!(ids(&ui).len(), 2);

    sileo.invoke_clear();
    assert!(ids(&ui).is_empty());
    assert_eq!(sileo.get_latest_id(), "");
}
