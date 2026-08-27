# Hotkey / command reference

A full inventory of the built-in commands each target application ships
with, one section per app. The first column is always that app's own
**native command identifier** — the literal string the app itself uses
internally (VS Code's command IDs, Chrome's internal `IDC_*` constants,
GNOME's `gsettings` schema keys, Neovim's canonical command names) — not
a made-up label. Default hotkey and comment are both optional; many
commands have no default binding (menu/command-palette/context-menu
only) and are listed anyway, since the point here is everything each app
can theoretically do, not just what's bound out of the box.

This intentionally does not filter or annotate against Keylex's own
`vocabulary.toml`/`actions.toml` — it's a raw reference of each app's
command surface to plan future Keylex actions against, not a diff
against what Keylex already implements.

## VS Code

*Command IDs and default keybindings compiled from the official VS Code
documentation (code.visualstudio.com/docs/reference/default-keybindings,
docs/configure/keybindings) and the vscode-docs source repo. Windows/Linux
defaults — macOS remaps most `Ctrl` to `Cmd`. Core, built-in commands
only; extension-contributed commands are out of scope.*

### File

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.files.newUntitledFile` | Ctrl+N | New untitled text file |
| `workbench.action.files.openFile` | Ctrl+O | Open File... |
| `workbench.action.files.openFileFolder` | Ctrl+O | Open File or Folder (browse dialog) |
| `workbench.action.files.openFolder` | Ctrl+K Ctrl+O | Open Folder... |
| `workbench.action.addRootFolder` | | Add Folder to Workspace... |
| `workbench.action.openWorkspace` | | Open Workspace from File... |
| `workbench.action.saveWorkspaceAs` | | Save Workspace As... |
| `workbench.action.openRecent` | Ctrl+R | Open Recent (files/folders/workspaces) |
| `workbench.action.files.save` | Ctrl+S | Save |
| `workbench.action.files.saveAs` | Ctrl+Shift+S | Save As... |
| `saveAll` | Ctrl+K S | Save All open editors |
| `workbench.action.files.saveWithoutFormatting` | Ctrl+K Ctrl+Shift+S | Save without running formatters |
| `workbench.action.files.revert` | | Revert File to last saved state |
| `workbench.action.closeActiveEditor` | Ctrl+W / Ctrl+F4 | Close Editor |
| `workbench.action.closeAllEditors` | Ctrl+K Ctrl+W | Close All Editors |
| `workbench.action.closeUnmodifiedEditors` | Ctrl+K U | Close Unmodified Editors |
| `workbench.action.closeEditorsInGroup` | Ctrl+K W | Close all editors in the active group |
| `workbench.action.closeOtherEditors` | | Close Other Editors in group |
| `workbench.action.closeEditorsToTheRight` | | Close Editors to the Right |
| `workbench.action.closeFolder` | Ctrl+K F | Close Folder |
| `workbench.action.closeWindow` | Ctrl+Shift+W / Alt+F4 | Close Window |
| `workbench.action.newWindow` | Ctrl+Shift+N | Open a new empty window |
| `workbench.action.reopenClosedEditor` | Ctrl+Shift+T | Reopen Closed Editor |
| `workbench.action.files.copyPathOfActiveFile` | Ctrl+K P | Copy Path of Active File |
| `workbench.action.files.revealActiveFileInWindows` | Ctrl+K R | Reveal Active File in Explorer/Finder |
| `workbench.files.action.compareWithSaved` | Ctrl+K D | Compare Active File with Saved |
| `workbench.action.duplicateWorkspaceInNewWindow` | | Duplicate As Workspace in New Window |

### Edit

| Command ID | Default hotkey | Comment |
|---|---|---|
| `editor.action.clipboardCutAction` | Ctrl+X | Cut (cuts whole line if selection is empty) |
| `editor.action.clipboardCopyAction` | Ctrl+C | Copy (copies whole line if selection is empty) |
| `editor.action.clipboardPasteAction` | Ctrl+V | Paste |
| `editor.action.clipboardCopyWithSyntaxHighlightingAction` | | Copy With Syntax Highlighting (as HTML) |
| `undo` | Ctrl+Z | Undo |
| `redo` | Ctrl+Y / Ctrl+Shift+Z | Redo |
| `cursorUndo` | Ctrl+U | Undo last cursor operation |
| `editor.action.insertLineAfter` | Ctrl+Enter | Insert Line Below |
| `editor.action.insertLineBefore` | Ctrl+Shift+Enter | Insert Line Above |
| `editor.action.deleteLines` | Ctrl+Shift+K | Delete Line |
| `editor.action.moveLinesDownAction` | Alt+Down | Move Line Down |
| `editor.action.moveLinesUpAction` | Alt+Up | Move Line Up |
| `editor.action.copyLinesDownAction` | Shift+Alt+Down | Copy Line Down |
| `editor.action.copyLinesUpAction` | Shift+Alt+Up | Copy Line Up |
| `editor.action.indentLines` | Ctrl+] | Indent Line |
| `editor.action.outdentLines` | Ctrl+[ | Outdent Line |
| `tab` | Tab | Indent / insert tab |
| `outdent` | Shift+Tab | Outdent |
| `deleteLeft` | Backspace | Delete character to the left |
| `deleteRight` | Delete | Delete character to the right |
| `deleteWordLeft` | Ctrl+Backspace | Delete Word (left) |
| `deleteWordRight` | Ctrl+Delete | Delete Word (right) |
| `editor.action.commentLine` | Ctrl+/ | Toggle Line Comment |
| `editor.action.addCommentLine` | Ctrl+K Ctrl+C | Add Line Comment |
| `editor.action.removeCommentLine` | Ctrl+K Ctrl+U | Remove Line Comment |
| `editor.action.blockComment` | Shift+Alt+A | Toggle Block Comment |
| `editor.action.formatDocument` | Shift+Alt+F | Format Document |
| `editor.action.formatSelection` | Ctrl+K Ctrl+F | Format Selection |
| `editor.action.organizeImports` | Shift+Alt+O | Organize Imports |
| `editor.action.autoFix` | Shift+Alt+. | Apply Auto Fix |
| `editor.action.quickFix` | Ctrl+. | Show Quick Fix menu |
| `editor.action.refactor` | Ctrl+Shift+R | Show Refactor menu |
| `editor.action.sourceAction` | | Show Source Action menu |
| `editor.action.rename` | F2 | Rename Symbol |
| `editor.action.linkedEditing` | Ctrl+Shift+F2 | Start Linked/Type Editing (rename tag pairs etc.) |
| `editor.action.jumpToBracket` | Ctrl+Shift+\ | Go to Bracket |
| `editor.action.removeBrackets` | Ctrl+Alt+Backspace | Remove Brackets |
| `editor.action.trimTrailingWhitespace` | Ctrl+K Ctrl+X | Trim Trailing Whitespace |
| `editor.action.inPlaceReplace.up` | Ctrl+Shift+, | Replace with Previous Value |
| `editor.action.inPlaceReplace.down` | Ctrl+Shift+. | Replace with Next Value |
| `editor.action.transpose` | | Transpose Letters around cursor |
| `editor.action.joinLines` | | Join current line with the next |
| `editor.action.sortLinesAscending` | | Sort Lines Ascending |
| `editor.action.sortLinesDescending` | | Sort Lines Descending |
| `editor.action.duplicateSelection` | | Duplicate Selection |
| `editor.action.transformToUppercase` | | Transform Selection to UPPERCASE |
| `editor.action.transformToLowercase` | | Transform Selection to lowercase |
| `editor.action.transformToTitlecase` | | Transform Selection to Title Case |
| `editor.action.transformToSnakecase` | | Transform Selection to snake_case |
| `editor.emmet.action.expandAbbreviation` | Tab | Emmet: Expand Abbreviation |
| `editor.action.toggleWordWrap` | Alt+Z | Toggle Word Wrap |
| `editor.action.toggleTabFocusMode` | Ctrl+M | Toggle whether Tab moves focus instead of indenting |
| `editor.action.showHover` | Ctrl+K Ctrl+I | Show Hover for symbol under cursor |
| `editor.action.setSelectionAnchor` | Ctrl+K Ctrl+B | Set Selection Anchor |
| `editor.action.selectFromAnchorToCursor` | Ctrl+K Ctrl+K | Select from Anchor to Cursor |
| `editor.action.cancelSelectionAnchor` | | Cancel Selection Anchor |
| `editor.action.showContextMenu` | Shift+F10 | Show Editor Context Menu |

### Code Folding

| Command ID | Default hotkey | Comment |
|---|---|---|
| `editor.fold` | Ctrl+Shift+[ | Fold (collapse) region |
| `editor.unfold` | Ctrl+Shift+] | Unfold (uncollapse) region |
| `editor.toggleFold` | Ctrl+K Ctrl+L | Toggle Fold region |
| `editor.foldRecursively` | Ctrl+K Ctrl+[ | Fold region and all subregions |
| `editor.unfoldRecursively` | Ctrl+K Ctrl+] | Unfold region and all subregions |
| `editor.toggleFoldRecursively` | Ctrl+K Ctrl+Shift+L | Toggle fold region and subregions |
| `editor.foldAll` | Ctrl+K Ctrl+0 | Fold All regions |
| `editor.unfoldAll` | Ctrl+K Ctrl+J | Unfold All regions |
| `editor.foldAllBlockComments` | Ctrl+K Ctrl+/ | Fold All Block Comments |
| `editor.foldAllMarkerRegions` | Ctrl+K Ctrl+8 | Fold All Regions marked by `//region` |
| `editor.unfoldAllMarkerRegions` | Ctrl+K Ctrl+9 | Unfold All Marker Regions |
| `editor.foldAllExcept` | Ctrl+K Ctrl+- | Fold All Except Selected |
| `editor.unfoldAllExcept` | Ctrl+K Ctrl+= | Unfold All Except Selected |
| `editor.foldLevel1` … `editor.foldLevel7` | Ctrl+K Ctrl+1 … Ctrl+K Ctrl+7 | Fold to nesting Level N |
| `editor.createFoldingRangeFromSelection` | Ctrl+K Ctrl+, | Create Manual Folding Range from Selection |
| `editor.removeManualFoldingRanges` | Ctrl+K Ctrl+. | Remove Manual Folding Ranges |

### Selection & Multi-cursor

| Command ID | Default hotkey | Comment |
|---|---|---|
| `editor.action.selectAll` | Ctrl+A | Select All |
| `expandLineSelection` | Ctrl+L | Select Current Line |
| `editor.action.smartSelect.expand` | Shift+Alt+Right | Expand Selection (AST-aware) |
| `editor.action.smartSelect.shrink` | Shift+Alt+Left | Shrink Selection |
| `editor.action.insertCursorAbove` | Ctrl+Alt+Up | Add Cursor Above |
| `editor.action.insertCursorBelow` | Ctrl+Alt+Down | Add Cursor Below |
| `editor.action.insertCursorAtEndOfEachLineSelected` | Shift+Alt+I | Add Cursors to Line Ends of selection |
| `editor.action.addSelectionToNextFindMatch` | Ctrl+D | Add Selection to Next Find Match |
| `editor.action.addSelectionToPreviousFindMatch` | | Add Selection to Previous Find Match |
| `editor.action.moveSelectionToNextFindMatch` | Ctrl+K Ctrl+D | Move Last Selection to Next Find Match |
| `editor.action.selectHighlights` | Ctrl+Shift+L | Select All Occurrences of Current Selection |
| `editor.action.changeAll` | Ctrl+F2 | Change/select All Occurrences of Current Word |
| `editor.action.selectAllMatches` | Alt+Enter | Select All Occurrences of Find Match (from Find widget) |
| `cursorColumnSelectUp` | Ctrl+Shift+Alt+Up | Column (box) select up |
| `cursorColumnSelectDown` | Ctrl+Shift+Alt+Down | Column (box) select down |
| `cursorColumnSelectLeft` | Ctrl+Shift+Alt+Left | Column select left |
| `cursorColumnSelectRight` | Ctrl+Shift+Alt+Right | Column select right |
| `cursorColumnSelectPageUp` | Ctrl+Shift+Alt+PageUp | Column select page up |
| `cursorColumnSelectPageDown` | Ctrl+Shift+Alt+PageDown | Column select page down |
| `removeSecondaryCursors` | Escape | Collapse multi-cursor down to a single cursor |
| `cancelSelection` | Escape | Cancel current selection |
| `editor.action.wordHighlight.next` | F7 | Go to Next symbol/word highlight |
| `editor.action.wordHighlight.prev` | Shift+F7 | Go to Previous symbol/word highlight |

### View & Workbench Navigation

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.showCommands` | Ctrl+Shift+P / F1 | Show Command Palette |
| `workbench.action.quickOpen` | Ctrl+P | Go to File... (Quick Open) |
| `workbench.action.quickOpenView` | Ctrl+Q | Quick Open a View |
| `workbench.view.explorer` | Ctrl+Shift+E | Show Explorer / toggle focus |
| `workbench.view.search` | Ctrl+Shift+F | Show Search |
| `workbench.view.scm` | Ctrl+Shift+G | Show Source Control |
| `workbench.view.debug` | Ctrl+Shift+D | Show Run and Debug |
| `workbench.view.extensions` | Ctrl+Shift+X | Show Extensions |
| `workbench.actions.view.problems` | Ctrl+Shift+M | Show Problems Panel |
| `workbench.action.output.toggleOutput` | Ctrl+Shift+U | Show Output Panel |
| `workbench.debug.action.toggleRepl` | Ctrl+Shift+Y | Show Debug Console |
| `workbench.action.terminal.toggleTerminal` | Ctrl+` | Show/hide Integrated Terminal |
| `workbench.action.toggleDevTools` | Ctrl+Shift+I | Toggle Developer Tools |
| `workbench.action.quickOpenPreviousRecentlyUsedEditorInGroup` | Ctrl+Tab | Show/cycle recently used editors in group |
| `workbench.action.quickOpenLeastRecentlyUsedEditorInGroup` | Ctrl+Shift+Tab | Cycle recently used editors backward |
| `workbench.action.quickOpenNavigateNextInFilePicker` | Ctrl+P (held) | Navigate Next in Quick Open file picker |
| `workbench.action.quickOpenNavigateNextInRecentFilesPicker` | Ctrl+R (held) | Navigate in Open Recent picker |
| `workbench.action.quickOpenNavigateNextInViewPicker` | Ctrl+Q (held) | Navigate in Quick View picker |
| `workbench.action.quickOpenNavigateNextInEditorPicker` | Ctrl+Tab (held) | Navigate in editor-history picker |
| `workbench.action.closeQuickOpen` | Escape | Close the Quick Open / Command Palette widget |
| `workbench.action.navigateBack` | Alt+Left | Go Back |
| `workbench.action.navigateForward` | Alt+Right | Go Forward |
| `workbench.action.quickInputBack` | | Go back one step in a multi-step Quick Input |

### Go

| Command ID | Default hotkey | Comment |
|---|---|---|
| `editor.action.revealDefinition` | F12 | Go to Definition |
| `editor.action.revealDefinitionAside` | Ctrl+K F12 | Open Definition to the Side |
| `editor.action.peekDefinition` | Alt+F12 | Peek Definition inline |
| `editor.action.goToTypeDefinition` | | Go to Type Definition |
| `editor.action.goToImplementation` | Ctrl+F12 | Go to Implementation |
| `editor.action.peekImplementation` | Ctrl+Shift+F12 | Peek Implementation |
| `editor.action.goToReferences` | Shift+F12 | Go to References |
| `references-view.findReferences` | Shift+Alt+F12 | Find All References (References view) |
| `references-view.showCallHierarchy` | Shift+Alt+H | Show Call Hierarchy |
| `references-view.next` | F4 | Go to Next Reference (References view) |
| `references-view.prev` | Shift+F4 | Go to Previous Reference (References view) |
| `goToNextReference` | F4 | Go to Next Reference (Peek widget) |
| `goToPreviousReference` | Shift+F4 | Go to Previous Reference (Peek widget) |
| `workbench.action.gotoLine` | Ctrl+G | Go to Line/Column... |
| `workbench.action.gotoSymbol` | Ctrl+Shift+O | Go to Symbol in Editor... |
| `workbench.action.showAllSymbols` | Ctrl+T | Go to Symbol in Workspace... |
| `editor.action.jumpToBracket` | Ctrl+Shift+\ | Go to Matching Bracket |
| `workbench.action.navigateToLastEditLocation` | Ctrl+K Ctrl+Q | Go to Last Edit Location |
| `editor.action.marker.next` | Alt+F8 | Go to Next Problem in file |
| `editor.action.marker.prev` | Shift+Alt+F8 | Go to Previous Problem in file |
| `editor.action.marker.nextInFiles` | F8 | Go to Next Problem (across files) |
| `editor.action.marker.prevInFiles` | Shift+F8 | Go to Previous Problem (across files) |
| `editor.action.dirtydiff.next` | Alt+F3 | Go to Next Change (uncommitted diff gutter) |
| `editor.action.dirtydiff.previous` | Shift+Alt+F3 | Go to Previous Change |
| `workbench.action.editor.nextChange` | Alt+F5 | Go to Next Change |
| `workbench.action.editor.previousChange` | Shift+Alt+F5 | Go to Previous Change |
| `workbench.action.compareEditor.nextChange` | Alt+F5 | Next Change (diff editor) |
| `workbench.action.compareEditor.previousChange` | Shift+Alt+F5 | Previous Change (diff editor) |
| `workbench.action.compareEditor.openSide` | Ctrl+K Shift+O | Open Changes to the Side |
| `breadcrumbs.focus` | Ctrl+Shift+; | Focus Breadcrumbs |
| `breadcrumbs.focusAndSelect` | Ctrl+Shift+. | Focus and Select Breadcrumbs |
| `breadcrumbs.focusNext` / `breadcrumbs.focusPrevious` | Right / Left | Move focus along breadcrumb trail |
| `breadcrumbs.revealFocused` | Enter | Reveal focused breadcrumb entry |

