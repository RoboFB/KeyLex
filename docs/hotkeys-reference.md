# Hotkey / command reference

A working reference of the default commands and keyboard shortcuts each
target application already ships with, collected in one place so new
Keylex actions (`config/vocabulary.toml` / `config/actions.toml`) and
target capabilities (`extensions/*/capabilities.toml`) can be planned
against what each app already does, instead of guessing.

Not every command has a default hotkey (many are menu/command-palette
only), and not every row needs a comment — both columns are left blank
where there's nothing useful to add. Shortcuts are the Windows/Linux
defaults (Chrome and VS Code remap most `Ctrl` to `Cmd` on macOS; GNOME
and Neovim are the same across platforms since they don't have a macOS
build/equivalent in Keylex's scope).

## VS Code

| Command | Default hotkey | Comment |
|---|---|---|
| Save | `Ctrl+S` | matches Keylex's `save` |
| Save As | `Ctrl+Shift+S` | |
| Save All | `Ctrl+K S` | |
| New File | `Ctrl+N` | |
| New Window | `Ctrl+Shift+N` | |
| Close Editor (tab) | `Ctrl+W` | matches Keylex's `close.tab` |
| Close Window | `Ctrl+Shift+W` | matches Keylex's `close.window`; no default key bound in `actions.toml` (`notify_only`) |
| Close Folder | `Ctrl+K F` | |
| Close Panel | | matches Keylex's `close.pane`; VS Code has no default key for this either |
| Kill Terminal | | matches Keylex's `close.terminal`; no VS Code default key |
| Toggle Sidebar Visibility | `Ctrl+B` | close-only in Keylex today (`close.sidebar`); VS Code's own binding is a toggle, not a close |
| Reopen Closed Editor | `Ctrl+Shift+T` | |
| Cut / Copy / Paste | `Ctrl+X` / `Ctrl+C` / `Ctrl+V` | |
| Undo / Redo | `Ctrl+Z` / `Ctrl+Y` | |
| Copy Line Down (duplicate line) | `Shift+Alt+Down` | Keylex's `duplicate.line` fallback keycode (`Ctrl+Shift+D`) doesn't match this — worth reconciling |
| Move Line Up / Down | `Alt+Up` / `Alt+Down` | candidate for a future `move.up` / `move.down` |
| Delete Line | `Ctrl+Shift+K` | |
| Insert Line Below / Above | `Ctrl+Enter` / `Ctrl+Shift+Enter` | |
| Toggle Line Comment | `Ctrl+/` | matches Keylex's `comment.line`; VS Code does have a default key, but Keylex's action is `notify_only` with none bound |
| Toggle Block Comment | `Shift+Alt+A` | |
| Format Document | `Shift+Alt+F` | |
| Trigger Suggest (autocomplete) | `Ctrl+Space` | |
| Go to Definition | `F12` | matches Keylex's `go_to.definition`; VS Code has a default key, Keylex currently doesn't bind one |
| Peek Definition | `Alt+F12` | |
| Go to References | `Shift+F12` | |
| Go to Line/Column | `Ctrl+G` | |
| Go Back / Forward | `Alt+Left` / `Alt+Right` | |
| Go to File (Quick Open) | `Ctrl+P` | |
| Go to Symbol in File | `Ctrl+Shift+O` | |
| Go to Symbol in Workspace | `Ctrl+T` | |
| Show All Commands | `Ctrl+Shift+P` | |
| Toggle Terminal | `` Ctrl+` `` | |
| New Terminal | `` Ctrl+Shift+` `` | |
| Split Editor | `Ctrl+\` | |
| Toggle Panel | `Ctrl+J` | |
| Toggle Full Screen | `F11` | |
| Zen Mode | `Ctrl+K Z` | |
| Toggle Word Wrap | `Alt+Z` | |
| Find / Replace | `Ctrl+F` / `Ctrl+H` | |
| Find in Files / Replace in Files | `Ctrl+Shift+F` / `Ctrl+Shift+H` | |
| Add Cursor Above / Below | `Ctrl+Alt+Up` / `Ctrl+Alt+Down` | |
| Add Selection to Next Find Match | `Ctrl+D` | |
| Select All Occurrences | `Ctrl+Shift+L` | |
| Next / Previous Editor (tab) | `Ctrl+Tab` / `Ctrl+Shift+Tab` | |

## Google Chrome

| Command | Default hotkey | Comment |
|---|---|---|
| New Tab | `Ctrl+T` | |
| New Window | `Ctrl+N` | |
| New Incognito Window | `Ctrl+Shift+N` | |
| Close Tab | `Ctrl+W` | matches Keylex's `close.tab` |
| Close Window | `Ctrl+Shift+W` | matches Keylex's `close.window` |
| Reopen Closed Tab | `Ctrl+Shift+T` | |
| Next Tab / Previous Tab | `Ctrl+Tab` / `Ctrl+Shift+Tab` | |
| Go to Tab 1-8 | `Ctrl+1` .. `Ctrl+8` | |
| Go to Last Tab | `Ctrl+9` | |
| Toggle Side Panel | | matches Keylex's `close.sidebar`, currently mapped to `chrome.sidepanel.toggle` in `capabilities.toml` — no Chrome default key |
| Back / Forward | `Alt+Left` / `Alt+Right` | |
| Reload / Hard Reload | `Ctrl+R` (or `F5`) / `Ctrl+Shift+R` | |
| Open Home Page | `Alt+Home` | |
| Focus Address Bar | `Ctrl+L` (or `F6`, `Alt+D`) | |
| Zoom In / Out / Reset | `Ctrl++` / `Ctrl+-` / `Ctrl+0` | |
| Full Screen | `F11` | |
| Find in Page | `Ctrl+F` | |
| Print | `Ctrl+P` | |
| Save Page As | `Ctrl+S` | |
| Bookmark This Page | `Ctrl+D` | |
| Bookmark All Tabs | `Ctrl+Shift+D` | |
| View Page Source | `Ctrl+U` | |
| Open DevTools | `Ctrl+Shift+I` (or `F12`) | |
| Open Downloads | `Ctrl+J` | |
| Open History | `Ctrl+H` | |
| Open File | `Ctrl+O` | |
| Open Task Manager | `Shift+Esc` | |
| Clear Browsing Data | `Ctrl+Shift+Delete` | |

## Linux (GNOME)

Defaults from vanilla GNOME Shell; distros (Ubuntu, Fedora Workstation)
sometimes rebind a few of these — worth re-checking on whatever GNOME
build `extensions/linux-extension` ends up targeting.

| Command | Default hotkey | Comment |
|---|---|---|
| Open Activities Overview | `Super` | |
| Show All Applications | `Super+A` | |
| Switch Applications | `Alt+Tab` | |
| Switch Windows of Current App | `` Alt+` `` | |
| Switch Windows (accessibility order) | `Alt+Esc` | |
| Show Notification List | `Super+V` | |
| Lock Screen | `Super+L` | |
| Log Out / Power Off Dialog | `Ctrl+Alt+Delete` | |
| Close Window | `Alt+F4` | candidate native path for Keylex's `close.window` on Linux via the system listener |
| Maximize Window | `Super+Up` | |
| Unmaximize / Restore Window | `Super+Down` | |
| Snap Window Left | `Super+Left` | matches Keylex's `move.left` |
| Snap Window Right | `Super+Right` | matches Keylex's `move.right`; Keylex's own binding (`win+left`/`win+right`) matches this already |
| Switch to Workspace Up / Down | `Super+Page Up` / `Super+Page Down` | |
| Move Window to Workspace Up / Down | `Shift+Super+Page Up` / `Shift+Super+Page Down` | |
| Run Command Dialog | `Alt+F2` | |
| Screenshot (full screen) | `Print Screen` | matches Keylex's `show.desktop`? no — Keylex's `show.desktop` fallback is `win+d`, which isn't a GNOME default; `Print Screen` is GNOME's own screenshot binding and does something different |
| Screenshot (active window) | `Alt+Print Screen` | |
| Screenshot (selected area) | `Shift+Print Screen` | |
| Screenshot to clipboard | `Ctrl+Print Screen` | |
| Screen Recording Toggle | `Ctrl+Shift+Alt+R` | |
| Switch Input Source | `Super+Space` | |
| Open Terminal | `Ctrl+Alt+T` | common distro default (Ubuntu/Fedora), not vanilla GNOME |

## Neovim

Neovim's own defaults are modal `Normal`-mode commands and Ex-commands
rather than app-wide hotkeys; nothing here needs a modifier key unless
noted. This is what `config/targets.toml`'s inline `neovim` target
(`exempt_command_grammar = true`) is drawing its command strings from.

| Command | Default hotkey | Comment |
|---|---|---|
| Write (save) buffer | `:w` | matches Keylex's `save`; already used as neovim's `[target.supports]` value |
| Quit | `:q` | |
| Write and quit | `:wq` (or `:x`) | |
| Quit without saving | `:q!` | |
| Delete (close) buffer | `:bd` | matches Keylex's `close.tab`; already used as neovim's `[target.supports]` value |
| Next / previous buffer | `:bn` / `:bp` | |
| Split window horizontally | `:sp` (or `Ctrl+W s`) | |
| Split window vertically | `:vsp` (or `Ctrl+W v`) | |
| Close current window (split) | `Ctrl+W q` | matches Keylex's `close.pane`/`close.window` depending on how a split maps conceptually |
| Switch window (split) | `Ctrl+W w` | |
| Open terminal | `:terminal` | |
| Close terminal | `exit` in the shell, or `:bd!` | matches Keylex's `close.terminal`; no single default key |
| Delete line | `dd` | |
| Yank (copy) line | `yy` | |
| Duplicate line | `yyp` | matches Keylex's `duplicate.line`; already used as neovim's `[target.supports]` value |
| Paste after / before | `p` / `P` | |
| Undo / Redo | `u` / `Ctrl+R` | |
| Search forward / backward | `/` / `?` | |
| Next / previous match | `n` / `N` | |
| Go to top / bottom of file | `gg` / `G` | |
| Go to definition (tag jump) | `Ctrl+]` | matches Keylex's `go_to.definition`; real "jump to LSP definition" needs a plugin/LSP client (`gd` in many configs), not a vanilla default |
| Go back (jump list) | `Ctrl+O` | |
| Comment line | | no vanilla default — commenting is not built in; needs a plugin (`tpope/commentary`, or Neovim's own `gc` once/if it ships built-in) before `comment.line` can get a real neovim mapping |
| Insert mode | `i` / `a` / `o` / `O` | |
| Visual / Visual Line / Visual Block mode | `v` / `V` / `Ctrl+V` | |

## Notes for planning new Keylex actions

- `close.tab`, `save`, and `duplicate.line` already have real mappings on
  all four surfaces above (VS Code, Chrome where applicable, GNOME N/A,
  Neovim) — good candidates to double check for consistency across
  `capabilities.toml` files.
- `comment.line` and `go_to.definition` are `notify_only` in
  `config/actions.toml` today (no fallback keycode) even though VS Code
  has real default keys for both (`Ctrl+/`, `F12`) — worth reconsidering
  once a safe cross-app fallback guess is picked.
- GNOME's window-snap and workspace shortcuts (`Super+Left/Right`,
  `Super+Page Up/Down`) are a ready-made source for new `move.*`
  modifiers/locations beyond `move.left`/`move.right` (e.g.
  `move.up`/`move.down` for workspace switching) if that's ever wanted.
- Neovim has no built-in line-commenting default — any `comment.line`
  support for the `neovim` target depends on picking (and documenting)
  a plugin dependency first.
