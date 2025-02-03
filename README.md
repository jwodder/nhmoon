[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/nhmoon/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/nhmoon/actions/workflows/test.yml)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.74-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/nhmoon.svg)](https://opensource.org/licenses/MIT)

`nhmoon` is a Rust program for viewing & scrolling through a slice of the
calendar in your terminal.  Days with [new or full moons][moon] in [NetHack][]
are highlighted, though the code can easily be adjusted to use different
highlighting criteria instead.

[moon]: https://nethackwiki.com/wiki/Time#Moon_phase_and_date
[NetHack]: https://www.nethack.org

Screenshot of the program on startup on 2023 November 18:

![Screenshot of the program](screenshot.png)

Usage
=====

    nhmoon [<date>]

Opens a view of a proleptic Gregorian calendar centered on the given date, or
centered on the current date if no date is given.  Dates are given in the form
`YYYY-MM-DD` using [astronomical year numbering][years].  Only dates from
10,000 BC (-9999 in astronomical year numbering) through 9,999 AD are
supported.

[years]: https://en.wikipedia.org/wiki/Astronomical_year_numbering

Key Bindings
------------

| Key                                | Command              |
| ---------------------------------- | -------------------- |
| <kbd>j</kbd>, <kbd>Up</kbd>        | Scroll up one week   |
| <kbd>k</kbd>, <kbd>Down</kbd>      | Scroll down one week |
| <kbd>w</kbd>, <kbd>Page Up</kbd>   | Scroll up one page   |
| <kbd>z</kbd>, <kbd>Page Down</kbd> | Scroll down one page |
| <kbd>0</kbd>, <kbd>Home</kbd>      | Jump to today        |
| <kbd>?</kbd>                       | Show help            |
| <kbd>q</kbd>, <kbd>Escape</kbd>    | Quit                 |