### Run & Debug

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.debug.start` | F5 | Start Debugging |
| `workbench.action.debug.run` | Ctrl+F5 | Start Without Debugging |
| `workbench.action.debug.stop` | Shift+F5 | Stop debugging session |
| `workbench.action.debug.restart` | Ctrl+Shift+F5 | Restart debugging session |
| `workbench.action.debug.continue` | F5 | Continue (while paused) |
| `workbench.action.debug.pause` | F6 | Pause |
| `workbench.action.debug.stepOver` | F10 | Step Over |
| `workbench.action.debug.stepInto` | F11 | Step Into |
| `workbench.action.debug.stepOut` | Shift+F11 | Step Out |
| `workbench.action.debug.stepIntoTarget` | Ctrl+F11 | Step Into Target... |
| `workbench.action.debug.disconnect` | Shift+F5 | Disconnect from attached debuggee |
| `workbench.action.debug.selectandstart` | | Select and Start Debugging (config picker) |
| `debug.addConfiguration` | | Add Configuration... to launch.json |
| `editor.debug.action.toggleBreakpoint` | F9 | Toggle Breakpoint |
| `editor.debug.action.conditionalBreakpoint` | | Add Conditional Breakpoint... |
| `editor.debug.action.toggleInlineBreakpoint` | | Add Inline Breakpoint |
| `editor.debug.action.showDebugHover` | Ctrl+K Ctrl+I | Show hover for value while debugging |
| `editor.debug.action.closeExceptionWidget` | Escape | Close inline exception widget |
| `workbench.action.debug.nextConsole` | Ctrl+PageDown | Focus Next Debug Console |
| `workbench.action.debug.prevConsole` | Ctrl+PageUp | Focus Previous Debug Console |
| `workbench.debug.action.toggleRepl` | Ctrl+Shift+Y | Toggle/show Debug Console |
| `repl.action.acceptInput` | Enter | Execute typed expression in Debug Console |
| `repl.execute` | Ctrl+Enter / Enter | Execute cell in interactive Debug Console/REPL |
| `repl.action.find` | Ctrl+Alt+F | Find in Debug Console |
| `workbench.view.debug` | Ctrl+Shift+D | Show Run and Debug view |

### Terminal

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.terminal.toggleTerminal` | Ctrl+` | Show/Hide the Integrated Terminal |
| `workbench.action.terminal.new` | Ctrl+Shift+` | Create New Terminal |
| `workbench.action.terminal.newInNewWindow` | Ctrl+Shift+Alt+` | Create New Terminal in a new window |
| `workbench.action.terminal.split` | Ctrl+Shift+5 | Split Terminal |
| `workbench.action.terminal.splitActiveTab` | Ctrl+Shift+5 | Split the active terminal tab |
| `workbench.action.terminal.killActiveTab` | Delete | Kill the active terminal tab |
| `workbench.action.terminal.killEditor` | Ctrl+W / Ctrl+F4 | Kill terminal opened as an editor tab |
| `workbench.action.terminal.focus` | Ctrl+Down (in editor at bottom) | Focus Terminal |
| `workbench.action.terminal.focusNext` | Ctrl+PageDown | Focus Next Terminal |
| `workbench.action.terminal.focusPrevious` | Ctrl+PageUp | Focus Previous Terminal |
| `workbench.action.terminal.focusTabs` | Ctrl+Shift+\ | Focus Terminal Tabs list |
| `workbench.action.terminal.focusFind` | Ctrl+F | Focus Find in terminal |
| `workbench.action.terminal.hideFind` | Escape | Hide terminal Find widget |
| `workbench.action.terminal.clearSelection` | Escape | Clear terminal selection |
| `workbench.action.terminal.copySelection` | Ctrl+Shift+C | Copy Selection |
| `workbench.action.terminal.copyAndClearSelection` | Ctrl+C | Copy selection and clear it |
| `workbench.action.terminal.paste` | Ctrl+V / Ctrl+Shift+V | Paste into terminal |
| `workbench.action.terminal.renameActiveTab` | F2 | Rename active terminal tab |
| `workbench.action.terminal.runRecentCommand` | Ctrl+Alt+R | Run Recent Command from shell history |
| `workbench.action.terminal.goToRecentDirectory` | Ctrl+G | Go to Recent Directory |
| `workbench.action.terminal.scrollDown` | Ctrl+Alt+PageDown | Scroll Down (line) |
| `workbench.action.terminal.scrollUp` | Ctrl+Alt+PageUp | Scroll Up (line) |
| `workbench.action.terminal.scrollDownPage` | Shift+PageDown | Scroll Down (page) |
| `workbench.action.terminal.scrollUpPage` | Shift+PageUp | Scroll Up (page) |
| `workbench.action.terminal.scrollToTop` | Ctrl+Home | Scroll to Top |
| `workbench.action.terminal.scrollToBottom` | Ctrl+End | Scroll to Bottom |
| `workbench.action.terminal.scrollToNextCommand` | Ctrl+Down | Scroll to Next Shell-Integration Command |
| `workbench.action.terminal.scrollToPreviousCommand` | Ctrl+Up | Scroll to Previous Shell-Integration Command |
| `workbench.action.terminal.selectToNextCommand` | Ctrl+Shift+Down | Select to Next Command |
| `workbench.action.terminal.selectToPreviousCommand` | Ctrl+Shift+Up | Select to Previous Command |
| `workbench.action.terminal.findNext` | F3 | Find Next in terminal |
| `workbench.action.terminal.findPrevious` | Shift+F3 | Find Previous in terminal |
| `workbench.action.terminal.toggleFindCaseSensitive` | Alt+C | Toggle Find Case Sensitive |
| `workbench.action.terminal.toggleFindRegex` | Alt+R | Toggle Find Regex |
| `workbench.action.terminal.toggleFindWholeWord` | Alt+W | Toggle Find Whole Word |
| `workbench.action.terminal.sizeToContentWidth` | Alt+Z | Toggle terminal wrapping to content width |
| `workbench.action.terminal.showQuickFixes` | Ctrl+. | Show terminal Quick Fixes |
| `workbench.action.terminal.openDetectedLink` | Ctrl+Shift+G | Open Detected Link... |
| `workbench.action.terminal.searchWorkspace` | Ctrl+Shift+F | Search selected text in Workspace from terminal |
| `workbench.action.terminal.triggerSuggest` | Ctrl+Space | Trigger terminal shell-integration IntelliSense |
| `workbench.action.terminal.chat.start` | Ctrl+I | Open Terminal Inline Chat |
| `workbench.action.createTerminalEditor` | | Create New Terminal in Editor Area |

### Git (built-in)

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.view.scm` | Ctrl+Shift+G | Show Source Control view |
| `scm.acceptInput` | Ctrl+Enter | Commit staged changes (accept SCM input box) |
| `git.commit` | | Commit |
| `git.commitStaged` | | Commit Staged |
| `git.commitAll` | | Commit All (stage everything and commit) |
| `git.commitAllSigned` | | Commit All (Signed Off) |
| `git.commitStagedAmend` | | Commit Staged (Amend) |
| `git.stage` | | Stage Changes |
| `git.stageAll` | | Stage All Changes |
| `git.stageSelectedRanges` | Ctrl+K Ctrl+Alt+S | Stage Selected Ranges (in diff editor) |
| `git.unstage` | | Unstage Changes |
| `git.unstageAll` | | Unstage All Changes |
| `git.unstageSelectedRanges` | Ctrl+K Ctrl+N | Unstage Selected Ranges |
| `git.revertSelectedRanges` | Ctrl+K Ctrl+R | Revert Selected Ranges |
| `git.clean` | | Discard Changes |
| `git.cleanAll` | | Discard All Changes |
| `git.push` | | Push |
| `git.pushTo` | | Push to... (remote/branch picker) |
| `git.pushForce` | | Push (Force) |
| `git.pull` | | Pull |
| `git.pullFrom` | | Pull from... |
| `git.pullRebase` | | Pull (Rebase) |
| `git.sync` | | Sync (pull then push) |
| `git.fetch` | | Fetch |
| `git.fetchAll` | | Fetch From All Remotes |
| `git.checkout` | | Checkout to... branch/tag |
| `git.branch` | | Create Branch... |
| `git.branchFrom` | | Create Branch From... |
| `git.deleteBranch` | | Delete Branch... |
| `git.merge` | | Merge Branch... |
| `git.rebase` | | Rebase Branch... |
| `git.createTag` | | Create Tag |
| `git.deleteTag` | | Delete Tag |
| `git.stash` | | Stash |
| `git.stashApply` | | Apply Stash... |
| `git.stashPop` | | Pop Stash... |
| `git.stashDrop` | | Drop Stash... |
| `git.init` | | Initialize Repository |
| `git.clone` | | Clone |
| `git.close` | | Close Repository |
| `git.openChange` | | Open Changes (diff view for selected file) |
| `git.openFile` | | Open File (from a change) |
| `git.openAllChanges` | | Open All Changes |
| `git.undoCommit` | | Undo Last Commit |
| `git.ignore` | | Add File to .gitignore |
| `git.showOutput` | | Show Git Output |
| `git.viewHistory` | | View Git History for file/repo |
| `git.compareWithHead` | | Compare Changes with HEAD |

### Search

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.view.search` | Ctrl+Shift+F | Show Search / Find in Files |
| `workbench.action.replaceInFiles` | Ctrl+Shift+H | Replace in Files |
| `actions.find` | Ctrl+F | Find (in current editor) |
| `editor.action.startFindReplaceAction` | Ctrl+H | Replace (in current editor) |
| `editor.action.nextMatchFindAction` | F3 | Find Next |
| `editor.action.previousMatchFindAction` | Shift+F3 | Find Previous |
| `editor.action.nextSelectionMatchFindAction` | Ctrl+F3 | Find Next Selection Match |
| `editor.action.previousSelectionMatchFindAction` | Ctrl+Shift+F3 | Find Previous Selection Match |
| `toggleFindCaseSensitive` | Alt+C | Toggle Find Case Sensitive (editor Find widget) |
| `toggleFindRegex` | Alt+R | Toggle Find Use Regular Expression |
| `toggleFindWholeWord` | Alt+W | Toggle Find Whole Word |
| `toggleSearchCaseSensitive` | Alt+C | Toggle Match Case (Search view) |
| `toggleSearchWholeWord` | Alt+W | Toggle Match Whole Word (Search view) |
| `toggleSearchRegex` | Alt+R | Toggle Use Regular Expression (Search view) |
| `workbench.action.search.toggleQueryDetails` | | Toggle Search Details (include/exclude fields) |
| `search.action.focusNextSearchResult` | F4 | Focus Next Search Result |
| `search.action.focusPreviousSearchResult` | Shift+F4 | Focus Previous Search Result |
| `search.action.openInEditor` | | Open Results in Editor (Search Editor) |
| `search.action.focusQueryEditorWidget` | | Focus Search Editor Input |
| `rerunSearchEditorSearch` | Ctrl+Shift+R | Search Again (in Search Editor) |
| `search.searchEditor.action.deleteFileResults` | Ctrl+Shift+Backspace | Delete File Results (Search Editor) |
| `history.showNext` | Alt+Down | Show Next Search Term in field history |
| `history.showPrevious` | Alt+Up | Show Previous Search Term |
| `workbench.action.terminal.searchWorkspace` | Ctrl+Shift+F | Search Workspace using terminal selection |

### Notebook

| Command ID | Default hotkey | Comment |
|---|---|---|
| `notebook.cell.execute` | Ctrl+Alt+Enter | Execute the focused cell |
| `notebook.cell.executeAndInsertBelow` | Alt+Enter | Execute cell and insert a new one below |
| `notebook.cell.executeAndSelectBelow` | Shift+Enter | Execute cell and select the next one |
| `notebook.cell.insertCodeCellAbove` | Ctrl+Shift+Enter | Insert Code Cell Above |
| `notebook.cell.insertCodeCellBelow` | Ctrl+Enter | Insert Code Cell Below |
| `notebook.cell.edit` | Enter | Enter edit mode on the focused cell |
| `notebook.cell.quitEdit` | Escape | Stop editing / return to command mode |
| `notebook.cell.delete` | Delete (command mode) | Delete Cell |
| `notebook.cell.moveUp` | Alt+Up | Move Cell Up |
| `notebook.cell.moveDown` | Alt+Down | Move Cell Down |
| `notebook.cell.copyUp` | Shift+Alt+Up | Copy Cell Up |
| `notebook.cell.copyDown` | Shift+Alt+Down | Copy Cell Down |
| `notebook.cell.split` | Ctrl+Shift+\ | Split Cell at cursor |
| `notebook.cell.joinAbove` | Shift+Alt+Win+J | Join With Previous Cell |
| `notebook.cell.joinBelow` | Alt+Win+J | Join With Next Cell |
| `notebook.cell.changeToCode` | Y (command mode) | Change Cell to Code |
| `notebook.cell.changeToMarkdown` | M (command mode) | Change Cell to Markdown |
| `notebook.cell.changeLanguage` | Ctrl+K M | Change Cell Language |
| `notebook.cell.detectLanguage` | Shift+Alt+D | Detect Cell Language |
| `notebook.cell.clearOutputs` | Alt+Delete | Clear Cell Outputs |
| `notebook.cell.collapseCellInput` | Ctrl+K Ctrl+C | Collapse Cell Input |
| `notebook.cell.expandCellInput` | Ctrl+K Ctrl+C | Expand Cell Input |
| `notebook.cell.collapseCellOutput` | Ctrl+K T | Collapse Cell Output |
| `notebook.cell.expandCellOutput` | Ctrl+K T | Expand Cell Output |
| `notebook.cell.toggleOutputScrolling` | Ctrl+K Y | Toggle Scrollable Cell Output |
| `notebook.cell.pasteAbove` | Ctrl+Shift+V | Paste Cell Above |
| `notebook.cell.chat.start` | Ctrl+I | Start Cell inline chat |
| `notebook.cell.openFailureActions` | Ctrl+. | Open failed-cell Quick Fixes |
| `notebook.find` | Ctrl+F | Find in Notebook |
| `notebook.addFindMatchToSelection` | Ctrl+D | Add Find Match to Selection |
| `notebook.selectAllFindMatches` | Ctrl+Shift+L | Select All Find Matches |
| `notebook.centerActiveCell` | Ctrl+L | Center the active cell in the viewport |
| `notebook.focusTop` | Ctrl+Home | Focus First Cell |
| `notebook.focusBottom` | Ctrl+End | Focus Last Cell |
| `notebook.focusNextEditor` | Down | Focus Next Cell Editor |
| `notebook.focusPreviousEditor` | Up | Focus Previous Cell Editor |
| `notebook.fold` | Ctrl+Shift+[ | Fold Notebook Cell(s) |
| `notebook.unfold` | Ctrl+Shift+] | Unfold Notebook Cell(s) |
| `notebook.format` | Shift+Alt+F | Format Notebook |
| `notebook.commentSelectedCells` | Ctrl+/ | Comment Selected Cells |

### Comments

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.addComment` | Ctrl+K Ctrl+Alt+C | Add a Comment on the current line/range |
| `editor.action.submitComment` | Ctrl+Enter | Submit the comment being edited |
| `editor.action.nextCommentThreadAction` | Alt+F9 | Go to Next Comment Thread |
| `editor.action.previousCommentThreadAction` | Shift+Alt+F9 | Go to Previous Comment Thread |
| `editor.action.nextCommentedRangeAction` | Alt+F10 | Go to Next Commented Range |
| `editor.action.previousCommentedRangeAction` | Shift+Alt+F10 | Go to Previous Commented Range |
| `commentsFocusFilter` | Ctrl+F | Focus the Filter box in the Comments panel |
| `commentsClearFilterText` | Escape | Clear Comments panel filter text |
| `commentsFocusViewFromFilter` | Ctrl+Down | Move focus from filter into Comments list |
| `workbench.action.collapseAllComments` | | Collapse All Comment Threads |
| `workbench.action.expandAllComments` | | Expand All Comment Threads |
| `workbench.action.toggleCommentThread` | | Toggle a comment thread's expand state |
| `workbench.action.hideCommentThread` | | Hide/dispose a comment thread |
| `workbench.action.focusCommentsPanel` | | Focus the Comments panel |

### Window & Editor Group Management

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.splitEditor` | Ctrl+\ | Split Editor |
| `workbench.action.splitEditorDown` | Ctrl+K Ctrl+\ | Split Editor Down |
| `workbench.action.splitEditorOrthogonal` | | Split Editor Orthogonal (opposite of current layout) |
| `workbench.action.focusFirstEditorGroup` | Ctrl+1 | Focus First Editor Group |
| `workbench.action.focusSecondEditorGroup` | Ctrl+2 | Focus Second Editor Group |
| `workbench.action.focusThirdEditorGroup` | Ctrl+3 | Focus Third Editor Group |
| `workbench.action.focusFourthEditorGroup` | Ctrl+4 | Focus Fourth Editor Group |
| `workbench.action.focusFifthEditorGroup` | Ctrl+5 | Focus Fifth Editor Group |
| `workbench.action.focusSixthEditorGroup` | Ctrl+6 | Focus Sixth Editor Group |
| `workbench.action.focusSeventhEditorGroup` | Ctrl+7 | Focus Seventh Editor Group |
| `workbench.action.focusEighthEditorGroup` | Ctrl+8 | Focus Eighth Editor Group |
| `workbench.action.focusLeftGroup` | Ctrl+K Ctrl+Left | Focus Editor Group to the Left |
| `workbench.action.focusRightGroup` | Ctrl+K Ctrl+Right | Focus Editor Group to the Right |
| `workbench.action.focusAboveGroup` | Ctrl+K Ctrl+Up | Focus Editor Group Above |
| `workbench.action.focusBelowGroup` | Ctrl+K Ctrl+Down | Focus Editor Group Below |
| `workbench.action.focusSideBar` | Ctrl+0 | Focus into Side Bar |
| `workbench.action.focusNextPart` | F6 | Move Focus to Next Part of the workbench |
| `workbench.action.focusPreviousPart` | Shift+F6 | Move Focus to Previous Part |
| `workbench.action.moveEditorToNextGroup` | Ctrl+Alt+Right | Move Editor into Next Group |
| `workbench.action.moveEditorToPreviousGroup` | Ctrl+Alt+Left | Move Editor into Previous Group |
| `workbench.action.moveEditorLeftInGroup` | Ctrl+Shift+PageUp | Move Editor Left within Tab bar |
| `workbench.action.moveEditorRightInGroup` | Ctrl+Shift+PageDown | Move Editor Right within Tab bar |
| `workbench.action.moveEditorToFirstGroup` | Shift+Alt+1 | Move Editor into First Group |
| `workbench.action.moveEditorToLastGroup` | Shift+Alt+9 | Move Editor into Last Group |
| `workbench.action.moveActiveEditorGroupLeft` | | Move Active Editor Group Left |
| `workbench.action.moveActiveEditorGroupRight` | | Move Active Editor Group Right |
| `workbench.action.closeGroup` | Ctrl+W / Ctrl+F4 | Close the active editor group |
| `workbench.action.closeAllGroups` | Ctrl+K Ctrl+Shift+W | Close All Editor Groups |
| `workbench.action.toggleEditorGroupLayout` | Shift+Alt+0 | Toggle Vertical/Horizontal Editor Layout |
| `workbench.action.toggleMaximizeEditorGroup` | Ctrl+K Ctrl+M | Toggle Maximize Active Editor Group |
| `workbench.action.evenEditorWidths` | | Reset Editor Group Sizes to be even |
| `workbench.action.nextEditor` | Ctrl+PageDown | Open Next Editor |
| `workbench.action.previousEditor` | Ctrl+PageUp | Open Previous Editor |
| `workbench.action.nextEditorInGroup` | Ctrl+K Ctrl+PageDown | Open Next Editor in Group |
| `workbench.action.previousEditorInGroup` | Ctrl+K Ctrl+PageUp | Open Previous Editor in Group |

### Tabs

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.closeActiveEditor` | Ctrl+W | Close active tab |
| `workbench.action.closeOtherEditors` | | Close all other tabs in the group |
| `workbench.action.closeEditorsToTheRight` | | Close tabs to the right of this one |
| `workbench.action.closeAllEditors` | Ctrl+K Ctrl+W | Close All tabs |
| `workbench.action.closeUnmodifiedEditors` | Ctrl+K U | Close Unmodified tabs |
| `workbench.action.reopenClosedEditor` | Ctrl+Shift+T | Reopen a closed tab |
| `workbench.action.showAllEditors` | | Show All Editors by Most Recently Used |
| `workbench.action.showEditorsInActiveGroup` | | Show all tabs in the active group |
| `workbench.action.pinEditor` | Ctrl+K Shift+Enter | Pin Editor tab (keep open) |
| `workbench.action.unpinEditor` | Ctrl+K Shift+Enter | Unpin Editor tab |
| `workbench.action.keepEditor` | | Keep a preview editor tab permanently open |
| `workbench.action.quickOpenPreviousRecentlyUsedEditorInGroup` | Ctrl+Tab | Cycle through open tabs |
| `workbench.action.moveEditorLeftInGroup` | Ctrl+Shift+PageUp | Move Tab Left |
| `workbench.action.moveEditorRightInGroup` | Ctrl+Shift+PageDown | Move Tab Right |
| `workbench.action.openNextRecentlyUsedEditorInGroup` | | Open Next Recently Used Editor in current group |
| `workbench.action.openPreviousRecentlyUsedEditorInGroup` | | Open Previous Recently Used Editor in current group |

