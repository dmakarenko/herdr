# Glossary

- **Go to picker**: the centered overlay that lists the machines, workspaces, tabs, and panes of the session and jumps to the selected one. `goto` is its keybinding name and `Navigator` its name in the code.
  _Avoid_: session navigator, navigator, command palette
- **Pane**: one terminal in a tab. An agent pane runs a detected coding agent. A plain terminal pane runs anything else, for example a shell.
  _Avoid_: terminal (for the container), agent (for a pane with no agent)
- **Recent panes**: the panes that one TUI client focused last, newest first, each kept in focus for at least the dwell time. The current pane is not one of them.
  _Avoid_: history, last pane (that is the single previous pane of `last_pane`)
- **Dwell time**: the least time a pane must keep focus to become a recent pane.
- **Turn text**: the text of the last turn of an agent session in a pane: the last user prompt and the start of the last agent answer. A hook reports it to herdr as pane metadata tokens.
  _Avoid_: transcript (that is the full session file), scrollback (that is terminal text)
- **Content match**: a Go to row that matches the query only by its turn text, not by a name, tab, workspace, branch, or path.
