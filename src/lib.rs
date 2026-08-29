//! Keylex intercepts keystrokes as deep in the OS as it can, resolves the
//! bound ones into abstract actions (`close.tab`, `save`), works out which
//! application is focused, and dispatches each action through that app's
//! own API -- synthesizing a keycode only as a fallback.
//!
//! The pipeline reads left to right: [`config`] loads the action and target
//! vocabulary, [`capture`] turns keystrokes into action ids, [`focus`] says
//! which app should receive them, [`dispatch`] picks the route, and
//! [`adapters`] carries the command to the target. [`spotlight`] is a
//! second front end onto the same dispatch path, and [`cli`] wires it all
//! together.

pub mod adapters;
pub mod capture;
pub mod cli;
pub mod config;
pub mod dispatch;
pub mod focus;
pub mod spotlight;