### Zen & Layout

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.toggleZenMode` | Ctrl+K Z | Toggle Zen Mode |
| `workbench.action.exitZenMode` | Escape Escape | Exit Zen Mode |
| `workbench.action.toggleFullScreen` | F11 | Toggle Full Screen |
| `workbench.action.toggleSidebarVisibility` | Ctrl+B | Toggle Primary Side Bar Visibility |
| `workbench.action.toggleAuxiliaryBar` | Ctrl+Alt+B | Toggle Secondary Side Bar |
| `workbench.action.togglePanel` | Ctrl+J | Toggle Panel Visibility |
| `workbench.action.toggleMaximizedPanel` | | Toggle Maximized Panel |
| `workbench.action.toggleStatusbarVisibility` | | Toggle Status Bar Visibility |
| `workbench.action.toggleActivityBarVisibility` | | Toggle Activity Bar Visibility |
| `workbench.action.toggleMenuBar` | | Toggle Menu Bar Visibility |
| `workbench.action.toggleTabsVisibility` | | Toggle Editor Tabs Visibility |
| `workbench.action.moveSideBarRight` | | Move Side Bar to the Right |
| `workbench.action.moveSideBarLeft` | | Move Side Bar to the Left |
| `workbench.action.zoomIn` | Ctrl+= | Zoom In |
| `workbench.action.zoomOut` | Ctrl+- | Zoom Out |
| `workbench.action.zoomReset` | Ctrl+Numpad0 | Reset Zoom |

### Tasks

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.tasks.runTask` | | Run Task... (task picker) |
| `workbench.action.tasks.build` | Ctrl+Shift+B | Run Build Task |
| `workbench.action.tasks.test` | | Run Test Task |
| `workbench.action.tasks.reRunTask` | | Rerun Last Task |
| `workbench.action.tasks.rerunForActiveTerminal` | Ctrl+Shift+R | Rerun the Task associated with the active terminal |
| `workbench.action.tasks.restartTask` | | Restart Running Task |
| `workbench.action.tasks.terminate` | | Terminate Task |
| `workbench.action.tasks.showTasks` | | Show Running Tasks |
| `workbench.action.tasks.showLog` | | Show Task Log |
| `workbench.action.tasks.configureTaskRunner` | | Configure Task |
| `workbench.action.tasks.configureDefaultBuildTask` | | Configure Default Build Task |

### Markdown Preview

| Command ID | Default hotkey | Comment |
|---|---|---|
| `markdown.showPreview` | | Open Markdown Preview |
| `markdown.showPreviewToSide` | Ctrl+K V | Open Markdown Preview to the Side |
| `markdown.showSource` | | Show Markdown Source |
| `markdown.togglePreview` | Ctrl+Shift+V | Toggle between editor and Preview |
| `markdown.showPreviewSecuritySelector` | | Change Markdown Preview Security Settings |
| `markdown.preview.toggleLock` | | Toggle Preview Locking to current editor |
| `markdown.preview.refresh` | | Refresh Markdown Preview |
| `markdown.findInPreview` | | Find in the rendered Preview |
| `markdown.copyImage` | | Copy Image from preview |

### Accessibility

