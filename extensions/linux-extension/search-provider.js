// GNOME Shell search provider ("org.gnome.Shell.SearchProvider2") for
// Keylex's spotlight action search (../../src/spotlight.rs), so the same
// fuzzy-ranked action catalog the `keylex --spotlight` terminal launcher
// shows is also reachable from GNOME Shell's own Activities search --
// "talking nicely to the OS/window manager" instead of Keylex inventing its
// own always-on-top launcher window.
//
// This deliberately does NOT reimplement fuzzy matching in JavaScript: every
// query and every activation is a plain subprocess call into the `keylex`
// binary itself (`--spotlight-query` / `--spotlight-run`, see
// ../../src/main.rs), so ranking is always the one, pure-Rust
// (nucleo-matcher) implementation the Rust daemon uses everywhere else --
// this file is glue, not a second search engine.
//
// UNTESTED outside a real GNOME Shell session: this sandbox has no GNOME
// Shell / session D-Bus bus to register against (same caveat this repo
// already carries for src/capture/windows.rs -- see CLAUDE.md's "Known
// gaps"). Written carefully against the documented SearchProvider2 D-Bus
// interface, but never actually exercised end-to-end in a real desktop
// session.
//
// Requires the `dbus-next` package (see package.json in this folder --
// `npm install` here before running). Registration also needs the
// accompanying .desktop and search-provider .ini files -- see README.md.
const path = require("path");
const { execFile } = require("child_process");
const dbus = require("dbus-next");
const Variant = dbus.Variant;
const { Interface } = dbus.interface;

const BUS_NAME = "com.keylex.SearchProvider";
const OBJECT_PATH = "/com/keylex/SearchProvider";
const MAX_RESULTS = 9;

// The release build is the default, since a search provider that GNOME
// Shell keeps alive in the background shouldn't be running an unoptimized
// debug binary. Override with KEYLEX_BIN for a dev build, and
// KEYLEX_CONFIG_DIR if the daemon was started with a non-default
// `--config-dir` (see ../../src/main.rs).
const KEYLEX_BIN = process.env.KEYLEX_BIN || path.join(__dirname, "..", "..", "target", "release", "keylex");
const CONFIG_DIR_ARGS = process.env.KEYLEX_CONFIG_DIR ? ["--config-dir", process.env.KEYLEX_CONFIG_DIR] : [];

function runKeylex(args) {
  return new Promise((resolve, reject) => {
    execFile(KEYLEX_BIN, [...CONFIG_DIR_ARGS, ...args], (err, stdout, stderr) => {
      if (err) {
        reject(new Error(stderr || err.message));
        return;
      }
      resolve(stdout);
    });
  });
}

// Mirrors src/spotlight.rs's SpotlightMatch/SpotlightEntry JSON shape
// (serde `Serialize` derive) -- see `keylex --spotlight-query`'s output.
async function spotlightSearch(query) {
  const stdout = await runKeylex(["--spotlight-query", query]);
  const matches = JSON.parse(stdout);
  return matches.slice(0, MAX_RESULTS).map((m) => ({
    id: m.entry.action_id,
    name: m.entry.title,
    description: m.entry.key_hint ? `${m.entry.source} · ${m.entry.key_hint}` : m.entry.source,
  }));
}

async function spotlightActivate(actionId) {
  await runKeylex(["--spotlight-run", actionId]);
}

class KeylexSearchProvider extends Interface {
  constructor() {
    super("org.gnome.Shell.SearchProvider2");
    // Populated by the most recent search, so GetResultMetas (called right
    // after GetInitialResultSet/GetSubsearchResultSet by the shell) can
    // answer without re-running the query.
    this._lastResults = new Map();
  }

  async _search(terms) {
    const query = terms.join(" ");
    let results = [];
    try {
      results = await spotlightSearch(query);
    } catch (err) {
      console.error("keylex search-provider: search failed:", err.message);
      return [];
    }
    this._lastResults = new Map(results.map((r) => [r.id, r]));
    return results.map((r) => r.id);
  }

  async GetInitialResultSet(terms) {
    return this._search(terms);
  }

  async GetSubsearchResultSet(_previousResults, terms) {
    return this._search(terms);
  }

  async GetResultMetas(identifiers) {
    return identifiers.map((id) => {
      const result = this._lastResults.get(id);
      return {
        id: new Variant("s", id),
        name: new Variant("s", result ? result.name : id),
        description: new Variant("s", result ? result.description : ""),
      };
    });
  }

  async ActivateResult(identifier, _terms, _timestamp) {
    try {
      await spotlightActivate(identifier);
    } catch (err) {
      console.error(`keylex search-provider: activating ${identifier} failed:`, err.message);
    }
  }

  async LaunchSearch(terms, _timestamp) {
    const ids = await this._search(terms);
    if (ids.length > 0) {
      await this.ActivateResult(ids[0], terms, _timestamp);
    }
  }
}

KeylexSearchProvider.configureMembers({
  methods: {
    GetInitialResultSet: { inSignature: "as", outSignature: "as" },
    GetSubsearchResultSet: { inSignature: "asas", outSignature: "as" },
    GetResultMetas: { inSignature: "as", outSignature: "aa{sv}" },
    ActivateResult: { inSignature: "sasu", outSignature: "" },
    LaunchSearch: { inSignature: "asu", outSignature: "" },
  },
});

async function main() {
  const bus = dbus.sessionBus();
  await bus.requestName(BUS_NAME);
  bus.export(OBJECT_PATH, new KeylexSearchProvider());
  console.log(`keylex GNOME search provider registered as ${BUS_NAME} at ${OBJECT_PATH}`);
}

main().catch((err) => {
  console.error("keylex search-provider: failed to start:", err);
  process.exit(1);
});
