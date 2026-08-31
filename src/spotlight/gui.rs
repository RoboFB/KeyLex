//! The small popup window `keylex --spotlight-daemon` and
//! `keylex --spotlight-popup` both raise -- the same search and dispatch as
//! `ui.rs`'s terminal launcher, but drawn with `egui`/`eframe` as a real OS
//! window instead of into a terminal, since neither command has a
//! controlling terminal to draw into.
//!
//! The window is created once and never destroyed again for as long as the
//! process runs: `eframe::run_native` owns the whole native event loop, so
//! "reopen the popup" (the daemon case) means toggling
//! `ViewportCommand::Visible` on the one window that already exists, not
//! recreating winit's event loop on every hotkey press (not something every
//! platform reliably supports).

use std::io;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use eframe::egui;

use super::Index;
use crate::dispatch::Router;
use crate::focus;

const MAX_VISIBLE_MATCHES: usize = 10;

/// How the popup came to exist, and so what "done with it" should mean.
pub enum Lifecycle {
    /// `--spotlight-daemon`: starts hidden, shown and re-hidden repeatedly
    /// by signals on the channel (`hotkey::listen`'s trigger), and never
    /// exits on its own -- only killing the daemon ends it.
    Daemon(Receiver<()>),
    /// `--spotlight-popup`: a single invocation with nothing external
    /// triggering it (meant to be *the* command an outside keybinding
    /// mechanism -- a GNOME custom shortcut, a WM keybind -- runs), so it
    /// starts visible immediately and the process exits as soon as the
    /// popup is dismissed one way or another.
    OneShot,
}

struct App {
    index: Index<'static>,
    router: Router<'static>,
    show_rx: Option<Receiver<()>>,
    exit_when_hidden: bool,
    visible: bool,
    started: bool,
    start_visible: bool,
    focus_search_box: bool,
    query: String,
    selected: usize,
    last_dispatch: Option<String>,
}

impl App {
    fn show(&mut self, ctx: &egui::Context) {
        self.visible = true;
        self.focus_search_box = true;
        self.query.clear();
        self.selected = 0;
        self.last_dispatch = None;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    fn hide(&mut self, ctx: &egui::Context) {
        self.visible = false;
        if self.exit_when_hidden {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }
}

impl eframe::App for App {
    /// Runs whether or not the window is currently visible (see
    /// `eframe::App::logic`'s own doc comment) -- the only place a hidden
    /// window can still notice the hotkey fired and ask to be shown again.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // `NativeOptions::viewport.with_visible(false)` is only a creation
        // hint -- at least one backend still shows the window briefly while
        // setting up its graphics context, so startup needs an explicit
        // show/hide once egui is actually running, not just a field that
        // starts out one way or the other.
        if !std::mem::replace(&mut self.started, true) {
            if self.start_visible {
                self.show(ctx);
            } else {
                self.hide(ctx);
            }
        }

        while self
            .show_rx
            .as_ref()
            .is_some_and(|show_rx| show_rx.try_recv().is_ok())
        {
            self.show(ctx);
        }
        // The titlebar close button hides the daemon's popup rather than
        // exiting the whole daemon; the one-shot popup has nothing to stay
        // alive for, so its close button is left to behave normally.
        if !self.exit_when_hidden && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.hide(ctx);
        }
        // The only place `show_rx` gets polled, so this has to keep firing
        // on its own even while hidden and nothing else requests a repaint.
        // Harmless (if wasted) for the one-shot popup, which has no channel
        // to poll but exits as soon as it's dismissed anyway.
        ctx.request_repaint_after(Duration::from_millis(50));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide(&ctx);
            return;
        }

        let matches = self.index.search(&self.query);
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.selected = self.selected.saturating_add(1);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.selected = self.selected.saturating_sub(1);
        }
        self.selected = self.selected.min(matches.len().saturating_sub(1));

        let mut run_selected = ctx.input(|i| i.key_pressed(egui::Key::Enter));

        let response = ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text("Search commands...")
                .desired_width(f32::INFINITY),
        );
        if std::mem::take(&mut self.focus_search_box) {
            response.request_focus();
        }

        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if matches.is_empty() {
                ui.label("(no matches)");
            }
            for (i, m) in matches.iter().take(MAX_VISIBLE_MATCHES).enumerate() {
                let key_hint = m
                    .entry
                    .key_hint
                    .as_deref()
                    .map_or_else(String::new, |k| format!(" ({k})"));
                let label = format!("{}{key_hint}  [{}]", m.entry.title, m.entry.source);
                if ui.selectable_label(i == self.selected, label).clicked() {
                    self.selected = i;
                    run_selected = true;
                }
            }
        });

        if let Some(message) = &self.last_dispatch {
            ui.separator();
            ui.label(message);
        }

        if run_selected {
            if let Some(entry) = matches.get(self.selected).map(|m| m.entry.clone()) {
                let outcome =
                    entry.dispatch(focus::focused_process_name().as_deref(), &self.router);
                self.index.record_use(&entry.action_id);
                self.last_dispatch = Some(format!("{} -> {outcome}", entry.action_id));
            }
            self.hide(&ctx);
        }
    }
}

/// Runs the popup according to `lifecycle` (see its own doc comment for
/// what "daemon" vs "one-shot" mean here). `index` and `router` are
/// `'static` because `eframe::App` itself requires it -- leaking the
/// `Registry` they both borrow from is how the caller gets one cheaply,
/// since it needs to live for the process's whole run anyway either way
/// (see `cli.rs`'s `spotlight_daemon` and `spotlight_popup`).
pub fn run(index: Index<'static>, router: Router<'static>, lifecycle: Lifecycle) -> io::Result<()> {
    let (show_rx, start_visible, exit_when_hidden) = match lifecycle {
        Lifecycle::Daemon(rx) => (Some(rx), false, false),
        Lifecycle::OneShot => (None, true, true),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Keylex spotlight")
            .with_inner_size([480.0, 360.0])
            .with_always_on_top()
            .with_visible(false),
        ..Default::default()
    };
    let app = App {
        index,
        router,
        show_rx,
        exit_when_hidden,
        visible: false,
        started: false,
        start_visible,
        focus_search_box: false,
        query: String::new(),
        selected: 0,
        last_dispatch: None,
    };

    eframe::run_native(
        "Keylex spotlight",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| io::Error::other(format!("spotlight popup window failed: {e}")))
}