| Command ID | Default hotkey | Comment |
|---|---|---|
| `editor.action.accessibilityHelp` | Alt+F1 | Open Accessibility Help dialog |
| `editor.action.accessibleView` | Alt+F2 | Open Accessible View (screen-reader friendly rendering) |
| `editor.action.accessibleViewNext` | Alt+] | Show Next in Accessible View |
| `editor.action.accessibleViewPrevious` | Alt+[ | Show Previous in Accessible View |
| `editor.action.accessibleViewNextCodeBlock` | Ctrl+Alt+PageDown | Accessible View: Next Code Block |
| `editor.action.accessibleViewPreviousCodeBlock` | Ctrl+Alt+PageUp | Accessible View: Previous Code Block |
| `editor.action.accessibleViewGoToSymbol` | Ctrl+Shift+O | Go to Symbol in Accessible View |
| `editor.action.accessibleViewDisableHint` | Alt+F6 | Disable the hint for opening Accessible View |
| `editor.action.accessibilityHelpConfigureKeybindings` | Alt+K | Configure Keybindings from Accessibility Help |
| `editor.action.accessibilityHelpConfigureAssignedKeybindings` | Alt+A | Configure Assigned Keybindings |
| `editor.action.accessibilityHelpOpenHelpLink` | Alt+H | Open Help Link from Accessibility Help |
| `editor.action.accessibleDiffViewer.next` | F7 | Go to Next Difference in Accessible Diff Viewer |
| `editor.action.accessibleDiffViewer.prev` | Shift+F7 | Go to Previous Difference in Accessible Diff Viewer |
| `cursorWordAccessibilityLeft` | Ctrl+Left | Move cursor a word left (screen-reader mode word logic) |
| `cursorWordAccessibilityRight` | Ctrl+Right | Move cursor a word right (screen-reader mode) |
| `workbench.action.toggleScreenReaderAccessibilityMode` | | Toggle Screen Reader Accessibility Mode |

### Settings & Keybindings Editor

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.action.openSettings` | Ctrl+, | Open Settings (UI editor) |
| `workbench.action.openSettingsJson` | | Open Settings (JSON) |
| `workbench.action.openGlobalSettings` | | Open User Settings |
| `workbench.action.openWorkspaceSettings` | | Open Workspace Settings |
| `workbench.action.openAccessibilitySettings` | | Open Accessibility Settings |
| `workbench.action.openGlobalKeybindings` | Ctrl+K Ctrl+S | Open Keyboard Shortcuts editor |
| `workbench.action.openGlobalKeybindingsFile` | | Open Keyboard Shortcuts (JSON) |
| `workbench.action.openDefaultKeybindingsFile` | | Open Default Keyboard Shortcuts (JSON), read-only reference |
| `workbench.action.selectTheme` | Ctrl+K Ctrl+T | Select Color Theme |
| `workbench.action.selectIconTheme` | | Select File Icon Theme |
| `workbench.action.configureLanguageBasedSettings` | | Configure Language Specific Settings... |
| `settings.action.search` | Ctrl+F | Focus Settings Search box |
| `settings.action.clearSearchResults` | Escape | Clear Settings Search Results |
| `settings.action.toggleAiSearch` | Ctrl+I | Toggle AI-assisted Settings Search |

### Extensions View

| Command ID | Default hotkey | Comment |
|---|---|---|
| `workbench.view.extensions` | Ctrl+Shift+X | Show Extensions view |
| `workbench.extensions.action.installExtension` | | Install a specific Extension by id |
| `workbench.extensions.action.showInstalledExtensions` | | Show Installed Extensions |
| `workbench.extensions.action.showEnabledExtensions` | | Show Enabled Extensions |
| `workbench.extensions.action.showDisabledExtensions` | | Show Disabled Extensions |
| `workbench.extensions.action.showRecommendedExtensions` | | Show Recommended Extensions |
| `workbench.extensions.action.showPopularExtensions` | | Show Popular Extensions |
| `workbench.extensions.action.showBuiltInExtensions` | | Show Built-in Extensions |
| `workbench.extensions.action.checkForUpdates` | | Check for Extension Updates |
| `workbench.extensions.action.updateAllExtensions` | | Update All Extensions |
| `workbench.extensions.action.disableAll` | | Disable All Installed Extensions |
| `workbench.extensions.action.enableAll` | | Enable All Extensions |
| `workbench.extensions.action.configureWorkspaceRecommendedExtensions` | | Configure Workspace Recommended Extensions |
| `workbench.extensions.action.openExtensionsFolder` | | Open Extensions Folder |

### IntelliSense & Suggestions

| Command ID | Default hotkey | Comment |
|---|---|---|
| `editor.action.triggerSuggest` | Ctrl+Space / Ctrl+I | Trigger Suggest (IntelliSense) |
| `editor.action.triggerParameterHints` | Ctrl+Shift+Space | Trigger Parameter Hints |
| `editor.action.inlineSuggest.commit` | Tab | Accept the current inline (ghost text) suggestion |
| `editor.action.inlineSuggest.showNext` | Alt+] | Show Next Inline Suggestion |
| `editor.action.inlineSuggest.showPrevious` | Alt+[ | Show Previous Inline Suggestion |
| `editor.action.inlineSuggest.hide` | Escape | Hide the current inline suggestion |
| `toggleExplainMode` | Ctrl+/ | Toggle Explain Mode in the suggest widget |
| `insertSnippet` | Tab | Insert a matching snippet completion |

## Google Chrome

*Command IDs and platform gating taken directly from the Chromium source
— `chrome/app/chrome_command_ids.h` and the default-accelerator table
`chrome/browser/ui/accelerator_table.cc` (github.com/chromium/chromium,
main branch), cross-checked against Google's official shortcut list and
computerhope.com where possible. Hotkeys are Chrome's Windows/Linux
defaults (`Ctrl` stands in for `EF_PLATFORM_ACCELERATOR`, which is `Cmd`
on macOS). Most `IDC_*` constants have no default keyboard binding at all
— reachable only via menu/mouse/context-menu — and are listed anyway
since the point is a full command inventory, not just the bound subset.
Purely structural submenu-container placeholder IDs are omitted, as
they're not real invokable commands. A handful of very new commands
(Glic, omnibox AI features) aren't yet documented on secondary sites, so
their descriptions are inferred from the identifier and surrounding
source comments.*

### Navigation

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_BACK` | Alt+Left | Go back one page in tab history |
| `IDC_FORWARD` | Alt+Right | Go forward one page in tab history |
| `IDC_RELOAD` | Ctrl+R, F5 | Reload the current page |
| `IDC_HOME` | Alt+Home | Navigate to the homepage |
| `IDC_OPEN_CURRENT_URL` | | Navigate to whatever's typed in the omnibox |
| `IDC_STOP` | | Stop the page from loading |
| `IDC_RELOAD_BYPASSING_CACHE` | Ctrl+Shift+R, Shift+F5, Ctrl+F5 | Hard reload, bypassing the cache |
| `IDC_RELOAD_CLEARING_CACHE` | | Reload after clearing the cache |

### Windows & Tabs

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_NEW_WINDOW` | Ctrl+N | Open a new browser window |
| `IDC_NEW_INCOGNITO_WINDOW` | Ctrl+Shift+N | Open a new Incognito window |
| `IDC_CLOSE_WINDOW` | Ctrl+Shift+W, Alt+F4 | Close the current window |
| `IDC_NEW_TAB` | Ctrl+T | Open a new tab |
| `IDC_CLOSE_TAB` | Ctrl+W, Ctrl+F4 | Close the current tab |
| `IDC_SELECT_NEXT_TAB` | Ctrl+PageDown | Switch to the next tab |
| `IDC_SELECT_PREVIOUS_TAB` | Ctrl+PageUp | Switch to the previous tab |
| `IDC_CYCLE_TO_NEXT_TAB` | Ctrl+Tab | Cycle forward through tabs |
| `IDC_CYCLE_TO_PREV_TAB` | Ctrl+Shift+Tab | Cycle backward through tabs |
| `IDC_SELECT_TAB_0` | Ctrl+1 | Jump to tab 1 |
| `IDC_SELECT_TAB_1` | Ctrl+2 | Jump to tab 2 |
| `IDC_SELECT_TAB_2` | Ctrl+3 | Jump to tab 3 |
| `IDC_SELECT_TAB_3` | Ctrl+4 | Jump to tab 4 |
| `IDC_SELECT_TAB_4` | Ctrl+5 | Jump to tab 5 |
| `IDC_SELECT_TAB_5` | Ctrl+6 | Jump to tab 6 |
| `IDC_SELECT_TAB_6` | Ctrl+7 | Jump to tab 7 |
| `IDC_SELECT_TAB_7` | Ctrl+8 | Jump to tab 8 |
| `IDC_SELECT_LAST_TAB` | Ctrl+9 | Jump to the last tab |
| `IDC_DUPLICATE_TAB` | | Duplicate the current tab |
| `IDC_RESTORE_TAB` | Ctrl+Shift+T | Reopen the most recently closed tab/window |
| `IDC_FULLSCREEN` | F11 | Toggle fullscreen |
| `IDC_EXIT` | | Quit the browser |
| `IDC_MOVE_TAB_NEXT` | Ctrl+Shift+PageDown | Move current tab one slot right |
| `IDC_MOVE_TAB_PREVIOUS` | Ctrl+Shift+PageUp | Move current tab one slot left |
| `IDC_MINIMIZE_WINDOW` | | Minimize the window |
| `IDC_MAXIMIZE_WINDOW` | | Maximize the window |
| `IDC_RESTORE_WINDOW` | | (Windows/Linux) Restore a minimized/maximized window |
| `IDC_OPEN_IN_PWA_WINDOW` | | Reopen the current page as an installed web app |
| `IDC_MOVE_TAB_TO_NEW_WINDOW` | | Move the current tab into its own new window |
| `IDC_NEW_SPLIT_TAB` | Alt+Shift+N | (Win/Linux) Split the tab into a side-by-side view |
| `IDC_TOGGLE_VERTICAL_TABS` | | Toggle the vertical tab-strip layout |
| `IDC_TOGGLE_VERTICAL_TABS_COLLAPSE` | Ctrl+Shift+L | Collapse/expand the vertical tab strip |

### Tab Groups

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_ADD_NEW_TAB_TO_GROUP` | Alt+Shift+C | Add a new tab to the current tab group |
| `IDC_CREATE_NEW_TAB_GROUP` | Alt+Shift+P | Group the current tab into a new tab group |
| `IDC_FOCUS_NEXT_TAB_GROUP` | Alt+Shift+X | Move focus to the next tab group |
| `IDC_FOCUS_PREV_TAB_GROUP` | Alt+Shift+Z | Move focus to the previous tab group |
| `IDC_CLOSE_TAB_GROUP` | Alt+Shift+W | Close every tab in the current group |
| `IDC_GROUP_UNGROUPED_TABS` | | Group all currently ungrouped tabs |
| `IDC_ADD_NEW_TAB_RECENT_GROUP` | | Add a new tab into a recently-used group |
| `IDC_UNFOCUS_TAB_GROUP` | | Move keyboard focus out of a tab group |

### Web App (PWA) Windows

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_COPY_URL` | | Copy the current page's URL |
| `IDC_OPEN_IN_CHROME` | | Open this web app page in a regular Chrome tab |
| `IDC_WEB_APP_SETTINGS` | | Open this web app's settings |

### Bookmarks & Reading List

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_BOOKMARK_THIS_TAB` | Ctrl+D | Bookmark the current tab |
| `IDC_BOOKMARK_ALL_TABS` | Ctrl+Shift+D | Bookmark all open tabs into a new folder |
| `IDC_SHOW_BOOKMARK_BAR` | Ctrl+Shift+B | Toggle the bookmarks bar |
| `IDC_SHOW_BOOKMARK_MANAGER` | Ctrl+Shift+O | Open the Bookmark Manager |
| `IDC_BOOKMARK_MANAGER` | | Open Bookmark Manager (context-menu variant) |
| `IDC_SHOW_BOOKMARK_SIDE_PANEL` | | Open Bookmarks in the side panel |
| `IDC_READING_LIST_MENU_ADD_TAB` | | Add the current tab to the Reading List |
| `IDC_READING_LIST_MENU_SHOW_UI` | | Open the Reading List |
| `IDC_SHOW_READING_MODE_SIDE_PANEL` | | Open Reading Mode in the side panel |
| `IDC_SHOW_READING_MODE_KEYBOARD` | Alt+Shift+R | Toggle Reading Mode |
| `IDC_CONTENT_CONTEXT_ADD_LINK_TO_READING_LIST` | | Add the right-clicked link to the Reading List |
| `IDC_CONTENT_CONTEXT_OPEN_IN_READING_MODE` | | Open the page in Reading Mode |
| `IDC_BOOKMARK_BAR_OPEN_ALL` | | Open every bookmark in this folder as tabs |
| `IDC_BOOKMARK_BAR_OPEN_ALL_NEW_WINDOW` | | Open every bookmark in this folder in a new window |
| `IDC_BOOKMARK_BAR_OPEN_ALL_INCOGNITO` | | Open every bookmark in this folder in Incognito |
| `IDC_BOOKMARK_BAR_OPEN_INCOGNITO` | | Open this one bookmark in Incognito |
| `IDC_BOOKMARK_BAR_OPEN_ALL_NEW_TAB_GROUP` | | Open every bookmark in this folder into a new tab group |
| `IDC_BOOKMARK_BAR_RENAME_FOLDER` | | Rename a bookmark folder |
| `IDC_BOOKMARK_BAR_EDIT` | | Edit a bookmark |
| `IDC_BOOKMARK_BAR_REMOVE` | | Delete a bookmark or folder |
| `IDC_BOOKMARK_BAR_UNDO` | | Undo the last bookmark change |
| `IDC_BOOKMARK_BAR_REDO` | | Redo the last undone bookmark change |
| `IDC_BOOKMARK_BAR_ADD_NEW_BOOKMARK` | | Add a new bookmark |
| `IDC_BOOKMARK_BAR_NEW_FOLDER` | | Create a new bookmark folder |
| `IDC_BOOKMARK_BAR_ALWAYS_SHOW` | | Toggle "Always show bookmarks bar" |
| `IDC_BOOKMARK_BAR_ADD_TO_BOOKMARKS_BAR` | | Add this bookmark to the bookmarks bar |
| `IDC_BOOKMARK_BAR_REMOVE_FROM_BOOKMARKS_BAR` | | Remove this bookmark from the bookmarks bar |
| `IDC_BOOKMARK_BAR_MOVE` | | Move a bookmark to another folder |

### Page Actions (Print, Save, Sharing, Site Controls)

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_VIEW_SOURCE` | Ctrl+U | View the page's HTML source |
| `IDC_PRINT` | Ctrl+P | Print the current page |
| `IDC_SAVE_PAGE` | Ctrl+S | Save the current page to disk |
| `IDC_BASIC_PRINT` | Ctrl+Shift+P | Open the system print dialog, bypassing print preview |
| `IDC_EMAIL_PAGE_LOCATION` | | Email a link to the current page |
| `IDC_SHOW_TRANSLATE` | | Translate the current page |
| `IDC_MANAGE_PASSWORDS_FOR_PAGE` | | Open the password bubble for this site |
| `IDC_ROUTE_MEDIA` | | Start Casting media from this page |
| `IDC_WINDOW_MUTE_SITE` | | Mute audio from the current tab's site |
| `IDC_WINDOW_PIN_TAB` | | Pin the current tab |
| `IDC_WINDOW_GROUP_TAB` | | Add the current tab to a tab group |
| `IDC_SEND_TAB_TO_SELF` | | Send the current tab to another signed-in device |
| `IDC_FOCUS_THIS_TAB` | | Move keyboard focus to the current tab |
| `IDC_QRCODE_GENERATOR` | | Generate a QR code for the current page |
| `IDC_WINDOW_CLOSE_TABS_TO_RIGHT` | | Close all tabs to the right of this one |
| `IDC_WINDOW_CLOSE_OTHER_TABS` | | Close every tab except this one |
| `IDC_NEW_TAB_TO_RIGHT` | | Open a new tab immediately to the right |
| `IDC_SHARING_HUB` | | Open the sharing hub for this page |
| `IDC_SHARING_HUB_SCREENSHOT` | | Take a screenshot via the sharing hub |
| `IDC_SHOW_PASSWORD_MANAGER` | | Open Chrome's Password Manager |
| `IDC_SHOW_PAYMENT_METHODS` | | Open saved payment methods |
| `IDC_SHOW_ADDRESSES` | | Open saved addresses |
| `IDC_ORGANIZE_TABS` | | Open AI-assisted tab organization |
| `IDC_TAKE_SCREENSHOT` | | Capture a screenshot of the page |
| `IDC_CONTENT_CONTEXT_GENERATE_QR_CODE` | | Generate a QR code for the link/page (context menu) |

### Zoom

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_ZOOM_PLUS` | Ctrl+= (also Ctrl+Shift+=) | Zoom in |
| `IDC_ZOOM_NORMAL` | Ctrl+0 | Reset zoom to 100% |
| `IDC_ZOOM_MINUS` | Ctrl+- (also Ctrl+Shift+-) | Zoom out |

### Find in Page

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_FIND` | Ctrl+F | Open the find-in-page bar |
| `IDC_FIND_NEXT` | Ctrl+G, F3 | Jump to the next match |
| `IDC_FIND_PREVIOUS` | Ctrl+Shift+G, Shift+F3 | Jump to the previous match |
| `IDC_CLOSE_FIND_OR_STOP` | Esc | Close the find bar, or stop page load if find isn't open |

### Clipboard

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_CUT` | Ctrl+X | Cut |
| `IDC_COPY` | Ctrl+C | Copy |
| `IDC_PASTE` | Ctrl+V | Paste |
| `IDC_PASTE_AND_GO` | | Paste clipboard text into the omnibox and navigate |

### History & Downloads

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_SHOW_HISTORY` | Ctrl+H | Open History |
| `IDC_SHOW_HISTORY_SIDE_PANEL` | | Open History in the side panel |
| `IDC_SHOW_HISTORY_CLUSTERS_SIDE_PANEL` | | Open History Journeys side panel |
| `IDC_SHOW_DOWNLOADS` | Ctrl+J | Open Downloads |
| `IDC_CLEAR_BROWSING_DATA` | Ctrl+Shift+Delete | Open "Clear browsing data" |
| `IDC_OPEN_RECENT_TAB` | | Reopen a specific recently-closed tab (dynamic menu item) |
| `IDC_RECENT_TABS_SEE_DEVICE_TABS` | | See tabs open on other signed-in devices |
| `IDC_SHOW_TABS_FROM_OTHER_DEVICES_SIDE_PANEL` | | Show other devices' open tabs in the side panel |

### DevTools

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_DEV_TOOLS_TOGGLE` | F12 | Toggle DevTools open/closed |
| `IDC_DEV_TOOLS` | Ctrl+Shift+I | Open DevTools |
| `IDC_DEV_TOOLS_CONSOLE` | Ctrl+Shift+J | Open DevTools to the JS console |
| `IDC_DEV_TOOLS_INSPECT` | Ctrl+Shift+C | Open DevTools in inspect-element mode |
| `IDC_CONTENT_CONTEXT_INSPECTELEMENT` | | Inspect the right-clicked element |
| `IDC_TASK_MANAGER` | | Open Chrome's Task Manager |
| `IDC_TASK_MANAGER_SHORTCUT` | Shift+Esc | Open Task Manager (keyboard variant) |

### Profiles & Sync

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_SHOW_AVATAR_MENU` | Ctrl+Shift+M | Open the profile/avatar menu |
| `IDC_CUSTOMIZE_CHROME` | | Open the "Customize Chrome" side panel |
| `IDC_CLOSE_PROFILE` | | Close all windows for the current profile |
| `IDC_MANAGE_GOOGLE_ACCOUNT` | | Open your Google Account page |
| `IDC_OPEN_GUEST_PROFILE` | | Open a new Guest window |
| `IDC_ADD_NEW_PROFILE` | | Add a new Chrome profile |
| `IDC_MANAGE_CHROME_PROFILES` | | Open profile-management settings |
| `IDC_SHOW_SIGNIN` | | Open the sign-in flow |

### Settings, Browser UI & Extensions

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_OPTIONS` | | Open Chrome Settings |
| `IDC_OPEN_FILE` | Ctrl+O | Open a local file in the browser |
| `IDC_CREATE_SHORTCUT` | | Create a desktop shortcut for the current page/app |
| `IDC_IMPORT_SETTINGS` | | Import bookmarks/settings from another browser |
| `IDC_EDIT_SEARCH_ENGINES` | | Open search-engine settings |
| `IDC_VIEW_PASSWORDS` | | Open the Password Manager |
| `IDC_ABOUT` | | Open the About/relaunch page |
| `IDC_HELP_PAGE_VIA_KEYBOARD` | F1 | Open Chrome help |
| `IDC_HELP_PAGE_VIA_MENU` | | Open Chrome help (via menu) |
| `IDC_SHOW_APP_MENU` | Alt+E, Alt+F | Open the Chrome (3-dot) menu |
| `IDC_MANAGE_EXTENSIONS` | | Open the Extensions management page |
| `IDC_UPGRADE_DIALOG` | | Show the "relaunch to update" dialog |
| `IDC_INSTALL_PWA` | | Install the current site as an app |
| `IDC_SHOW_MANAGEMENT_PAGE` | | Open chrome://management |
| `IDC_SHOW_FULL_URLS` | | Toggle showing full URLs in the address bar |
| `IDC_CHROME_WHATS_NEW` | | Open "What's New in Chrome" |
| `IDC_PERFORMANCE` | | Open Performance settings (memory/energy saver) |
| `IDC_EXTENSIONS_SUBMENU_MANAGE_EXTENSIONS` | | Manage extensions (toolbar submenu) |
| `IDC_EXTENSIONS_SUBMENU_VISIT_CHROME_WEB_STORE` | | Open the Chrome Web Store |
| `IDC_SHOW_CHROME_LABS` | | Open Chrome Labs (experimental features flask) |
| `IDC_OPEN_SAFETY_HUB` | | Open Safety Check / Safety Hub |
| `IDC_SHOW_GOOGLE_LENS_SHORTCUT` | | Open Google Lens search |
| `IDC_SHOW_CUSTOMIZE_CHROME_SIDE_PANEL` | | Open the Customize Chrome side panel |
| `IDC_SET_BROWSER_AS_DEFAULT` | | Set Chrome as the default browser |
| `IDC_FEEDBACK` | Alt+Shift+I | Report an issue / send feedback (branded builds) |
| `IDC_UPDATE_SIDE_PANEL_PIN_STATE` | | Pin/unpin a side-panel entry to the toolbar |
| `IDC_STATUS_TRAY_KEEP_CHROME_RUNNING_IN_BACKGROUND` | | Toggle "keep running background apps" |
| `IDC_OPEN_GLIC` | | Open Glic, Chrome's built-in AI assistant panel |
| `IDC_GLIC_TOGGLE_PIN` | | Pin/unpin the Glic button on the toolbar |
| `IDC_GLIC_STATUS_ICON_MENU_TOGGLE` | | Show/hide the Glic panel (system tray) |
| `IDC_TAB_SEARCH` | Ctrl+Shift+A | Open the tab-search dropdown |
| `IDC_TAB_SEARCH_CLOSE` | | Close the tab-search dropdown |
| `IDC_LIVE_CAPTION` | | Toggle Live Caption |
| `IDC_CHROME_ENTERPRISE_RELEASE_NOTES` | | Open Chrome Enterprise release notes |

### Accessibility

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_FOCUS_INACTIVE_POPUP_FOR_ACCESSIBILITY` | Alt+Shift+A | Move focus to an inactive popup/infobar for screen readers |
| `IDC_CARET_BROWSING_TOGGLE` | F7 | Toggle caret browsing |
| `IDC_CONTENT_CONTEXT_ACCESSIBILITY_LABELS_TOGGLE` | | Toggle "Get image descriptions" labels |
| `IDC_CONTENT_CONTEXT_LISTEN_TO_THIS_PAGE` | | Read this page aloud |

### Focus / Keyboard Navigation

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_FOCUS_TOOLBAR` | Alt+Shift+T | Move keyboard focus to the toolbar |
| `IDC_FOCUS_LOCATION` | Ctrl+L, Alt+D | Move keyboard focus to the address bar |
| `IDC_FOCUS_SEARCH` | Ctrl+K, Ctrl+E | Focus the address bar in search/keyword mode |
| `IDC_FOCUS_MENU_BAR` | F10, Alt | Move keyboard focus to the menu bar (Linux) |
| `IDC_FOCUS_NEXT_PANE` | F6 | Move focus to the next UI pane |
| `IDC_FOCUS_PREVIOUS_PANE` | Shift+F6 | Move focus to the previous UI pane |
| `IDC_FOCUS_BOOKMARKS` | Alt+Shift+B | Move keyboard focus to the bookmarks bar |
| `IDC_FOCUS_WEB_CONTENTS_PANE` | Ctrl+F6 | Move keyboard focus to the page content |

### Spell Check & Writing Direction

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_CHECK_SPELLING_WHILE_TYPING` | | Toggle "Check spelling while typing" |
| `IDC_SPELLCHECK_ADD_TO_DICTIONARY` | | Add the selected word to the custom dictionary |
| `IDC_SPELLCHECK_MULTI_LINGUAL` | | Toggle multilingual spellcheck |
| `IDC_SPELLCHECK_REMOVE_FROM_DICTIONARY` | | Remove the selected word from the custom dictionary |
| `IDC_CONTENT_CONTEXT_SPELLING_SUGGESTION` | | Apply the top spelling suggestion |
| `IDC_CONTENT_CONTEXT_SPELLING_TOGGLE` | | Toggle spell-check for this field |
| `IDC_WRITING_DIRECTION_LTR` | | Set editable text's direction to left-to-right |
| `IDC_WRITING_DIRECTION_RTL` | | Set editable text's direction to right-to-left |

### Autofill, Passwords & Payments

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_SAVE_CREDIT_CARD_FOR_PAGE` | | Show the "save this card" prompt |
| `IDC_SAVE_AUTOFILL_ADDRESS` | | Show the "save this address" prompt |
| `IDC_CONTENT_CONTEXT_GENERATEPASSWORD` | | Suggest a generated password for this field |
| `IDC_CONTENT_CONTEXT_SHOWALLSAVEDPASSWORDS` | | Show all saved passwords |
| `IDC_CONTENT_CONTEXT_USE_PASSKEY_FROM_ANOTHER_DEVICE` | | Use a passkey from another device |
| `IDC_CONTENT_CONTEXT_AUTOFILL_FEEDBACK` | | Send feedback about an Autofill suggestion |

### Sharing, QR Codes & Casting

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_MEDIA_ROUTER_ABOUT` | | About Google Cast |
| `IDC_MEDIA_ROUTER_TOGGLE_MEDIA_REMOTING` | | Toggle media remoting for a cast session |
| `IDC_MEDIA_TOOLBAR_CONTEXT_SHOW_OTHER_SESSIONS` | | Show other active Cast sessions |

### Context Menu — Links

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_CONTENT_CONTEXT_OPENLINKNEWTAB` | | Open link in a new tab |
| `IDC_CONTENT_CONTEXT_OPENLINKNEWWINDOW` | | Open link in a new window |
| `IDC_CONTENT_CONTEXT_OPENLINKOFFTHERECORD` | | Open link in a new Incognito window |
| `IDC_CONTENT_CONTEXT_OPENLINKSPLITVIEW` | | Open link in a new split-view tab |
| `IDC_CONTENT_CONTEXT_SAVELINKAS` | | Save the link target as a file |
| `IDC_CONTENT_CONTEXT_COPYLINKLOCATION` | | Copy the link URL |
| `IDC_CONTENT_CONTEXT_COPYLINKTEXT` | | Copy the link's visible text |
| `IDC_CONTENT_CONTEXT_COPYLINKTOTEXT` | | Copy a link that scrolls to and highlights this text |

### Context Menu — Images, Audio & Video

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_CONTENT_CONTEXT_SAVEIMAGEAS` | | Save image as a file |
| `IDC_CONTENT_CONTEXT_COPYIMAGELOCATION` | | Copy the image URL |
| `IDC_CONTENT_CONTEXT_COPYIMAGE` | | Copy the image to the clipboard |
| `IDC_CONTENT_CONTEXT_OPENIMAGENEWTAB` | | Open the image in a new tab |
| `IDC_CONTENT_CONTEXT_SEARCHWEBFORIMAGE` | | Search the web for this image |
| `IDC_CONTENT_CONTEXT_SEARCHLENSFORIMAGE` | | Search this image with Google Lens |
| `IDC_CONTENT_CONTEXT_SAVEVIDEOFRAMEAS` | | Save the current video frame as an image |
| `IDC_CONTENT_CONTEXT_SAVEAVAS` | | Save the audio/video file as |
| `IDC_CONTENT_CONTEXT_PICTUREINPICTURE` | | Open the video in Picture-in-Picture |

### Context Menu — Edit

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_CONTENT_CONTEXT_COPY` | | Copy selected content |
| `IDC_CONTENT_CONTEXT_CUT` | | Cut selected content |
| `IDC_CONTENT_CONTEXT_PASTE` | | Paste clipboard content |
| `IDC_CONTENT_CONTEXT_PASTE_AND_MATCH_STYLE` | | Paste without formatting |
| `IDC_CONTENT_CONTEXT_DELETE` | | Delete the selected content |
| `IDC_CONTENT_CONTEXT_UNDO` | | Undo the last edit |
| `IDC_CONTENT_CONTEXT_REDO` | | Redo the last undone edit |
| `IDC_CONTENT_CONTEXT_SELECTALL` | | Select all content |

### Context Menu — Page & Frame

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_CONTENT_CONTEXT_TRANSLATE` | | Translate the selected text/page |
| `IDC_CONTENT_CONTEXT_PARTIAL_TRANSLATE` | | Translate only the selected text |
| `IDC_CONTENT_CONTEXT_LANGUAGE_SETTINGS` | | Open language settings |
| `IDC_CONTENT_CONTEXT_LOOK_UP` | | Look up the selected word/phrase |
| `IDC_CONTENT_CONTEXT_RELOADFRAME` | | Reload just the clicked iframe |
| `IDC_CONTENT_CONTEXT_VIEWFRAMESOURCE` | | View the clicked iframe's source |
| `IDC_CONTENT_CONTEXT_SEARCHWEBFOR` | | Search the web for the selected text |
| `IDC_CONTENT_CONTEXT_SEARCHWEBFORNEWTAB` | | Search the web for the selected text in a new tab |
| `IDC_CONTENT_CONTEXT_LENS_OVERLAY` | | Open the Lens overlay search |
| `IDC_CONTENT_CONTEXT_EXIT_FULLSCREEN` | | Exit fullscreen (context menu) |
| `IDC_CONTENT_CONTEXT_EMOJI` | | Open the emoji picker |
| `IDC_CONTEXT_COMPOSE` | | Open Compose (AI writing help) for this field |
| `IDC_REPORT_UNSAFE_SITE` | | Report the current site as unsafe |

### Platform-Specific

| Command ID | Default hotkey | Comment |
|---|---|---|
| `IDC_HIDE_APP` | | (macOS only) Hide the Chrome application |

## Linux (GNOME)

*Schema key names, types, and default values were pulled directly from
the `main`/`master` branches of the upstream GNOME source repos
(`mutter`, `gnome-shell`, `gsettings-desktop-schemas`,
`gnome-settings-daemon`), corresponding to roughly the GNOME 48/49
development line. Most keys are stable across GNOME 42–49; a few are
recent additions and won't exist on older releases (`show-screenshot-ui`
/`show-screen-recording-ui` — GNOME 42+; `cancel-input-capture` — GNOME
46+; `output-luminance` — GNOME 47+). Distro packagers generally ship
these defaults unmodified; known exceptions are called out inline, but
treat any distro-specific claim as unverified unless flagged — confirm
on a live system with `gsettings get <schema> <key>` if precision
matters. Empty hotkey cells mean the upstream default array is `[]` (no
binding shipped out of the box).*

### Window Management — Basic Actions

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.desktop.wm.keybindings close` | Alt+F4 | Close the focused window |
| `org.gnome.desktop.wm.keybindings toggle-maximized` | Alt+F10 | Toggle maximize/restore |
| `org.gnome.desktop.wm.keybindings maximize` | Super+Up | Maximize window |
| `org.gnome.desktop.wm.keybindings unmaximize` | Super+Down or Alt+F5 | Restore (un-maximize) window |
| `org.gnome.desktop.wm.keybindings minimize` | Super+H | Minimize window |
| `org.gnome.desktop.wm.keybindings toggle-fullscreen` | | Toggle fullscreen mode |
| `org.gnome.desktop.wm.keybindings toggle-above` | | Toggle "always on top"; appears to duplicate `always-on-top` below |
| `org.gnome.desktop.wm.keybindings always-on-top` | | Toggle window always-on-top (newer key, same effect as `toggle-above`) |
| `org.gnome.desktop.wm.keybindings begin-move` | Alt+F7 | Start interactive move (keyboard-driven) |
| `org.gnome.desktop.wm.keybindings begin-resize` | Alt+F8 | Start interactive resize (keyboard-driven) |
| `org.gnome.desktop.wm.keybindings raise` | | Raise window above others |
| `org.gnome.desktop.wm.keybindings lower` | | Lower window below others |
| `org.gnome.desktop.wm.keybindings raise-or-lower` | | Raise if covered, else lower |
| `org.gnome.desktop.wm.keybindings maximize-vertically` | | Maximize only vertically |
| `org.gnome.desktop.wm.keybindings maximize-horizontally` | | Maximize only horizontally |
| `org.gnome.desktop.wm.keybindings toggle-on-all-workspaces` | | Pin window to all workspaces / unpin |
| `org.gnome.desktop.wm.keybindings activate-window-menu` | Alt+Space | Open the window's title-bar menu |
| `org.gnome.desktop.wm.keybindings show-desktop` | | Minimize all normal windows |
| `org.gnome.desktop.wm.keybindings panel-run-dialog` | Alt+F2 | Show the "run a command" prompt |

### Window Tiling & Snapping

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.mutter.keybindings toggle-tiled-left` | Super+Left | Tile window to left half of screen |
| `org.gnome.mutter.keybindings toggle-tiled-right` | Super+Right | Tile window to right half of screen |
| `org.gnome.desktop.wm.keybindings move-to-corner-nw` | | Snap window to top-left corner (quarter-tile) |
| `org.gnome.desktop.wm.keybindings move-to-corner-ne` | | Snap window to top-right corner |
| `org.gnome.desktop.wm.keybindings move-to-corner-sw` | | Snap window to bottom-left corner |
| `org.gnome.desktop.wm.keybindings move-to-corner-se` | | Snap window to bottom-right corner |
| `org.gnome.desktop.wm.keybindings move-to-side-n` | | Snap window to top half (full-width) |
| `org.gnome.desktop.wm.keybindings move-to-side-s` | | Snap window to bottom half |
| `org.gnome.desktop.wm.keybindings move-to-side-e` | | Snap window to right half |
| `org.gnome.desktop.wm.keybindings move-to-side-w` | | Snap window to left half |
| `org.gnome.desktop.wm.keybindings move-to-center` | | Center window on screen without resizing |

### Workspace Switching

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.desktop.wm.keybindings switch-to-workspace-{1..12}` | workspace 1: Super+Home; 2–12: none | Jump directly to numbered workspace; only `switch-to-workspace-1` ships a default |
| `org.gnome.desktop.wm.keybindings switch-to-workspace-left` | Super+Page_Up or Super+Alt+Left or Ctrl+Alt+Left | Switch to the workspace to the left |
| `org.gnome.desktop.wm.keybindings switch-to-workspace-right` | Super+Page_Down or Super+Alt+Right or Ctrl+Alt+Right | Switch to the workspace to the right |
| `org.gnome.desktop.wm.keybindings switch-to-workspace-up` | Ctrl+Alt+Up | Switch to the workspace above (vertical workspace layout) |
| `org.gnome.desktop.wm.keybindings switch-to-workspace-down` | Ctrl+Alt+Down | Switch to the workspace below |
| `org.gnome.desktop.wm.keybindings switch-to-workspace-last` | Super+End | Jump to the last workspace |

### Moving Windows Between Workspaces & Monitors

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.desktop.wm.keybindings move-to-workspace-{1..12}` | workspace 1: Super+Shift+Home; 2–12: none | Move focused window to numbered workspace |
| `org.gnome.desktop.wm.keybindings move-to-workspace-left` | Super+Shift+Page_Up or Super+Shift+Alt+Left or Ctrl+Shift+Alt+Left | Move window to workspace on the left |
| `org.gnome.desktop.wm.keybindings move-to-workspace-right` | Super+Shift+Page_Down or Super+Shift+Alt+Right or Ctrl+Shift+Alt+Right | Move window to workspace on the right |
| `org.gnome.desktop.wm.keybindings move-to-workspace-up` | Ctrl+Shift+Alt+Up | Move window one workspace up |
| `org.gnome.desktop.wm.keybindings move-to-workspace-down` | Ctrl+Shift+Alt+Down | Move window one workspace down |
| `org.gnome.desktop.wm.keybindings move-to-workspace-last` | Super+Shift+End | Move window to the last workspace |
| `org.gnome.desktop.wm.keybindings move-to-monitor-left` | Super+Shift+Left | Move window to the next monitor on the left |
| `org.gnome.desktop.wm.keybindings move-to-monitor-right` | Super+Shift+Right | Move window to the next monitor on the right |
| `org.gnome.desktop.wm.keybindings move-to-monitor-up` | Super+Shift+Up | Move window to the monitor above |
| `org.gnome.desktop.wm.keybindings move-to-monitor-down` | Super+Shift+Down | Move window to the monitor below |

### Application & Window Switching

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.desktop.wm.keybindings switch-applications` | Super+Tab or Alt+Tab | Switch between applications (Alt-Tab switcher) |
| `org.gnome.desktop.wm.keybindings switch-applications-backward` | Shift+Super+Tab or Shift+Alt+Tab | Reverse-direction app switch |
| `org.gnome.desktop.wm.keybindings switch-windows` | | Switch between individual windows (no default; distinct from `switch-applications`) |
| `org.gnome.desktop.wm.keybindings switch-windows-backward` | | Reverse-direction window switch |
| `org.gnome.desktop.wm.keybindings switch-group` | Super+` or Alt+` | Switch between windows of the *current* application only |
| `org.gnome.desktop.wm.keybindings switch-group-backward` | Shift+Super+` or Shift+Alt+` | Reverse-direction same-app window switch |
| `org.gnome.desktop.wm.keybindings switch-panels` | Ctrl+Alt+Tab | Switch between panels/system controls (a11y-style switcher, shows icons) |
| `org.gnome.desktop.wm.keybindings switch-panels-backward` | Shift+Ctrl+Alt+Tab | Reverse-direction panel switch |
| `org.gnome.desktop.wm.keybindings cycle-windows` | Alt+Escape | Switch windows directly, no popup switcher |
| `org.gnome.desktop.wm.keybindings cycle-windows-backward` | Shift+Alt+Escape | Reverse-direction, no popup |
| `org.gnome.desktop.wm.keybindings cycle-group` | Alt+F6 | Switch same-app windows directly, no popup |
| `org.gnome.desktop.wm.keybindings cycle-group-backward` | Shift+Alt+F6 | Reverse-direction, no popup |
| `org.gnome.desktop.wm.keybindings cycle-panels` | Ctrl+Alt+Escape | Switch panels directly, no popup |
| `org.gnome.desktop.wm.keybindings cycle-panels-backward` | Shift+Ctrl+Alt+Escape | Reverse-direction, no popup |

### GNOME Shell — Overview, Search & App Launching

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.mutter overlay-key` | Super (tap alone) | Single string modifier key (not an array) that toggles the Activities overview; this, not `toggle-overview`, is how Super opens the overview by default |
| `org.gnome.shell.keybindings toggle-overview` | | User-configurable *keybinding* for the overview toggle — empty by default since `overlay-key` already covers the plain Super tap |
| `org.gnome.shell.keybindings toggle-application-view` | Super+A | Open the "Show Applications" app grid |
| `org.gnome.shell.keybindings shift-overview-up` | Super+Alt+Up | Shift between overview states (windows ↔ app grid) |
| `org.gnome.shell.keybindings shift-overview-down` | Super+Alt+Down | Shift between overview states, opposite direction |
| `org.gnome.shell.keybindings switch-to-application-{1..9}` | Super+1 … Super+9 | Activate/launch the Nth favorite/running app on the dash |
| `org.gnome.shell.keybindings open-new-window-application-{1..9}` | Ctrl+Super+1 … Ctrl+Super+9 | Open a new window of the Nth dash app |

### Screenshots & Screen Recording

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.shell.keybindings show-screenshot-ui` | Print | Open the interactive screenshot tool (GNOME 42+) |
| `org.gnome.shell.keybindings screenshot-window` | Alt+Print | Screenshot the focused window directly (no UI) |
| `org.gnome.shell.keybindings screenshot` | Shift+Print | Screenshot the whole screen directly (no UI) |
| `org.gnome.shell.keybindings show-screen-recording-ui` | Ctrl+Shift+Alt+R | Open the interactive screencast tool (GNOME 42+; older versions used a `toggle-recording` key with the same default combo) |

### Notifications

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.shell.keybindings toggle-message-tray` | Super+V or Super+M | Open/close the notification list (Super+M is a legacy alias kept alongside Super+V) |
| `org.gnome.shell.keybindings toggle-quick-settings` | Super+S | Open/close the Quick Settings menu (Wi-Fi/volume/power panel) |
| `org.gnome.shell.keybindings focus-active-notification` | Super+N | Move keyboard focus to the currently showing notification banner |

### Mutter / Compositor & Session-Level

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.mutter.keybindings switch-monitor` | Super+P or XF86Display | Cycle display/monitor configuration (mirror, extend, single) |
| `org.gnome.mutter.keybindings rotate-monitor` | XF86RotateWindows | Rotate the built-in display (tablet/convertible hardware key) |
| `org.gnome.mutter.keybindings cancel-input-capture` | Super+Shift+Escape | Cancel an active remote-desktop input-capture session (GNOME 46+) |
| `org.gnome.mutter locate-pointer-key` | Ctrl_L (double-tap) | Single string modifier key, not an array — double-tapping it flashes a locator ring around the mouse pointer |
| `org.gnome.mutter.wayland.keybindings restore-shortcuts` | Super+Escape | Re-enable shortcuts after an app has grabbed them (e.g. a VM or remote-desktop session) |
| `org.gnome.mutter.wayland.keybindings switch-to-session-{1..12}` | Ctrl+Alt+F1 … Ctrl+Alt+F12 | Switch virtual terminal/session (X11 VT-switch equivalent); Wayland/X11-server-level, not app-facing |

### Accessibility

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.settings-daemon.plugins.media-keys magnifier` | Alt+Super+8 | Toggle the full-screen magnifier (zoom) |
| `org.gnome.settings-daemon.plugins.media-keys magnifier-zoom-in` | Alt+Super+= | Increase magnifier zoom level |
| `org.gnome.settings-daemon.plugins.media-keys magnifier-zoom-out` | Alt+Super+- | Decrease magnifier zoom level |
| `org.gnome.settings-daemon.plugins.media-keys screenreader` | Alt+Super+S | Toggle Orca screen reader |
| `org.gnome.settings-daemon.plugins.media-keys on-screen-keyboard` | | Toggle the on-screen keyboard |
| `org.gnome.settings-daemon.plugins.media-keys increase-text-size` | | Increase system-wide text size |
| `org.gnome.settings-daemon.plugins.media-keys decrease-text-size` | | Decrease system-wide text size |
| `org.gnome.settings-daemon.plugins.media-keys toggle-contrast` | | Toggle high-contrast theme |

*Note on the next three sections:* since roughly GNOME 42, most
`media-keys` actions are split into a user-editable key (shown/rebindable
in Settings, defaulting to `['']`/empty) and a hidden paired
**`<name>-static`** key that carries the actual out-of-the-box hardware
default and isn't user-visible. The rows below name the user-facing key
and show the effective default, which in practice comes from its
`-static` twin unless noted otherwise.

### Media Keys — Volume, Microphone & Playback

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.settings-daemon.plugins.media-keys volume-up` | XF86AudioRaiseVolume or Ctrl+XF86AudioRaiseVolume | Raise system volume |
| `org.gnome.settings-daemon.plugins.media-keys volume-down` | XF86AudioLowerVolume or Ctrl+XF86AudioLowerVolume | Lower system volume |
| `org.gnome.settings-daemon.plugins.media-keys volume-mute` | XF86AudioMute | Mute/unmute system volume |
| `org.gnome.settings-daemon.plugins.media-keys volume-up-quiet` | Alt+XF86AudioRaiseVolume or Alt+Ctrl+XF86AudioRaiseVolume | Raise volume in smaller/quiet increments |
| `org.gnome.settings-daemon.plugins.media-keys volume-down-quiet` | Alt+XF86AudioLowerVolume or Alt+Ctrl+XF86AudioLowerVolume | Lower volume in smaller/quiet increments |
| `org.gnome.settings-daemon.plugins.media-keys volume-mute-quiet` | Alt+XF86AudioMute | Quiet-variant mute toggle |
| `org.gnome.settings-daemon.plugins.media-keys volume-up-precise` | Shift+XF86AudioRaiseVolume or Ctrl+Shift+XF86AudioRaiseVolume | Raise volume in fine (1%) increments |
| `org.gnome.settings-daemon.plugins.media-keys volume-down-precise` | Shift+XF86AudioLowerVolume or Ctrl+Shift+XF86AudioLowerVolume | Lower volume in fine (1%) increments |
| `org.gnome.settings-daemon.plugins.media-keys mic-mute` | XF86AudioMicMute | Mute/unmute microphone |
| `org.gnome.settings-daemon.plugins.media-keys play` | XF86AudioPlay or Ctrl+XF86AudioPlay | Play / play-pause current track |
| `org.gnome.settings-daemon.plugins.media-keys pause` | XF86AudioPause | Pause playback |
| `org.gnome.settings-daemon.plugins.media-keys stop` | XF86AudioStop | Stop playback |
| `org.gnome.settings-daemon.plugins.media-keys next` | XF86AudioNext or Ctrl+XF86AudioNext | Next track |
| `org.gnome.settings-daemon.plugins.media-keys previous` | XF86AudioPrev or Ctrl+XF86AudioPrev | Previous track |
| `org.gnome.settings-daemon.plugins.media-keys playback-rewind` | XF86AudioRewind | Skip backward within current track |
| `org.gnome.settings-daemon.plugins.media-keys playback-forward` | XF86AudioForward | Skip forward within current track |
| `org.gnome.settings-daemon.plugins.media-keys playback-repeat` | XF86AudioRepeat | Toggle repeat playback mode |
| `org.gnome.settings-daemon.plugins.media-keys playback-random` | XF86AudioRandomPlay | Toggle shuffle/random playback mode |

### Media Keys — Screen & Keyboard Brightness

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.shell.keybindings screen-brightness-up` | XF86MonBrightnessUp | Increase screen brightness (also mirrored in the `media-keys` schema for backward compatibility; GNOME Shell's copy is authoritative as of GNOME 42+) |
| `org.gnome.shell.keybindings screen-brightness-up-monitor` | Shift+XF86MonBrightnessUp | Increase brightness of only the currently-focused monitor (multi-monitor) |
| `org.gnome.shell.keybindings screen-brightness-down` | XF86MonBrightnessDown | Decrease screen brightness |
| `org.gnome.shell.keybindings screen-brightness-down-monitor` | Shift+XF86MonBrightnessDown | Decrease brightness of only the focused monitor |
| `org.gnome.shell.keybindings screen-brightness-cycle` | XF86MonBrightnessCycle | Cycle through brightness presets |
| `org.gnome.shell.keybindings screen-brightness-cycle-monitor` | Shift+XF86MonBrightnessCycle | Cycle brightness presets on the focused monitor only |
| `org.gnome.settings-daemon.plugins.media-keys keyboard-brightness-up` | XF86KbdBrightnessUp | Increase keyboard backlight brightness |
| `org.gnome.settings-daemon.plugins.media-keys keyboard-brightness-down` | XF86KbdBrightnessDown | Decrease keyboard backlight brightness |
| `org.gnome.settings-daemon.plugins.media-keys keyboard-brightness-toggle` | XF86KbdLightOnOff | Toggle keyboard backlight on/off |

### Launch Applications

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.settings-daemon.plugins.media-keys calculator` | XF86Calculator | Launch the calculator app |
| `org.gnome.settings-daemon.plugins.media-keys control-center` | XF86Tools | Launch Settings (control center) |
| `org.gnome.settings-daemon.plugins.media-keys email` | XF86Mail | Launch the default email client |
| `org.gnome.settings-daemon.plugins.media-keys eject` | XF86Eject | Eject removable media |
| `org.gnome.settings-daemon.plugins.media-keys help` | Super+F1 | Launch the help browser (default lives directly on the plain key, not a `-static` twin) |
| `org.gnome.settings-daemon.plugins.media-keys home` | XF86Explorer | Open the file manager at the home folder |
| `org.gnome.settings-daemon.plugins.media-keys media` | XF86AudioMedia | Launch the default media player |
| `org.gnome.settings-daemon.plugins.media-keys search` | XF86Search | Launch the default search app |
| `org.gnome.settings-daemon.plugins.media-keys www` | XF86WWW | Launch the default web browser |
| `org.gnome.settings-daemon.plugins.media-keys touchpad-toggle` | XF86TouchpadToggle or Ctrl+Super+XF86TouchpadToggle | Toggle touchpad on/off |
| `org.gnome.settings-daemon.plugins.media-keys touchpad-on` | XF86TouchpadOn | Switch touchpad on |
| `org.gnome.settings-daemon.plugins.media-keys touchpad-off` | XF86TouchpadOff | Switch touchpad off |

### Power, Lock & Session

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.settings-daemon.plugins.media-keys screensaver` | Super+L or XF86ScreenSaver | Lock the screen (both a real key default and a `-static` hardware-key twin are active simultaneously here) |
| `org.gnome.settings-daemon.plugins.media-keys logout` | Ctrl+Alt+Delete | Log out of the session |
| `org.gnome.settings-daemon.plugins.media-keys reboot` | | Reboot the machine (no default binding) |
| `org.gnome.settings-daemon.plugins.media-keys shutdown` | | Shut down the machine (no default binding) |
| `org.gnome.settings-daemon.plugins.media-keys power` | XF86PowerOff | Trigger the power button action (usually shows the power-off dialog) |
| `org.gnome.settings-daemon.plugins.media-keys suspend` | XF86Sleep | Suspend the machine |
| `org.gnome.settings-daemon.plugins.media-keys hibernate` | XF86Suspend or XF86Hibernate | Hibernate the machine (hardware "sleep" keys often map here too) |

### Input Source Switching

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.desktop.wm.keybindings switch-input-source` | Super+Space or XF86Keyboard | Switch to the next keyboard layout/input method |
| `org.gnome.desktop.wm.keybindings switch-input-source-backward` | Shift+Super+Space or Shift+XF86Keyboard | Switch to the previous keyboard layout/input method |

### Hardware & Misc Toggles

| Schema key | Default hotkey | Comment |
|---|---|---|
| `org.gnome.settings-daemon.plugins.media-keys rotate-video-lock` | Super+O or XF86RotationLockToggle | Toggle automatic screen-orientation rotation (tablets/convertibles) |
| `org.gnome.settings-daemon.plugins.media-keys battery-status` | XF86Battery | Show battery status |
| `org.gnome.settings-daemon.plugins.media-keys rfkill` | XF86WLAN or XF86UWB or XF86RFKill | Toggle all radios (airplane mode) |
| `org.gnome.settings-daemon.plugins.media-keys rfkill-bluetooth` | XF86Bluetooth | Toggle Bluetooth radio only |

## Neovim

*Citing Neovim v0.11.7 built-in ("vanilla") defaults — verified against
`runtime/doc/index.txt` (the source for `:h ex-cmd-index` and `:h
index.txt`) and `runtime/lua/vim/_defaults.lua` in the neovim/neovim
source tree. Commands marked **[Ex-only]** have no direct normal-mode key
and must be typed on the `:` command line; everything else is a
normal-mode key (or key sequence) with, where one exists, its Ex-command
equivalent noted in the comment.*

### Buffers

| Command name | Default hotkey | Comment |
|---|---|---|
| `:buffer` | `:b` | Go to buffer by number or name **[Ex-only]** |
| `:bnext` | `:bn` | Go to next buffer in the buffer list **[Ex-only]** |
| `:bprevious` | `:bp` | Go to previous buffer in the buffer list **[Ex-only]** |
| `:bNext` | `:bN` | Same as `:bprevious` **[Ex-only]** |
| `:bfirst` | `:bf` | Go to first buffer in the list **[Ex-only]** |
| `:blast` | `:bl` | Go to last buffer in the list **[Ex-only]** |
| `:brewind` | `:br` | Go to first buffer in the list **[Ex-only]** |
| `:bmodified` | `:bm` | Go to next modified buffer **[Ex-only]** |
| `:buffers` | `:ls` | List all buffers **[Ex-only]** |
| `:badd` | `:bad` | Add a buffer to the buffer list without loading it **[Ex-only]** |
| `:balt` | `:balt` | Like `:badd` but also sets the alternate file **[Ex-only]** |
| `:ball` | `:ba` | Open a window for every buffer in the list **[Ex-only]** |
| `:sball` | `:sba` | Split window for every buffer in the list **[Ex-only]** |
| `:bunload` | `:bun` | Unload a buffer, keep it in the buffer list **[Ex-only]** |
| `:bdelete` | `:bd` | Remove a buffer from the buffer list **[Ex-only]** |
| `:bwipeout` | `:bw` | Completely delete a buffer and its info **[Ex-only]** |
| `:bufdo` | `:bufdo` | Execute a command in every listed buffer **[Ex-only]** |
| `:enew` | `:ene` | Edit a new, unnamed buffer **[Ex-only]** |
| Go to next buffer | `]b` | vim-unimpaired-style next buffer (built-in default) |
| Go to previous buffer | `[b` | vim-unimpaired-style previous buffer (built-in default) |
| Go to first buffer | `[B` | Built-in default; `:brewind` unless a count is given |
| Go to last buffer | `]B` | Built-in default; `:blast` unless a count is given |
| Edit alternate file | `Ctrl-^` | Switch to the alternate (`#`) buffer, `N` selects alternate file N |

### Windows / Splits

| Command name | Default hotkey | Comment |
|---|---|---|
| `:split` | `:sp` | Split window horizontally **[Ex-only]** |
| `:vsplit` | `:vs` | Split window vertically **[Ex-only]** |
| `:new` | `:new` | Create a new empty window (horizontal) **[Ex-only]** |
| `:vnew` | `:vne` | Create a new empty window (vertical) **[Ex-only]** |
| `:close` | `:clo` | Close the current window **[Ex-only]** |
| `:hide` | `:hid` | Hide the current window/buffer **[Ex-only]** |
| `:only` | `:on` | Close all windows except the current one **[Ex-only]** |
| `:quit` | `:q` | Quit the current window **[Ex-only]** |
| `:resize` | `:res` | Change current window height **[Ex-only]** |
| `:wincmd` | `:winc` | Execute a `Ctrl-W` window command by letter **[Ex-only]** |
| Split window | `Ctrl-W s` | Same as `:split` |
| Split window vertically | `Ctrl-W v` | Same as `:vsplit` |
| Close window | `Ctrl-W c` | Same as `:close` |
| Close other windows | `Ctrl-W o` | Same as `:only` |
| Quit window | `Ctrl-W q` | Same as `:quit` |
| Go to next window | `Ctrl-W w` | Cycle to next window, wraps around |
| Go to previous window | `Ctrl-W W` | Cycle to previous window, wraps around |
| Go to last-accessed window | `Ctrl-W p` | Jump to the previously focused window |
| Go to window left/down/up/right | `Ctrl-W h/j/k/l` | Directional window navigation |
| Go to top window | `Ctrl-W t` | Move to topmost window |
| Go to bottom window | `Ctrl-W b` | Move to bottommost window |
| Move window far left/bottom/top/right | `Ctrl-W H/J/K/L` | Relocate current window to a screen edge |
| Move window to new tab page | `Ctrl-W T` | Break window out into its own tab |
| Rotate windows downwards | `Ctrl-W r` | Rotate window layout |
| Rotate windows upwards | `Ctrl-W R` | Rotate window layout, reverse |
| Exchange windows | `Ctrl-W x` | Swap current window with another |
| Equalize window sizes | `Ctrl-W =` | Make all windows equal height/width |
| Increase/decrease window height | `Ctrl-W +` / `Ctrl-W -` | Resize by N lines |
| Increase/decrease window width | `Ctrl-W >` / `Ctrl-W <` | Resize by N columns |
| Set window height | `Ctrl-W _` | Max height (or N) |
| Set window width | `Ctrl-W \|` | Max width (or N) |
| Open new window | `Ctrl-W n` | New window, N lines high |
| Close preview window | `Ctrl-W z` | Same as `:pclose` |
| Go to preview window | `Ctrl-W P` | Jump into the preview window |
| Split and edit file under cursor | `Ctrl-W f` | Opens `<cfile>` in a new split |
| Split, jump to tag under cursor | `Ctrl-W ]` | Split window and follow tag |
| Split and edit alternate file | `Ctrl-W ^` | Split with alternate buffer N |
| Show diagnostic under cursor | `Ctrl-W d` | Built-in default (see LSP section) |

### Tabs

| Command name | Default hotkey | Comment |
|---|---|---|
| `:tabnew` / `:tabedit` | `:tabnew` / `:tabe` | Open a file in a new tab page **[Ex-only]** |
| `:tabclose` | `:tabc` | Close the current tab page **[Ex-only]** |
| `:tabonly` | `:tabo` | Close all tab pages except the current one **[Ex-only]** |
| `:tabnext` | `:tabn` | Go to next tab page **[Ex-only]** |
| `:tabprevious` | `:tabp` | Go to previous tab page **[Ex-only]** |
| `:tabNext` | `:tabN` | Same as `:tabprevious` **[Ex-only]** |
| `:tabfirst` | `:tabfir` | Go to first tab page **[Ex-only]** |
| `:tablast` | `:tabl` | Go to last tab page **[Ex-only]** |
| `:tabmove` | `:tabm` | Move tab page to another position **[Ex-only]** |
| `:tabs` | `:tabs` | List tab pages and their windows **[Ex-only]** |
| `:tabdo` | `:tabd` | Execute a command in every tab page **[Ex-only]** |
| `:tabfind` | `:tabf` | Find file in `'path'`, edit in a new tab **[Ex-only]** |
| Go to next tab page | `gt` | Also `{count}gt` for a specific tab |
| Go to previous tab page | `gT` | Also `{count}gT` |
| Go to last-accessed tab page | `g<Tab>` | Also `Ctrl-W g<Tab>` / `Ctrl-<Tab>` |

### File / Session (write, read, edit, source)

| Command name | Default hotkey | Comment |
|---|---|---|
| `:write` | `:w` | Write buffer to file **[Ex-only]** |
| `:write!` | `:w!` | Force write, overriding checks **[Ex-only]** |
| `:wall` | `:wa` | Write all changed buffers **[Ex-only]** |
| `:wq` | `:wq` | Write and quit window/Vim **[Ex-only]** |
| `:wqall` | `:wqa` | Write all buffers and quit Vim **[Ex-only]** |
| `:xall` | `:xa` | Same as `:wqall` **[Ex-only]** |
| `:xit` | `:x` | Write if modified, then close window **[Ex-only]** |
| `:update` | `:up` | Write buffer only if modified **[Ex-only]** |
| `:edit` | `:e` | Edit a file **[Ex-only]** |
| `:edit!` | `:e!` | Re-edit file, discarding changes **[Ex-only]** |
| `:read` | `:r` | Read a file into the current buffer **[Ex-only]** |
| `:saveas` | `:sav` | Save the buffer under a new name **[Ex-only]** |
| `:source` | `:so` | Read and execute Vimscript/Lua from a file **[Ex-only]** |
| `:browse` | `:bro` | Use a file-selection dialog for the next command **[Ex-only]** |
| `:recover` | `:rec` | Recover a file from its swap file **[Ex-only]** |
| `:preserve` | `:pre` | Force-write the swap file **[Ex-only]** |
| `:mksession` | `:mks` | Write current session to a file **[Ex-only]** |
| `:mkview` | `:mkvie` | Save the view of the current window **[Ex-only]** |
| `:loadview` | `:lo` | Load a previously saved view **[Ex-only]** |
| `:checktime` | `:checkt` | Check if any buffer changed on disk **[Ex-only]** |
| `:cd` | `:cd` | Change the global working directory **[Ex-only]** |
| `:lcd` | `:lc` | Change working directory for current window only **[Ex-only]** |
| `:tcd` | `:tc` | Change working directory for current tab page only **[Ex-only]** |
| `:pwd` | `:pw` | Print current working directory **[Ex-only]** |
| `:find` | `:fin` | Find a file on `'path'` and edit it **[Ex-only]** |
| `:args` | `:ar` | Print/list the argument list **[Ex-only]** |
| `:argadd` | `:arga` | Add files to the argument list **[Ex-only]** |
| `:argdelete` | `:argd` | Remove files from the argument list **[Ex-only]** |
| `:next` | `:n` | Go to next file in argument list **[Ex-only]** |
| `:previous` | `:prev` | Go to previous file in argument list **[Ex-only]** |
| `:first` | `:fir` | Go to first file in argument list **[Ex-only]** |
| `:last` | `:la` | Go to last file in argument list **[Ex-only]** |
| `:argdo` | `:argdo` | Execute a command on every argument-list file **[Ex-only]** |
| `:oldfiles` | `:ol` | List files with marks in the ShaDa file **[Ex-only]** |
| `:checkhealth` | `:che` | Run built-in healthchecks **[Ex-only]** |

### Editing — Delete, Yank & Put

| Command name | Default hotkey | Comment |
|---|---|---|
| Delete character | `x` | Delete N characters under/after cursor |
| Delete character before cursor | `X` | Delete N characters before cursor |
| Delete `{motion}` | `d{motion}` | Delete text covered by a motion into a register |
| Delete N lines | `dd` | Delete N whole lines |
| Delete to end of line | `D` | Synonym for `d$` |
| Yank `{motion}` | `y{motion}` | Yank (copy) text covered by a motion |
| Yank N lines | `yy` | Also `Y` (default mapped to `y$`... actually `yy`) |
| Put after cursor | `p` | Put register contents after cursor/line |
| Put before cursor | `P` | Put register contents before cursor/line |
| Put after, leave cursor after | `gp` | Like `p` but cursor moves past the new text |
| Put before, leave cursor after | `gP` | Like `P` but cursor moves past the new text |
| Put, adjusting indent | `]p` / `[p` | Like `p`/`P` but re-indents to current line |
| Put block without trailing spaces | `zp` / `zP` | Blockwise paste, strips trailing whitespace |
| Yank without trailing spaces | `zy` | Blockwise yank, strips trailing whitespace |
| Replace char(s) | `r{char}` | Replace N characters with `{char}` |
| Substitute character | `s` | Delete N chars and start Insert (`c` + `l`) |
| Substitute line(s) | `S` | Delete N lines and start Insert (synonym for `cc`) |
| `:delete` | `:d` | Delete lines by range **[Ex-only]** |
| `:yank` | `:y` | Yank lines by range into a register **[Ex-only]** |
| `:put` | `:pu` | Insert register contents as new lines **[Ex-only]** |
| `:copy` | `:co` (also `:t`) | Copy lines to another location **[Ex-only]** |
| `:move` | `:m` | Move lines to another location **[Ex-only]** |

### Editing — Change, Undo, Redo, Repeat

| Command name | Default hotkey | Comment |
|---|---|---|
| Change `{motion}` | `c{motion}` | Delete text covered by motion, start Insert |
| Change N lines | `cc` | Delete N lines, start Insert |
| Change to end of line | `C` | Synonym for `c$` |
| Undo | `u` | Undo latest changes |
| Undo line | `U` | Undo all latest changes on one line |
| Redo | `Ctrl-R` | Redo changes undone with `u` |
| Repeat last change | `.` | Repeat last change, count can be replaced |
| Repeat last `:substitute` | `&` | Also `:&&` default (keeps flags) |
| Repeat last `:substitute`, all lines | `g&` | Repeat `:s` on every line |
| Go to older text state | `g-` | Step backward through undo tree by time |
| Go to newer text state | `g+` | Step forward through undo tree by time |
| `:undo` | `:u` | Undo N changes **[Ex-only]** |
| `:redo` | `:red` | Redo one undone change **[Ex-only]** |
| `:earlier` | `:ea` | Go to an older text state **[Ex-only]** |
| `:later` | `:lat` | Go to a newer text state **[Ex-only]** |
| `:undolist` | `:undol` | List leaves of the undo tree **[Ex-only]** |
| `:undojoin` | `:undoj` | Join next change into previous undo block **[Ex-only]** |

### Editing — Join, Indent, Case, Format

| Command name | Default hotkey | Comment |
|---|---|---|
| Join lines | `J` | Join N lines with a space, default 2 |
| Join lines without space | `gJ` | Join without inserting a space |
| Shift left | `<{motion}` / `<<` | Shift lines one `'shiftwidth'` left |
| Shift right | `>{motion}` / `>>` | Shift lines one `'shiftwidth'` right |
| Filter through `indent` | `={motion}` / `==` | Auto-indent (`'equalprg'`/`'indentexpr'`) lines |
| Format text | `gq{motion}` | Format lines to `'textwidth'` |
| Format, keep cursor | `gw{motion}` | Like `gq` but cursor doesn't move |
| Swap case | `~` | Toggle case of N chars (or `{motion}` with `'tildeop'`) |
| Swap case, operator | `g~{motion}` | Swap case of a motion's text |
| Make uppercase | `gU{motion}` | Uppercase a motion's text |
| Make lowercase | `gu{motion}` | Lowercase a motion's text |
| Rot13 encode | `g?{motion}` | Rot13-encode a motion's text |
| Increment number | `Ctrl-A` | Add N to number at/after cursor |
| Decrement number | `Ctrl-X` | Subtract N from number at/after cursor |
| Toggle comment | `gc{motion}` / `gcc` | Built-in commenting (0.10+ default) |
| Comment text object | `gc` (operator-pending) | Select the surrounding comment block |
| Filter through external command | `!{motion}` / `!!` | Pipe lines through a shell filter |
| `:join` | `:j` | Join lines by range **[Ex-only]** |
| `:left` / `:right` / `:center` | `:le` / `:ri` / `:ce` | Align lines **[Ex-only]** |
| `:retab` | `:ret` | Convert tabs/spaces per `'tabstop'` **[Ex-only]** |
| `:sort` | `:sor` | Sort lines **[Ex-only]** |
| `:normal` | `:norm` | Execute Normal-mode commands as an Ex command **[Ex-only]** |
| `:global` | `:g` | Execute a command on lines matching a pattern **[Ex-only]** |
| `:vglobal` | `:v` | Execute a command on lines *not* matching a pattern **[Ex-only]** |
| `:substitute` | `:s` | Find and replace text **[Ex-only]** |

### Insert Mode

| Command name | Default hotkey | Comment |
|---|---|---|
| Insert before cursor | `i` | Enter Insert mode before the cursor |
| Insert at start of line | `I` | Insert before first non-blank char |
| Append after cursor | `a` | Enter Insert mode after the cursor |
| Append at end of line | `A` | Insert at end of line |
| Open line below | `o` | New line below, enter Insert |
| Open line above | `O` | New line above, enter Insert |
| Insert at `'^` mark | `gi` | Like `i`, resumes at last insert position |
| Insert always in column 1 | `gI` | Like `I` but ignores indent |
| Replace mode | `R` | Overtype existing characters |
| Virtual Replace mode | `gR` | Replace mode that accounts for tabs/width |
| Execute one command, return to Insert | `Ctrl-O` | "Insert normal" mode for one command |
| Insert register contents | `Ctrl-R {reg}` | Insert a register as if typed |
| Insert char literally | `Ctrl-V {char}` | Bypass mapping/special meaning |
| Delete word before cursor | `Ctrl-W` | Delete the preceding word |
| Delete all entered chars on line | `Ctrl-U` | Clear back to start of insert on this line |
| Insert one shiftwidth of indent | `Ctrl-T` | Indent current line |
| Delete one shiftwidth of indent | `Ctrl-D` | Un-indent current line |
| Complete previous match | `Ctrl-P` | Keyword completion, previous match |
| Complete next match | `Ctrl-N` | Keyword completion, next match |
| Omni completion | `Ctrl-X Ctrl-O` | Language/plugin-aware completion |
| File name completion | `Ctrl-X Ctrl-F` | Complete file paths |
| Line completion | `Ctrl-X Ctrl-L` | Complete whole lines |
| Tag completion | `Ctrl-X Ctrl-]` | Complete from the tags file |
| Enter digraph | `Ctrl-K {c1}{c2}` | Insert a digraph character |
| End Insert mode | `<Esc>` | Also `Ctrl-C` (skips abbreviation expansion) |
| `:startinsert` | `:star` | Start Insert mode **[Ex-only]** |
| `:stopinsert` | `:stopi` | Stop Insert mode **[Ex-only]** |

### Motion / Navigation — Character, Line, Word

| Command name | Default hotkey | Comment |
|---|---|---|
| Left/down/up/right | `h` / `j` / `k` / `l` | Basic cursor motion, N times |
| Start of line | `0` | First column |
| First non-blank | `^` | First non-blank character on the line |
| End of line | `$` | End of Nth next line |
| Last non-blank | `g_` | Last non-blank char, N-1 lines lower |
| First non-blank of line N | `_` | First CHAR, N-1 lines lower |
| Go to line N | `gg` | Default: first line |
| Go to line N (default last) | `G` | Default: last line |
| Go to N percent | `{count}%` | Go to N percentage into the file |
| Top of window | `H` | Cursor to line N from top of screen |
| Middle of window | `M` | Cursor to middle line of screen |
| Bottom of window | `L` | Cursor to line N from bottom of screen |
| Word forward | `w` | Cursor N words forward |
| WORD forward | `W` | Cursor N WORDS forward (whitespace-delimited) |
| Word backward | `b` | Cursor N words backward |
| WORD backward | `B` | Cursor N WORDS backward |
| End of word | `e` | Forward to end of word N |
| End of WORD | `E` | Forward to end of WORD N |
| Backward end of word | `ge` | Backward to end of previous word |
| Backward end of WORD | `gE` | Backward to end of previous WORD |
| Find char forward | `f{char}` | To Nth occurrence of char, on it |
| Find char forward, before it | `t{char}` | To Nth occurrence of char, just before it |
| Find char backward | `F{char}` | Same as `f` but backward |
| Find char backward, after it | `T{char}` | Same as `t` but backward |
| Repeat find | `;` | Repeat latest `f`/`t`/`F`/`T` |
| Repeat find, reversed | `,` | Repeat latest `f`/`t`/`F`/`T`, opposite direction |
| Go to matching bracket | `%` | Jump to matching `()[]{}` |
| Go to column N | `\|` | Cursor to column N |

### Motion / Navigation — Screen, Paragraph, Jumps

| Command name | Default hotkey | Comment |
|---|---|---|
| Scroll forward a screen | `Ctrl-F` | Scroll N screens forward |
| Scroll backward a screen | `Ctrl-B` | Scroll N screens backward |
| Scroll down half screen | `Ctrl-D` | Scroll down N lines (default half screen) |
| Scroll up half screen | `Ctrl-U` | Scroll up N lines (default half screen) |
| Scroll down one line | `Ctrl-E` | Keep cursor line, scroll view down |
| Scroll up one line | `Ctrl-Y` | Keep cursor line, scroll view up |
| Redraw, cursor to top | `z<CR>` / `zt` | Reposition current line to top of window |
| Redraw, cursor to center | `z.` / `zz` | Reposition current line to center |
| Redraw, cursor to bottom | `z-` / `zb` | Reposition current line to bottom |
| Paragraph backward | `{` | Cursor N paragraphs backward |
| Paragraph forward | `}` | Cursor N paragraphs forward |
| Sentence backward | `(` | Cursor N sentences backward |
| Sentence forward | `)` | Cursor N sentences forward |
| Section backward | `[[` | Cursor N sections backward |
| Section forward | `]]` | Cursor N sections forward |
| Closing section backward | `[]` | Cursor N sections backward (to `}`) |
| Closing section forward | `][` | Cursor N sections forward (to `}`) |
| Older jump-list entry | `Ctrl-O` | Go to N older position in jump list |
| Newer jump-list entry | `Ctrl-I` / `<Tab>` | Go to N newer position in jump list |
| Older change-list position | `g;` | Go to N older position in change list |
| Newer change-list position | `g,` | Go to N newer position in change list |
| Display file name/position | `Ctrl-G` | Show current file name and cursor position |
| Display cursor info (verbose) | `g Ctrl-G` | Show cursor byte/word/char counts |
| Wrapped-line down/up | `gj` / `gk` | Like `j`/`k` but moves by screen line |

### Search

| Command name | Default hotkey | Comment |
|---|---|---|
| Search forward | `/{pattern}<CR>` | Search forward for Nth match |
| Search backward | `?{pattern}<CR>` | Search backward for Nth match |
| Repeat search | `n` | Repeat latest `/` or `?`, same direction |
| Repeat search, reversed | `N` | Repeat latest `/` or `?`, opposite direction |
| Search word under cursor forward | `*` | Search forward for word under cursor |
| Search word under cursor backward | `#` | Search backward for word under cursor |
| Search word under cursor (no boundaries) | `g*` | Like `*` but without `\<`/`\>` |
| Search word under cursor backward (no boundaries) | `g#` | Like `#` but without `\<`/`\>` |
| Find & select next match visually | `gn` | Search forward and Visually select the match |
| Find & select previous match visually | `gN` | Search backward and Visually select the match |
| `:nohlsearch` | `:noh` | Suspend search highlighting **[Ex-only]** |
| `:global` (search-driven) | `:g/pat/cmd` | Execute a command on matching lines **[Ex-only]** |

### Marks & Jumps

| Command name | Default hotkey | Comment |
|---|---|---|
| Set mark | `m{a-zA-Z}` | Set a named mark at cursor position |
| Go to mark, line-wise | `'{mark}` | First non-blank of the marked line |
| Go to mark, exact position | `` `{mark} `` | Exact cursor position of the mark |
| Go to mark without jump-list | `g'{mark}` / `` g`{mark} `` | Like `'`/`` ` `` but doesn't alter jump list |
| Back to previous jump position | `` `` `` | Cursor to position before latest jump |
| Back to previous jump, line-wise | `''` | Line of cursor before latest jump |
| Start of last change/put | `` `[ `` | Also `'[` for line-wise |
| End of last change/put | `` `] `` | Also `']` for line-wise |
| Start of Visual area | `` `< `` | Also `'<` for line-wise |
| End of Visual area | `` `> `` | Also `'>` for line-wise |
| Previous lowercase mark | `` [` `` | Also `['` for line-wise |
| Next lowercase mark | `` ]` `` | Also `]'` for line-wise |
| `:marks` | `:marks` | List all marks **[Ex-only]** |
| `:delmarks` | `:delm` | Delete one or more marks **[Ex-only]** |
| `:jumps` | `:ju` | Print the jump list **[Ex-only]** |
| `:changes` | `:changes` | Print the change list **[Ex-only]** |
| `:clearjumps` | `:cle` | Clear the jump list **[Ex-only]** |

### Visual Mode

| Command name | Default hotkey | Comment |
|---|---|---|
| Start charwise Visual mode | `v` | Also toggles it off |
| Start linewise Visual mode | `V` | Also toggles it off |
| Start blockwise Visual mode | `Ctrl-V` | Also toggles it off |
| Reselect previous Visual area | `gv` | Restore last Visual selection |
| Exchange current/previous area | `gv` (while active) | Swap the two ends of history |
| Stop Visual mode | `<Esc>` | Also `Ctrl-C` |
| Move to other corner | `o` | Swap cursor to the other end of the selection |
| Move to other corner (block) | `O` | Blockwise: swap horizontally |
| Toggle Visual/Select mode | `Ctrl-G` | Switch between Visual and Select |
| Delete selection | `d` / `x` | Delete highlighted text |
| Delete lines | `D` / `X` | Delete highlighted lines (linewise) |
| Change selection | `c` / `s` | Delete selection, start Insert |
| Change lines | `C` / `R` / `S` | Delete highlighted lines, start Insert |
| Yank selection | `y` | Yank highlighted text |
| Yank lines | `Y` | Yank highlighted lines |
| Replace with register | `p` | Replace selection with register contents |
| Replace, keep registers | `P` | Like `p` but doesn't change registers |
| Replace with a character | `r{char}` | Replace every selected char with `{char}` |
| Join selected lines | `J` | Join with spaces |
| Join without space | `gJ` | Join without inserting spaces |
| Uppercase / lowercase | `U` / `u` | Case-convert selection |
| Swap case | `~` | Toggle case of selection |
| Shift left / right | `<` / `>` | Shift selection by `'shiftwidth'` |
| Filter selection | `!{filter}` | Pipe through external command |
| Format selection | `gq` | Format highlighted lines |
| Start a `:` range command | `:` | Prefills `:'<,'>` |
| Increment/decrement numbers in area | `Ctrl-A` / `Ctrl-X` | Add/subtract from every number in selection |
| Increment sequentially | `g Ctrl-A` | Increment as an arithmetic sequence |
| Block append | `A` (block mode) | Append same text to every line's end |
| Block insert | `I` (block mode) | Insert same text before every line |
| Extend selection with text object | `a{obj}` / `i{obj}` | e.g. `aw`, `iw`, `ap`, `ip`, `a"`, `i"`, `at`, `it`, `ab`/`aB`, `ib`/`iB` |

### Text Objects (operator-pending & Visual)

| Command name | Default hotkey | Comment |
|---|---|---|
| A word | `aw` | Word plus trailing whitespace |
| Inner word | `iw` | Word only |
| A WORD | `aW` | WORD plus trailing whitespace |
| Inner WORD | `iW` | WORD only |
| A sentence | `as` | Sentence plus whitespace |
| Inner sentence | `is` | Sentence only |
| A paragraph | `ap` | Paragraph plus whitespace |
| Inner paragraph | `ip` | Paragraph only |
| A `()` block | `ab` / `a(` / `a)` | From `(` to matching `)`, inclusive |
| Inner `()` block | `ib` / `i(` / `i)` | Contents only |
| A `{}` block | `aB` / `a{` / `a}` | From `{` to matching `}`, inclusive |
| Inner `{}` block | `iB` / `i{` / `i}` | Contents only |
| A `[]` block | `a[` / `a]` | From `[` to matching `]`, inclusive |
| Inner `[]` block | `i[` / `i]` | Contents only |
| A `<>` block | `a<` / `a>` | From `<` to matching `>`, inclusive |
| Inner `<>` block | `i<` / `i>` | Contents only |
| A quoted string | `a"` / `a'` / `` a` `` | Quotes plus the quoted text |
| Inner quoted string | `i"` / `i'` / `` i` `` | Text without the quotes |
| A tag block | `at` | XML/HTML tag pair, inclusive |
| Inner tag block | `it` | Contents between the tags |
| Force operator charwise | `v` (op-pending) | Override motion to charwise |
| Force operator linewise | `V` (op-pending) | Override motion to linewise |
| Force operator blockwise | `Ctrl-V` (op-pending) | Override motion to blockwise |

### Registers

| Command name | Default hotkey | Comment |
|---|---|---|
| Select register | `"{reg}` | Prefix for next delete/yank/put |
| Unnamed register | `""` | Default register for unqualified yank/delete |
| Yank register | `"0` | Holds most recent yank (not delete) |
| Small-delete register | `"-` | Holds deletes under one line |
| Black-hole register | `"_` | Discards text, doesn't affect other registers |
| Last-inserted-text register | `".` | Read-only, holds last inserted text |
| Filename register | `"%` | Read-only, current file name |
| Alternate-filename register | `"#` | Read-only, alternate file name |
| Last-command register | `":` | Read-only, last Ex command |
| Last-search-pattern register | `"/` | Read-only, last search pattern |
| Expression register | `"=` | Evaluates an expression on insert |
| `:registers` | `:reg` (also `:display`/`:di`) | Display contents of registers **[Ex-only]** |

### Macros

| Command name | Default hotkey | Comment |
|---|---|---|
| Record macro | `q{0-9a-zA-Z"}` | Start/stop recording into a register |
| Execute macro | `@{a-z}` | Run register contents as commands, N times |
| Repeat last macro | `@@` | Repeat the previous `@{reg}` |
| Repeat last `:` command | `@:` | Repeat previous Ex command N times |
| Replay last recorded register | `Q` | Executes the last-used `@{reg}` (Visual mode: runs it per line) |
| Edit command-line in cmdwin | `q:` / `q/` / `q?` | Open the command-line window to edit history |

### Folding

| Command name | Default hotkey | Comment |
|---|---|---|
| Create fold for motion | `zf{motion}` | Manually create a fold |
| Delete fold | `zd` | Delete fold at cursor |
| Delete folds recursively | `zD` | Delete fold and all nested folds |
| Eliminate all folds | `zE` | Remove every fold in the window |
| Open fold | `zo` | Open one fold level |
| Open folds recursively | `zO` | Open fold and all nested folds |
| Close fold | `zc` | Close one fold level |
| Close folds recursively | `zC` | Close fold and all nested folds |
| Toggle fold | `za` | Open if closed, close if open |
| Toggle fold recursively | `zA` | Recursive version of `za` |
| Set foldlevel to 0 | `zM` | Close all folds |
| Set foldlevel to deepest | `zR` | Open all folds |
| Increase foldlevel | `zr` | Add one to `'foldlevel'` |
| Decrease foldlevel | `zm` | Subtract one from `'foldlevel'` |
| Toggle `'foldenable'` | `zi` | Enable/disable folding entirely |
| View cursor line | `zv` | Open just enough folds to reveal cursor |
| Move to start of next fold | `zj` | Jump to next fold boundary |
| Move to end of previous fold | `zk` | Jump to previous fold boundary |
| Move to start of open fold | `[z` | Go to the start of the current open fold |
| Move to end of open fold | `]z` | Go to the end of the current open fold |
| `:foldopen` | `:foldo` | Open folds by range **[Ex-only]** |
| `:foldclose` | `:foldc` | Close folds by range **[Ex-only]** |

### Quickfix List

| Command name | Default hotkey | Comment |
|---|---|---|
| `:make` | `:mak` | Run `'makeprg'`, populate quickfix list **[Ex-only]** |
| `:grep` | `:gr` | Run `'grepprg'`, jump to first match **[Ex-only]** |
| `:vimgrep` | `:vim` | Search patterns in files using Vim's regex **[Ex-only]** |
| `:cexpr` | `:cex` | Load quickfix list from an expression **[Ex-only]** |
| `:cfile` | `:cf` | Read errors from a file **[Ex-only]** |
| `:copen` | `:cope` | Open the quickfix window **[Ex-only]** |
| `:cclose` | `:ccl` | Close the quickfix window **[Ex-only]** |
| `:cwindow` | `:cw` | Open quickfix window only if there are errors **[Ex-only]** |
| `:cnext` | `:cn` | Go to next error **[Ex-only]** |
| `:cprevious` | `:cp` | Go to previous error **[Ex-only]** |
| `:cfirst` | `:cfir` | Go to first error **[Ex-only]** |
| `:clast` | `:cla` | Go to last error **[Ex-only]** |
| `:cc` | `:cc` | Go to a specific error by number **[Ex-only]** |
| `:cdo` | `:cdo` | Execute a command on every quickfix entry **[Ex-only]** |
| `:colder` / `:cnewer` | `:col` / `:cnew` | Switch to older/newer quickfix list **[Ex-only]** |
| Go to next error | `]q` | vim-unimpaired-style, same as `:cnext` |
| Go to previous error | `[q` | vim-unimpaired-style, same as `:cprevious` |
| Go to first error | `[Q` | Same as `:crewind` |
| Go to last error | `]Q` | Same as `:clast` |

### Location List

| Command name | Default hotkey | Comment |
|---|---|---|
| `:lopen` | `:lop` | Open the location window **[Ex-only]** |
| `:lclose` | `:lcl` | Close the location window **[Ex-only]** |
| `:lnext` | `:lne` | Go to next location **[Ex-only]** |
| `:lprevious` | `:lp` | Go to previous location **[Ex-only]** |
| `:lfirst` | `:lfir` | Go to first location **[Ex-only]** |
| `:llast` | `:lla` | Go to last location **[Ex-only]** |
| `:ll` | `:ll` | Go to a specific location by number **[Ex-only]** |
| `:lgrep` | `:lgr` | Like `:grep` but fills the location list **[Ex-only]** |
| `:lmake` | `:lmak` | Like `:make` but fills the location list **[Ex-only]** |
| Go to next location | `]l` | vim-unimpaired-style, same as `:lnext` |
| Go to previous location | `[l` | vim-unimpaired-style, same as `:lprevious` |
| Go to first/last location | `[L` / `]L` | Same as `:lrewind` / `:llast` |

### Terminal Mode

| Command name | Default hotkey | Comment |
|---|---|---|
| `:terminal` | `:te` | Open a terminal buffer **[Ex-only]** |
| Go to Normal mode from terminal | `Ctrl-\ Ctrl-N` | Exit terminal-job input, keep buffer |
| Run one Normal command from terminal | `Ctrl-\ Ctrl-O` | Execute one Normal command, return to terminal |
| Enter terminal mode | `i` / `a` | From Normal mode in a terminal buffer, resumes job input |

### Diffs

| Command name | Default hotkey | Comment |
|---|---|---|
| `:diffthis` | `:difft` | Make current window a diff window **[Ex-only]** |
| `:diffoff` | `:diffo` | Switch off diff mode for a window **[Ex-only]** |
| `:diffupdate` | `:dif` | Recompute diff highlighting **[Ex-only]** |
| `:diffsplit` | `:diffs` | Split window and diff with another file **[Ex-only]** |
| `:diffpatch` | `:diffp` | Apply a patch and view differences **[Ex-only]** |
| `:diffget` | `:diffg` | Pull changes from the other diff buffer **[Ex-only]** |
| `:diffput` | `:diffpu` | Push changes to the other diff buffer **[Ex-only]** |
| Obtain diff hunk | `do` | Normal-mode alias for `:diffget` |
| Put diff hunk | `dp` | Normal-mode alias for `:diffput` |
| Previous change/hunk | `[c` | Move to start of previous diff change |
| Next change/hunk | `]c` | Move to start of next diff change |

### Spelling

| Command name | Default hotkey | Comment |
|---|---|---|
| Next misspelled word | `]s` | Move forward to a flagged word |
| Previous misspelled word | `[s` | Move backward to a flagged word |
| Spelling suggestions | `z=` | List suggestions for word under cursor |
| Mark word correct (permanent) | `zg` | Add to the spellfile |
| Mark word incorrect (permanent) | `zw` | Mark as a spelling mistake in the spellfile |
| Mark word correct (session-only) | `zG` | Temporary, doesn't touch the spellfile |
| Mark word incorrect (session-only) | `zW` | Temporary, doesn't touch the spellfile |
| Undo `zg` / `zw` / `zG` / `zW` | `zug` / `zuw` / `zuG` / `zuW` | Remove a good/bad-word mark |
| Suggest replacement (Insert mode) | `Ctrl-X Ctrl-S` | Spelling suggestions while typing |
| `:setlocal spell` | `:setl spell` | Turn on spell-checking **[Ex-only]** |
| `:spellgood` | `:spe` | Add a good word to the spellfile **[Ex-only]** |
| `:spellwrong` | `:spellw` | Add a wrong word to the spellfile **[Ex-only]** |
| `:spellundo` | `:spellu` | Remove a word from the spellfile **[Ex-only]** |
| `:spellrepall` | `:spellr` | Replace all instances like the last `z=` fix **[Ex-only]** |
| `:mkspell` | `:mksp` | Compile a `.spl` spell file **[Ex-only]** |

### LSP-Adjacent & Diagnostics Built-ins (0.11 global defaults)

| Command name | Default hotkey | Comment |
|---|---|---|
| LSP rename | `grn` | Calls `vim.lsp.buf.rename()`, unconditional global default |
| LSP code action | `gra` | Calls `vim.lsp.buf.code_action()` (Normal & Visual) |
| LSP references | `grr` | Calls `vim.lsp.buf.references()` |
| LSP go to implementation | `gri` | Calls `vim.lsp.buf.implementation()` |
| LSP go to type definition | `grt` | Calls `vim.lsp.buf.type_definition()` |
| LSP document symbols | `gO` | Calls `vim.lsp.buf.document_symbol()` |
| LSP signature help | `Ctrl-S` (Insert/Select mode) | Calls `vim.lsp.buf.signature_help()` |
| Keyword lookup / LSP hover | `K` | Runs `'keywordprg'`; LSP sets this to `vim.lsp.buf.hover()` on attach if the server supports it |
| Next diagnostic | `]d` | Calls `vim.diagnostic.jump()` forward |
| Previous diagnostic | `[d` | Calls `vim.diagnostic.jump()` backward |
| Last diagnostic | `]D` | Jump to the last diagnostic, no wrap |
| First diagnostic | `[D` | Jump to the first diagnostic, no wrap |
| Show diagnostic float | `Ctrl-W d` | Also `Ctrl-W Ctrl-D`; opens `vim.diagnostic.open_float()` |
| `:checkhealth` | `:che` | Run healthchecks, including LSP client health **[Ex-only]** |
| Snippet jump forward | `Tab` (Insert/Select, when snippet active) | Calls `vim.snippet.jump(1)` |
| Snippet jump backward | `Shift-Tab` (Insert/Select, when snippet active) | Calls `vim.snippet.jump(-1)` |

### Command-line (Ex Command-line) Editing

| Command name | Default hotkey | Comment |
|---|---|---|
| Enter Ex command-line | `:` | Also `{count}:` prefills a range |
| Enter filter command-line | `!{motion}` | Prompts `:{range}!` |
| Cursor to start of command-line | `Ctrl-B` | Also `Home` |
| Cursor to end of command-line | `Ctrl-E` | Also `End` |
| Do completion, insert all matches | `Ctrl-A` | Completes and inserts every match |
| List matching completions | `Ctrl-D` | Shows possible completions without inserting |
| Do completion (wildchar) | `Tab` | Default `'wildchar'`; cycles matches |
| Do completion, longest common part | `Ctrl-L` | Complete only the unambiguous portion |
| Older command-line from history | `Ctrl-P` (also `Up`) | Recall previous history entry |
| Newer command-line from history | `Ctrl-N` (also `Down`) | Recall next history entry |
| Insert register contents | `Ctrl-R {reg}` | Insert a register into the command-line |
| Delete word before cursor | `Ctrl-W` | Delete the preceding word |
| Delete entire command-line | `Ctrl-U` | Clear the line |
| Open command-line window | `q:` (also `Ctrl-F` / `'cedit'`) | Edit `:`-history in a buffer |
| Open search-history window | `q/` / `q?` | Edit `/`- or `?`-history in a buffer |
| Abandon command-line | `<Esc>` | Also `Ctrl-C` |
| Replace command-line via expression | `Ctrl-\ e {expr}` | Set the command-line to an expression's result |

### Miscellaneous / General

| Command name | Default hotkey | Comment |
|---|---|---|
| `:help` | `:h` | Open a help window **[Ex-only]** |
| `:quit` | `:q` | Quit current window (quits Vim if last one) **[Ex-only]** |
| `:quitall` | `:qa` | Quit all windows and exit Vim **[Ex-only]** |
| `:cquit` | `:cq` | Quit Vim returning a non-zero exit code **[Ex-only]** |
| `:version` | `:ve` | Print version and build info **[Ex-only]** |
| `:set` | `:se` | Show or set options **[Ex-only]** |
| `:redir` | `:redi` | Redirect messages to a file/register/variable **[Ex-only]** |
| `:messages` | `:mes` | Show previously displayed messages **[Ex-only]** |
| `:history` | `:his` | Print a history list (cmd/search/etc.) **[Ex-only]** |
| `:autocmd` | `:au` | Define or show autocommands **[Ex-only]** |
| `:command` | `:com` | Define a user command **[Ex-only]** |
| `:map` / `:noremap` | `:map` / `:no` | Show or create a key mapping **[Ex-only]** |
| `:lua` | `:lua` | Execute a Lua command **[Ex-only]** |
| `:luafile` | `:luaf` | Execute a Lua script file **[Ex-only]** |
| Write and close window | `ZZ` | Write buffer if changed, then close window |
| Close window without writing | `ZQ` | Discard changes and close window (like `:q!`) |
| Open file/URL under cursor | `gx` | Calls `vim.ui.open()` on the URL/path at cursor |
| Redraw screen | `Ctrl-L` | Also clears search highlight & updates diffs (0.9+ default) |
| Suspend Nvim | `Ctrl-Z` | Suspend to shell (also `:stop`/`:suspend`) |
| Execute one shell command | `:!{cmd}` | Run an external command **[Ex-only]** |
| Repeat last shell command | `:!!` | Re-run the previous `:!` command **[Ex-only]** |
| Print hex of char under cursor | `g8` | Show UTF-8 byte breakdown |
| Print ASCII value of char | `ga` | Show decimal/hex/octal value |

## Autodesk Fusion 360

*IDs and hotkeys compiled from web research (Autodesk's own API
documentation, real Fusion 360 add-in source code, and third-party
cheat sheets/forum discussions). Most primary Autodesk help pages
weren't directly fetchable in this environment, so most entries could
only be cross-checked through search snippets rather than opened and
verified verbatim. A handful of Command IDs (marked "Confirmed internal
ID") were verified directly against real Fusion 360 add-in source or
Autodesk's own `itemById()` examples; every other row uses the exact UI
command name in place of an unconfirmed internal ID, as flagged per
section. Hotkeys are given only where corroborated by at least one
source — most Fusion 360 commands ship with no default binding and are
reached via the `S` command-search box or bound manually in
Preferences > Keyboard Shortcuts.*

### File
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| New Design | Ctrl+N | Creates a new empty design document |
| Open | Ctrl+O | Opens a design from the Data Panel |
| Save | Ctrl+S | Saves the active design to the cloud |
| Save As | | Saves a copy of the design under a new name/location |
| Save a Copy | | Saves an unlinked copy of the current design |
| `ExportCommand` | | Confirmed internal ID. Exports the design/body to STEP, STL, IGES, F3D, etc. |
| 3D Print | | Sends the model to the 3D Print utility |
| Upload | | Uploads local CAD files into the Data Panel |
| Open From Local Drive | | Opens a non-cloud file (e.g. STEP, IGES) directly |
| Close | | Closes the active document tab |

### Edit
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Undo | Ctrl+Z | Steps back one action |
| Redo | Ctrl+Y | Re-applies an undone action |
| Cut | Ctrl+X | Cuts the selection |
| Copy | Ctrl+C | Copies the selection |
| Paste | Ctrl+V | Pastes the clipboard contents |
| `FusionPasteNewCommand` | | Confirmed internal ID. Pastes the clipboard contents as a new independent component |
| Find | Ctrl+F | Searches for an item in the browser tree |
| Select All | Ctrl+A | Selects all visible objects in context |
| Delete | Delete | Deletes the selected entities |
| Cancel | Esc | Cancels the active command/tool |

### View, Display & Navigation
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Command Search | S | Opens the type-ahead command search box |
| Look At | L | Orients the view normal to the selected face/sketch |
| Zoom | Scroll wheel | Zooms the 3D view in/out |
| Pan | Shift+MMB drag | Pans the 3D view |
| Orbit (Free Orbit) | MMB drag | Rotates the 3D view around the model |
| Home View | | Returns to the default isometric orientation |
| Fit | | Zooms to fit all visible geometry in the window |
| Shaded | Ctrl+4 | Visual style: shaded, no edges |
| Shaded with Hidden Edges | Ctrl+5 | Visual style: shaded, hidden edges dimmed |
| Shaded with Visible Edges Only | Ctrl+6 | Visual style: shaded, hidden edges suppressed |
| Wireframe | Ctrl+7 | Visual style: edges only, no shading |
| Wireframe with Hidden Edges | Ctrl+8 | Visual style: wireframe with hidden lines shown |
| Wireframe with Visible Edges Only | Ctrl+9 | Visual style: wireframe, hidden lines suppressed |
| Toggle Visibility | V | Shows/hides the selected component or body |
| Measure | I | Opens the Inspect > Measure tool |
| Section Analysis | | Cuts a temporary cross-section through the model for inspection |
| Interference Check | | Detects clashes between selected bodies/components |

### Panels & Workspace Layout
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Next Workspace | Ctrl+] | Switches to the next workspace in the switcher order |
| Previous Workspace | Ctrl+[ | Switches to the previous workspace in the switcher order |
| Show/Hide ViewCube | Ctrl+Alt+V | Toggles the navigation ViewCube |
| Show/Hide Browser | Ctrl+Alt+B | Toggles the design browser tree panel |
| Show/Hide Comments | Ctrl+Alt+A | Toggles the comments panel |
| Show/Hide Text Commands | Ctrl+Alt+C | Toggles the Text Commands debug console |
| Show/Hide Navigation Bar | Ctrl+Alt+N | Toggles the navigation bar |
| Show/Hide Data Panel | Ctrl+Alt+P | Toggles the Data Panel |
| Reset to Default Layout | Ctrl+Alt+R | Restores the default UI panel layout |

### Sketch — Draw
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Line | L | Draws a line or connected line/arc chain |
| Rectangle (2-Point) | R | Draws a rectangle from two corners |
| Center Rectangle | | Draws a rectangle from its center point |
| Circle (Center Diameter) | C | Draws a circle from center and diameter |
| Arc (3-Point) | A | Draws an arc through three points |
| Ellipse | | Draws an ellipse from center and two axes |
| Polygon | | Draws a regular polygon |
| Slot | | Draws a slot profile |
| Spline (Fit Point) | | Draws a smooth curve through fit points |
| Conic Curve | | Draws a conic-section curve |
| Sketch Point | | Places a reference point in the sketch |
| Sketch Text | | Adds text geometry to the sketch |

### Sketch — Modify
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Trim | T | Trims sketch geometry to the nearest intersection |
| Extend | | Extends a curve to the next intersection |
| Offset | O | Offsets selected sketch curves by a distance |
| Mirror (Sketch) | M | Mirrors sketch geometry about a line |
| Move/Copy (Sketch) | | Moves, rotates, or copies sketch geometry |
| Scale (Sketch) | | Scales sketch geometry from a point |
| Toggle Construction | X | Toggles selected geometry between normal and construction |
| Fillet (Sketch) | | Rounds a corner between two sketch curves |
| Break | | Splits a curve at a point |
| Project | | Projects existing geometry onto the active sketch |
| Include 3D Geometry | | Projects 3D edges/points into the sketch as sketch curves |

### Sketch — Constraints & Dimensions
*(UI names — internal IDs not confirmed, except SketchEditDimensionCmdDef)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Sketch Dimension | D | Applies a driving dimension to selected geometry |
| `SketchEditDimensionCmdDef` | | Confirmed internal ID. Edits an existing sketch dimension's value |
| Coincident Constraint | | Constrains two points to coincide |
| Collinear Constraint | | Constrains two lines to lie on the same line |
| Concentric Constraint | | Constrains two circles/arcs to share a center |
| Tangent Constraint | | Constrains a curve to be tangent to another |
| Parallel Constraint | | Constrains two lines to stay parallel |
| Perpendicular Constraint | | Constrains two lines to stay perpendicular |
| Horizontal/Vertical Constraint | | Constrains a line to horizontal or vertical |
| Equal Constraint | | Constrains two curves to equal length/radius |
| Symmetry Constraint | | Constrains two entities symmetric about a line |
| Fix/Unfix | | Locks or unlocks a sketch entity's position |
| Finish Sketch | | Exits sketch edit mode back to the 3D model |

### Solid — Create
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Extrude | E | Extrudes a profile into a solid or surface |
| Revolve | | Revolves a profile around an axis |
| Sweep | | Sweeps a profile along a path |
| Loft | | Blends between two or more profiles |
| Rib | | Creates a thin structural rib from an open profile |
| Web | | Creates a network of ribs from sketch lines |
| Emboss | | Raises or recesses sketch geometry on a face |
| Boundary Fill | | Creates solids/surfaces from enclosed regions |
| Coil | | Creates a helical coil feature |
| Pipe | | Creates a solid pipe along a path |
| Create Base Feature | | Starts an editable base feature (non-parametric) |

### Solid — Modify
*(UI names — internal IDs not confirmed, except a few marked)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Fillet | F | Rounds selected edges |
| Chamfer | | Bevels selected edges |
| Shell | | Hollows a solid, leaving selected faces open |
| Draft | | Adds a taper angle to faces for molding/casting |
| Thread | | Adds cosmetic or modeled thread to a cylindrical face |
| Press Pull | Q | Extrudes/offsets a face or profile interactively |
| Move/Copy | M | Moves, rotates, or copies bodies/components |
| Combine | | Boolean joins/cuts/intersects bodies |
| Split Body | | Splits a body using a plane/face/sketch curve |
| Split Face | | Splits a face using a plane/sketch curve |
| Replace Face | | Replaces a face with another surface |
| Delete Face | | Removes a face and heals or leaves the gap |
| Scale | | Scales a body uniformly or non-uniformly |
| `ChangeParameterCommand` | | Confirmed internal ID. Opens the Change Parameter / user-parameter edit dialog |
| `RenameCommand` | | Confirmed internal ID. Renames the selected component, body, sketch, or feature |
| Physical Material | | Assigns a physical (density/mass) material to a body |
| Appearance | | Assigns a visual appearance/material to a body or face |

### Solid — Pattern, Mirror & Combine
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Rectangular Pattern | | Patterns features/bodies along one or two directions |
| Circular Pattern | | Patterns features/bodies around an axis |
| Path Pattern | | Patterns features/bodies along a curve |
| Mirror (Feature) | | Mirrors solid features/bodies about a plane |
| Pattern on Path | | Alternate name for Path Pattern in some UI versions |
| Component Pattern | | Patterns whole occurrences (not just features) |

### Surface
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Patch | | Fills a closed boundary loop with a surface |
| Extend (Surface) | | Extends a surface edge |
| Trim (Surface) | | Trims a surface against a cutting tool |
| Stitch | | Stitches adjacent surfaces into one surface or a solid |
| Unstitch | | Separates a stitched surface/solid back into faces |
| Thicken | | Converts a surface into a solid of given thickness |
| Ruled Surface | | Creates a ruled surface from an edge/curve |
| Offset (Surface) | | Offsets a surface by a distance |

### Sculpt / T-Spline
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Create Form | | Enters the Sculpt (T-Spline) editing environment |
| Sculpt Box | | Creates a primitive box T-Spline body |
| Sculpt Sphere | | Creates a primitive sphere T-Spline body |
| Sculpt Cylinder | | Creates a primitive cylinder T-Spline body |
| Sculpt Plane | | Creates a primitive flat T-Spline body |
| Revolve (Sculpt) | | Creates a T-Spline body by revolving a profile |
| Edit Form | | Free-form moves/rotates/scales T-Spline control points |
| Bridge (Sculpt) | | Connects two open edges/faces with new geometry |
| Fill Hole (Sculpt) | | Caps an open edge loop |
| Crease | | Sharpens an edge on a smoothed T-Spline body |

### Assembly / Joints
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Joint | J | Creates a joint defining relative motion between components |
| As-Built Joint | Shift+J | Creates a joint that preserves the components' current position |
| Joint Origin | | Places a reusable joint reference point |
| Rigid Group | | Locks a set of components together as one rigid body |
| New Component | | Creates a new empty component in the assembly |
| Ground | | Fixes a component in place relative to the assembly origin |
| Align | | Aligns two components' faces/points/axes |
| Motion Study | | Opens a study for previewing joint-driven motion |
| Contact Sets | | Enables collision detection between selected components |
| Insert Derive | | Inserts another design as a linked/derived component |

### Sheet Metal
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Flange | | Adds a sheet-metal flange from a sketch, edge, or face |
| Contour Flange | | Extrudes a sheet-metal profile with bend allowances |
| Lofted Flange | | Lofts a sheet-metal wall between two open profiles |
| Convert to Sheet Metal | | Converts an existing solid body to sheet metal rules |
| Unfold | | Temporarily unfolds bent walls for editing |
| Refold | | Re-folds walls unfolded by Unfold |
| Flat Pattern | | Generates the flattened manufacturing pattern |
| Bend | | Adds a bend feature at a sketch line |
| Corner Round | | Rounds a sheet-metal corner |
| Corner Chamfer | | Chamfers a sheet-metal corner |
| Rip | | Splits a face to allow it to bend or unfold |

### Mesh
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Insert Mesh | | Imports a mesh file (STL, OBJ, etc.) into the design |
| Mesh to BRep | | Converts a mesh body to a solid B-Rep body |
| Remesh | | Re-triangulates a mesh to a new resolution |
| Reduce | | Reduces a mesh's triangle count |
| Prepare Mesh | | Repairs/cleans a mesh before further editing |
| Modify Mesh | | Smooths, offsets, or otherwise edits mesh geometry |
| Mesh Section Sketch | | Creates a sketch from a mesh cross-section |
| Watertight Analysis | | Checks a mesh body for gaps/non-manifold errors |

### Simulation
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| New Study | | Creates a new simulation study and selects a study type |
| Loads | | Applies forces, pressures, or torques to the model |
| Constraints (Simulation) | | Fixes or restrains geometry for the study |
| Contacts | | Defines how bodies interact/touch in the study |
| Study Materials | | Assigns simulation material properties |
| Mesh Settings | | Controls the finite-element mesh density |
| Simulate | | Runs the solve for the active study |
| Results | | Displays stress/displacement/factor-of-safety results |

### Render
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| In-Canvas Render | | Toggles progressive photorealistic rendering in the viewport |
| Render (Cloud/Local Gallery) | | Sends the scene to a full offline render job |
| Appearance (Render) | | Opens the appearance library for material assignment |
| Decal | | Applies an image decal to a face |
| Scene Settings | | Configures environment lighting/backdrop for rendering |
| Ground Plane | | Adds/adjusts a render ground plane and shadow catcher |

### Drawing / Documentation
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Base View | | Places the first view of a model on a drawing sheet |
| Projected View | | Creates an orthographic view projected from a base view |
| Section View | | Creates a cross-sectional drawing view |
| Detail View | | Creates a magnified callout of part of a view |
| Break View | | Shortens a long view with a break |
| General Dimension | | Adds a linear/angular/radial dimension to a drawing |
| Ordinate Dimension | | Adds a baseline (ordinate) dimension set |
| Center Mark | | Marks the center of a circular edge |
| Centerline | | Adds a centerline between two edges/points |
| Surface Finish Symbol | | Adds a surface-texture callout |
| Geometric Tolerance | | Adds a GD&T feature control frame |
| Balloon | | Adds a BOM item-number balloon |
| Parts List | | Inserts a bill-of-materials table |
| Hole Table | | Inserts a table listing hole positions/sizes |
| New Sheet | | Adds a new drawing sheet |

### Manufacture / CAM
*(UI names — internal IDs not confirmed)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| New Setup | | Defines stock, WCS, and post configuration for a job |
| 2D Adaptive Clearing | | Roughs a pocket with adaptive-clearing toolpaths |
| 2D Pocket | | Clears material from a closed 2D boundary |
| 2D Contour | | Cuts along a 2D profile boundary |
| 2D Face | | Faces the top of the stock flat |
| Drill | | Cycles a drilling toolpath on selected holes |
| 3D Adaptive Clearing | | Roughs a 3D solid with adaptive-clearing toolpaths |
| 3D Parallel | | Finishes a 3D surface with parallel passes |
| 3D Contour | | Finishes a 3D surface following contour lines |
| Regenerate Toolpath | G | Recomputes the selected/out-of-date toolpaths |
| Simulate (CAM) | | Simulates material removal for the selected toolpaths |
| Post Process | | Generates NC code from the selected toolpaths |
| Setup Sheet | | Generates a documentation sheet for a setup |
| Backplot | | Steps through toolpath motion move-by-move |

### Timeline / History
*(UI names — internal IDs not confirmed, except two marked)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| Roll History Marker Here | | Moves the timeline playhead to just after a feature |
| Suppress Features | | Temporarily disables selected timeline features |
| Skip | | Excludes selected features from the design without deleting them |
| Edit Feature | | Reopens a timeline feature's dialog for editing |
| Group | | Groups consecutive timeline features together |
| `FusionComputeAllCommand` | | Confirmed internal ID. Forces a full recompute of the design |
| `FusionRenameTimelineEntryCommand` | | Confirmed internal ID. Renames a timeline feature entry |
| Insert (Timeline Position) | | Sets where new features are inserted into history |

### Data Panel / Project
*(UI names — internal IDs not confirmed, except two marked)*

| Command ID | Default hotkey | Comment |
|---|---|---|
| New Project | | Creates a new project folder in the Data Panel |
| New Folder | | Creates a subfolder within a project |
| Upload (Data Panel) | | Uploads local files into the current project |
| Move | | Moves a file/folder to another project or folder |
| Copy | | Copies a file to another project or folder |
| Delete (Data Panel) | | Deletes a file or folder (moves to Recycle Bin) |
| Version History | | Lists and restores prior saved versions of a file |
| Manage Access | | Sets sharing/collaborator permissions on a project |
| `FusionPropertiesCommand` | | Confirmed internal ID. Opens the document properties dialog |
| `DesignConfigurationActivateRowCmd` | | Confirmed internal ID. Activates a row in the design's Configurations table |
