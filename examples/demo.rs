//! Playground for every kind, position and theme.
//!
//! `cargo run --example demo` opens it.
//! `cargo run --example demo -- <dir>` walks through the states, writes frames as PNG
//! into `<dir>` and quits. It also clicks, hovers and swipes with synthetic pointer
//! events. That is the visual check after a change to the toast.

use slint::Model as _;
use slint::platform::{PointerEventButton, WindowEvent};
use std::time::Duration;

slint::slint! {
    #[library_path(sileo) = "ui/sileo.slint"]
    import { Button, HorizontalBox, Palette, Switch, VerticalBox } from "std-widgets.slint";
    import { Sileo, SileoItem, SileoKind, SileoPosition, SileoToast, Toaster } from "@sileo";
    export { Sileo, SileoItem, SileoKind, SileoPosition, SileoToast }

    component PositionButton inherits Button {
        in property <SileoPosition> value;
        in-out property <SileoPosition> current;
        primary: self.current == self.value;
        clicked => { self.current = self.value; }
    }

    export component Demo inherits Window {
        title: "sileo-slint";
        preferred-width: 820px;
        preferred-height: 560px;

        in-out property <SileoPosition> position: SileoPosition.top-right;
        property <bool> uploading;

        public function set-dark-mode(dark: bool) {
            Palette.color-scheme = dark ? ColorScheme.dark : ColorScheme.light;
        }

        // The loading toast turns into a success toast: same id, new content.
        Timer {
            interval: 2s;
            running: root.uploading;
            triggered => {
                root.uploading = false;
                Sileo.success("Upload complete", "report.pdf is now shared with your team.");
            }
        }

        VerticalBox {
            alignment: center;
            spacing: 20px;

            Text {
                text: "sileo-slint";
                font-size: 28px;
                font-weight: 600;
                horizontal-alignment: center;
            }

            HorizontalBox {
                alignment: center;
                Button {
                    text: "Success";
                    clicked => { Sileo.success("Changes saved", "Your changes are stored and synced."); }
                }
                Button {
                    text: "Error";
                    clicked => { Sileo.error("Upload failed", "The server did not answer. Check your connection and try again."); }
                }
                Button {
                    text: "Warning";
                    clicked => { Sileo.warning("Storage almost full", "You have used 95 % of your space."); }
                }
                Button {
                    text: "Info";
                    clicked => { Sileo.info("New version", ""); }
                }
                Button {
                    text: "Action";
                    clicked => {
                        Sileo.show({
                            title: "Update available",
                            description: "Version 2.4 is ready to install.",
                            kind: SileoKind.action,
                            button: "Install now",
                        });
                    }
                }
                Button {
                    text: "Loading";
                    clicked => {
                        Sileo.loading("Uploading");
                        root.uploading = true;
                    }
                }
            }

            HorizontalBox {
                alignment: center;
                Button {
                    text: "Second toast";
                    clicked => { Sileo.show({ id: "second", title: "Another one", description: "Toasts with their own id stack.", kind: SileoKind.info }); }
                }
                Button {
                    text: "Dismiss";
                    clicked => { Sileo.dismiss("sileo-default"); }
                }
                Button {
                    text: "Clear";
                    clicked => { Sileo.clear(); }
                }
                Switch {
                    text: "Dark mode";
                    checked: Palette.color-scheme == ColorScheme.dark;
                    toggled => { root.set-dark-mode(self.checked); }
                }
            }

            GridLayout {
                spacing: 8px;
                Row {
                    Rectangle { }
                    PositionButton { text: "Top left"; value: SileoPosition.top-left; current <=> root.position; }
                    PositionButton { text: "Top center"; value: SileoPosition.top-center; current <=> root.position; }
                    PositionButton { text: "Top right"; value: SileoPosition.top-right; current <=> root.position; }
                    Rectangle { }
                }
                Row {
                    Rectangle { }
                    PositionButton { text: "Bottom left"; value: SileoPosition.bottom-left; current <=> root.position; }
                    PositionButton { text: "Bottom center"; value: SileoPosition.bottom-center; current <=> root.position; }
                    PositionButton { text: "Bottom right"; value: SileoPosition.bottom-right; current <=> root.position; }
                    Rectangle { }
                }
            }
        }

        // The theme follows the color scheme: dark toasts on a light app and the reverse.
        Toaster {
            position: root.position;
        }
    }
}

fn snap(ui: &Demo, dir: &str, name: &str) {
    let b = ui.window().take_snapshot().expect("snapshot");
    image::save_buffer(
        format!("{dir}/{name}.png"),
        b.as_bytes(),
        b.width(),
        b.height(),
        image::ColorType::Rgba8,
    )
    .expect("write png");
}

enum Pointer {
    Move,
    Press,
    Release,
}

