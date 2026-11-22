#!/usr/bin/env python3
"""A TUI with zero dependencies: just escape codes and raw mode."""
import os, sys, termios, tty

def entries(path):                                  # "..", then folders, then files
    try:
        names = [n + "/" if os.path.isdir(os.path.join(path, n)) else n
                 for n in os.listdir(path)]
    except OSError:                                 # e.g. permission denied
        names = []
    return [".."] + sorted(names, key=lambda n: (not n.endswith("/"), n))

def draw(path, items, selected):
    out = "\x1b[H\x1b[2J"                          # cursor home, clear screen
    out += f"\x1b[1;35m  {path}\x1b[0m\r\n\r\n"    # bold magenta title
    for i, item in enumerate(items):
        if i == selected:
            out += f"\x1b[7m > {item} \x1b[0m\r\n"  # reverse video
        else:
            out += f"   {item}\r\n"
    out += "\r\n\x1b[2m  j/k move, Enter open, Backspace up, q quit\x1b[0m"
    sys.stdout.write(out)
    sys.stdout.flush()

def main():
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    sys.stdout.write("\x1b[?1049h\x1b[?25l")       # alt screen, hide cursor
    try:
        tty.setraw(fd)                              # no echo, no line buffering
        path, selected = os.getcwd(), 0
        while True:
            items = entries(path)
            draw(path, items, selected)             # DRAW
            key = os.read(fd, 3)                    # WAIT (arrow keys are 3 bytes)
            if key in (b"q", b"\x03"):              # q or Ctrl-C
                break
            if key in (b"j", b"\x1b[B"):            # UPDATE
                selected = min(selected + 1, len(items) - 1)
            if key in (b"k", b"\x1b[A"):
                selected = max(selected - 1, 0)
            if key in (b"\r", b"l") and items[selected].endswith(("/", "..")):
                path = os.path.normpath(os.path.join(path, items[selected]))
                selected = 0
            if key in (b"\x7f", b"h"):              # Backspace
                path, selected = os.path.dirname(path), 0
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)
        sys.stdout.write("\x1b[?25h\x1b[?1049l")   # show cursor, main screen
        sys.stdout.flush()
    print(f"You picked: {os.path.normpath(os.path.join(path, items[selected]))}")

if __name__ == "__main__":
    main()
