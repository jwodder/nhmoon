[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/nhmoon/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/nhmoon/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/nhmoon/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/nhmoon)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.86-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/nhmoon.svg)](https://opensource.org/licenses/MIT)
[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

`nhmoon` is a [Rust](https://www.rust-lang.org) program for viewing & scrolling
through a slice of the calendar in your terminal.  Days with [new or full
moons][moon] in [NetHack][] are highlighted, though the code can easily be
adjusted to use different highlighting criteria instead.

[moon]: https://nethackwiki.com/wiki/Time#Moon_phase_and_date
[NetHack]: https://www.nethack.org

Screenshot of the program on startup on 2025 February 4:

![Screenshot of the program](screenshot.png)

Installation
============

In order to install `nhmoon`, you first need to have [Rust and Cargo
installed](https://www.rust-lang.org/tools/install).  You can then build the
latest version of `nhmoon` and install it in `~/.cargo/bin` by running:

    cargo install --git https://github.com/jwodder/nhmoon

Usage
=====

    nhmoon [<date>]

Opens a view of a proleptic Gregorian calendar centered on the given date, or
centered on the current date if no date is given.  Dates are given in the form
`YYYY-MM-DD` using [astronomical year numbering][years].  Only dates from
10,000 BC (-9999 in astronomical year numbering) through 9,999 AD are
supported.

[years]: https://en.wikipedia.org/wiki/Astronomical_year_numbering

The calendar highlights dates of NetHack full moons in bold yellow and new
moons in bright blue.

Key Bindings
------------

| Key                                | Command                 |
| ---------------------------------- | ----------------------- |
| <kbd>k</kbd>, <kbd>Up</kbd>        | Scroll up one week      |
| <kbd>j</kbd>, <kbd>Down</kbd>      | Scroll down one week    |
| <kbd>w</kbd>, <kbd>Page Up</kbd>   | Scroll up one page      |
| <kbd>z</kbd>, <kbd>Page Down</kbd> | Scroll down one page    |
| <kbd>0</kbd>, <kbd>Home</kbd>      | Jump to today           |
| <kbd>g</kbd>                       | Input a date to jump to |
| <kbd>?</kbd>                       | Show help               |
| <kbd>q</kbd>, <kbd>Escape</kbd>    | Quit                    |

Pressing <kbd>g</kbd> brings up an input prompt for entering a date in the form
`YYYY-MM-DD`.  (Enter digits only; the hyphens are filled in for you.)
Pressing <kbd>-</kbd> or <kbd>+</kbd> at the beginning of the prompt changes
the sign of the year.  Pressing <kbd>g</kbd>, <kbd>q</kbd>, or
<kbd>Escape</kbd> at any point while editing dismisses the prompt.  After
entering eight digits, press <kbd>Enter</kbd> to jump to the given date.