/// Sends a synthetic pointer event, in logical px of the 820 x 560 demo window.
fn pointer(ui: &Demo, kind: Pointer, x: f32, y: f32) {
    let position = slint::LogicalPosition::new(x, y);
    let button = PointerEventButton::Left;
    ui.window().dispatch_event(match kind {
        Pointer::Move => WindowEvent::PointerMoved { position },
        Pointer::Press => WindowEvent::PointerPressed { position, button },
        Pointer::Release => WindowEvent::PointerReleased { position, button },
    });
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = Demo::new()?;
    sileo_slint::init!(ui);

    ui.global::<Sileo>().on_button_clicked({
        let weak = ui.as_weak();
        move |_id| {
            let ui = weak.unwrap();
            ui.global::<Sileo>()
                .invoke_success("Installed".into(), "".into());
        }
    });

    // Frame dump, one tick = 50 ms.
    let timer = slint::Timer::default();
    if let Some(dir) = std::env::args().nth(1) {
        let weak = ui.as_weak();
        let mut n = 0;
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(50),
            move || {
                let ui = weak.unwrap();
                let sileo = ui.global::<Sileo>();
                n += 1;
                match n {
                    1 => ui.invoke_set_dark_mode(false),
                    6 => {
                        sileo.invoke_success(
                            "Changes saved".into(),
                            "Your changes are stored and synced.".into(),
                        );
                    }
                    8 => snap(&ui, &dir, "0-enter-100ms"),
                    11 => snap(&ui, &dir, "1-expanding-250ms"),
                    30 => {
                        snap(&ui, &dir, "2-expanded");
                        sileo.invoke_error(
                            "Upload failed".into(),
                            "The server did not answer. Check your connection and try again."
                                .into(),
                        );
                    }
                    33 => snap(&ui, &dir, "3-morph-150ms"),
                    37 => snap(&ui, &dir, "4-morph-350ms"),
                    60 => {
                        snap(&ui, &dir, "5-error-expanded");
                        ui.set_position(SileoPosition::BottomCenter);
                        sileo.invoke_show(SileoToast {
                            title: "Update available".into(),
                            description: "Version 2.4 is ready to install.".into(),
                            kind: SileoKind::Action,
                            button: "Install now".into(),
                            ..Default::default()
                        });
                    }
                    63 => snap(&ui, &dir, "6-bottom-150ms"),
                    90 => {
                        snap(&ui, &dir, "7-bottom-action");
                        // Click the button of the expanded toast.
                        pointer(&ui, Pointer::Move, 410.0, 478.0);
                        pointer(&ui, Pointer::Press, 410.0, 478.0);
                        pointer(&ui, Pointer::Release, 410.0, 478.0);
                    }
                    100 => {
                        snap(&ui, &dir, "8-button-clicked");
                        ui.window().dispatch_event(WindowEvent::PointerExited);
                        sileo.invoke_show(SileoToast {
                            title: "Hover me".into(),
                            description: "Without autopilot the body only opens on hover.".into(),
                            kind: SileoKind::Warning,
                            no_autopilot: true,
                            ..Default::default()
                        });
                    }
                    112 => {
                        snap(&ui, &dir, "9-no-autopilot");
                        pointer(&ui, Pointer::Move, 410.0, 530.0);
                    }
                    124 => {
                        snap(&ui, &dir, "10-hovered");
                        // Swipe up by 40 px: more than the 30 px that dismiss.
                        pointer(&ui, Pointer::Press, 410.0, 530.0);
                        pointer(&ui, Pointer::Move, 410.0, 490.0);
                    }
                    125 => {
                        snap(&ui, &dir, "11-dragging");
                        pointer(&ui, Pointer::Release, 410.0, 490.0);
                        ui.window().dispatch_event(WindowEvent::PointerExited);
                    }
                    126 => assert!(
                        sileo
                            .get_items()
                            .row_data(0)
                            .is_some_and(|item| item.exiting),
                        "a swipe dismisses the toast"
                    ),
                    140 => {
                        assert_eq!(sileo.get_items().row_count(), 0, "the swiped toast is gone");
                        ui.set_position(SileoPosition::TopLeft);
                        ui.invoke_set_dark_mode(true);
                        sileo.invoke_loading("Uploading".into());
                        sileo.invoke_show(SileoToast {
                            id: "second".into(),
                            title: "Another one".into(),
                            description: "Toasts with their own id stack.".into(),
                            kind: SileoKind::Info,
                            ..Default::default()
                        });
                    }
                    170 => {
                        snap(&ui, &dir, "12-stack-dark-mode");
                        sileo.invoke_dismiss("second".into());
                    }
                    171 => snap(&ui, &dir, "13-exit-50ms"),
                    190 => slint::quit_event_loop().expect("quit"),
                    _ => {}
                }
            },
        );
    }

    ui.run()
}
