# Running the talk

```bash
presenterm -x presentation.md     # -x enables the +exec escape code snippet
```

- Use a graphics-protocol terminal (kitty, WezTerm, Ghostty, iTerm2), else title is ASCII blocks
- Check fit (120x30+): `presenterm --validate-overflows presentation.md`
- Title image: `python assets/make_title.py` (needs Pillow)
- Electron meme: swap in `assets/history/bloat.png` (`make_bloat.py`) if publishing
- History photo credits: `assets/history/CREDITS.md`
- Bullets must fit on one line; `[2, 3]` history columns: under ~54 chars
- Second terminal / tmux window ready for demos

## Pre-talk checklist

- [ ] `cargo build --release --manifest-path demo/ratatui_list/Cargo.toml` (before, not live)
- [ ] doom-cli (`cd doomgeneric && make -f Makefile.cli`) + terminal-doom built, `doom1.wad` to hand
- [ ] Test doom-cli; if garbled use `-charset quadrant` or `-charset half`
- [ ] `podman pull fathyb/carbonyl` (big, not on conference wifi)
- [ ] Check `telnet mapscii.me`, `ssh terminal.shop` are up; screenshots as backup
- [ ] Check `telnet towel.blinkenlights.nl`; drop it if down
- [ ] Install `sl`, `cmatrix`, `cbonsai`, `asciiquarium`, `lolcat`
- [ ] Font size up, notifications off

<!-- end_slide -->

# Title / segue

- Slide 1: play it straight; "CONFIDENTIAL" footer gets a smirk
- ASCII banner: "Sorry, wrong kind of UI." This whole thing is a TUI

# How did we get here?

- ~30-45s per slide; each era solved a problem and created the next
- Punch cards: batch before interactive; IBM 80-column card (1928)
  - Cols 73-80 = sequence numbers; the stripe is the low-tech backup
  - Fortran used 72 cols (IBM 704 reader); "lace card" jammed readers
  - 80 columns → terminal widths, sets up VT100
- Teletype: _interactive_; ASR-33 = Unix console
  - `/dev/tty` and raw-mode `\r\n` come from here (callback in Part 4)
- VT100: most emulated terminal, not the first
  - No colour, no italics, couldn't hide cursor (came later)
  - xterm still answers as "VT100 with Advanced Video Option"
  - `vi` predates it (ADM-3A: `hjkl` arrows, `~` home)
  - "Your terminal is still pretending to be a VT100" → Part 3
- curses/Rogue: Ken Arnold ~1978, lifted vi's cursor code
  - `@` = where you're "at"; Ritchie: "biggest waste of CPU cycles"
  - "Draw to buffer, send minimal updates" = Ratatui, 45 years later
- Macintosh: GUIs a huge win; PARC Alto (1973), Engelbart's mouse; Mac 1984
- 1990s: toolkit per platform; Java; land "look weird everywhere"
- 2000s: web won on distribution; "folksonomy" laugh
- Electron: real win for shipping; image is the joke, 400MB punchline, move on
- Not anti-GUI: right tool for the job, terminal is underrated

# The mad stuff

- doom-cli: sextant blocks, 6 pixels/cell, 2 colours/cell
  - No key-up events: guesses from key repeat, one key at a time
  - Kitty keyboard protocol fixes this (sends key-up)
  - Controls: arrows, `z` shoot, space doors; README on sound: "Nope."
- terminal-doom: real pixels via kitty graphics (kitty/Ghostty)
- Carbonyl: Wikipedia "Text-based user interface" page; YouTube if time
- MapSCII: zoom to the venue
- terminal.shop: show menu, don't buy live
- Rapid fire: `sl`, `cmatrix`, `cbonsai -l`, `asciiquarium`, `ls | lolcat`

# The useful stuff

- Quick demo: lazygit, k9s or `btm`
- Newest dev tooling (AI agents) chooses the terminal
- Agents "built with" column sets up Part 4:
  - Codex: Ratatui; Gemini CLI: Ink; OpenCode: OpenTUI (Zig + SolidJS)
  - pi: own library with differential rendering (diffing slide)
  - Claude Code "TypeScript + React" is from public statements, not the package

# What makes a good TUI good

- Examples: lazygit key hints, yazi async previews, bottom density, fzf composability
- "Leaves no mess": a good TUI never makes you need `reset`

# How it works

- `+exec` snippet: `ctrl+e` to run
- Alt screen: `less` a file, quit, scrollback untouched
- Raw mode: `cat -v`, press arrows, Ctrl-D to exit
- `stty -a` if asked
- Ctrl-S freeze = XON/XOFF, Ctrl-Q unfreezes
- Broken terminal: `reset` or `stty sane`

# Build it

- `python3 demo/raw/list.py` from repo root; Backspace goes up
  - Point out `try/finally` = "leaves no mess"
  - `/proc`: no scrolling → "That works! But..."
  - Shrink terminal: no resize handling → libraries
- `demo/ratatui_list/target/release/ratatui_list`: resize just works
- `ls -lh` the binary: ~700K punchline
- Diffing slide escape codes are real (captured pressing `j`)

# Q&A ideas

- Accessibility: TUIs worse for screen readers; text there, layout semantics not
- Windows: Windows Terminal is great; crossterm/Bubble Tea/Textual support it
