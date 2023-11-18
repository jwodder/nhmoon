[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/nhmoon/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/nhmoon/actions/workflows/test.yml)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.70-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/nhmoon.svg)](https://opensource.org/licenses/MIT)

`nhmoon` is a Rust program for viewing & scrolling through a slice of the
calendar in your terminal.  Days with [new or full moons][moon] in [NetHack][]
are highlighted, though the code can easily be adjusted to highlight based on
different criteria instead.

[moon]: https://nethackwiki.com/wiki/Time#Moon_phase_and_date
[NetHack]: https://www.nethack.org

Key Bindings
============

| Key                                | Command              |
| ---------------------------------- | -------------------- |
| <kbd>j</kbd>, <kbd>Up</kbd>        | Scroll up one week   |
| <kbd>k</kbd>, <kbd>Down</kbd>      | Scroll down one week |
| <kbd>w</kbd>, <kbd>Page Up</kbd>   | Scroll up one page   |
| <kbd>z</kbd>, <kbd>Page Down</kbd> | Scroll down one page |
| <kbd>0</kbd>, <kbd>Home</kbd>      | Jump to today        |
| <kbd>?</kbd>                       | Show help            |
| <kbd>q</kbd>, <kbd>Escape</kbd>    | Quit                 |
