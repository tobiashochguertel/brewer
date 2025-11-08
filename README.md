# Brewer 🍺

An extremly fast [homebrew](https://brew.sh/) (macOS & Linux package manager)
CLI wrapper with extra features.

This is a WIP, alpha, early-stage, etc. project. A lot is missing and bugs are
expected.

[![asciicast](https://asciinema.org/a/xHsicw0FhBi0ehVJsuWxvtkWH.svg)](https://asciinema.org/a/xHsicw0FhBi0ehVJsuWxvtkWH)

## Features

- Fuzzy formulae/cask search with an embedded [skim] ([fzf] rust alternative)
- Locate which formulae provides the given binary (Ubuntu's `command-not-found`
  equivalent)
- Much faster than `brew search` (uses [nucleo] crate for non-interactive fuzzy
  search)
- Show plan before installing / uninstall kegs

## Install

```bash
git clone https://github.com/tobiashochguertel/brewer.git
cd brewer

# using cargo
cargo install --path brewer_term --force

# or using `just`
just
```

## Usage

```
Usage: brewer [OPTIONS] <COMMAND>

Commands:
  which      Locate the formulae which provides the given executable
  update     Update the local cache
  list       List installed formulae and casks
  info       Show information about formula or cask
  search     Search for formulae and casks
  paths      Show paths that brewer uses
  exists     Indicate if the given formula or cask exists by exit code
  install    Install the given formula or cask
  uninstall  Uninstall the given formula or cask
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  More output per occurrence
  -q, --quiet...    Less output per occurrence
  -h, --help        Print help
  -V, --version     Print version
```

[fzf]: https://github.com/junegunn/fzf
[nucleo]: https://github.com/helix-editor/nucleo
[skim]: https://github.com/lotabout/skim
