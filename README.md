# fgr - Find Git Repositories

`fgr` is a tiny command line utility which recursively searches a directory for git repositories.
Use it as a building block for shell scripts, terminal aliases and build pipelines. It's designed to
be simple, cross-platform, and as fast as possible.

## Usage

```
$ fgr --help
fgr 0.2
Nick Young
A simple utility for finding git repositories.

USAGE:
    fgr.exe [FLAGS] [OPTIONS] [PATH]

FLAGS:
    -a, --all                Do not ignore directories starting with `.`
        --any-depth          Drops the default max-depth limit, allowing unlimited depth
    -h, --help               Prints help information
    -p, --paranoid           Be extra certain that a directory is a git repository.
    -s, --follow-symlinks    Follow symlinks rather than ignoring them
    -V, --version            Prints version information
    -v, --verbose            Output detailed messages to standard error

OPTIONS:
    -d, --max-depth <max-depth>    Sets the maximum depth when recursively scanning subdirectories [default: 10]

ARGS:
    <PATH>    The directory where the search will begin
```

## Versus `find` or `fd`

`fgr` is a simple app, and it's functionality can be replicated using tools that you may already
have installed on your system.

```bash
# Invoking fgr with default options on the current directory
$ fgr

# This is the (roughly) equivalent operation using GNU find
# (Note that this does not return the absolute path of the repository)
$ find . -name .git -type d -prune | sed -e s/\.git$//

# This is the (roughly) equivalent operation using fd
$ fd --type d --glob --absolute-path --hidden --prune ".git" --max-depth 10 . | sed -e s/\.git$//
```

([fd](https://github.com/sharkdp/fd) is a great tool which I highly recommend)

### Built for purpose

Despite being very simple, `fgr` will normally outperform `find`, `fd`, and other similar
general-purpose tools. This is due to its ability to exploit some known properties of git
repositories:

1. Once it has determined that a directory is a git repository, it won't continue to recursively
   search its sub-directories.
2. It will optimistically check if a directory is a git repository before enumerating all of its
   contents.
3. It can handle that `.git` is itself a hidden directory, without having to enumerate all hidden
   directories while searching.

Additionally, the default options for `fgr` have been carefully chosen to match its typical use
cases.

1. It does not traverse hidden directories by default.
   - Disable this with the `-a/--all` flag
2. It does not traverse symlinks by default.
   - Disable this with the `-s/--follow-symlinks` flag
3. It does not traverse deeper than 10 levels of sub-directories by default.
   - Disable this with `--any-depth`, or choose a different maximum with `-d/--max-depth`

## Paranoid Mode

In the unlikely event your filesystem contains directories named `.git` which do not belong to a git
repository, you can enable the `-p/--paranoid` mode in `fgr`. Doing so will cause an additional
check to be performed on each discovered repository to make sure it truly is a git repository.

Specifically, `fgr` will try to invoke `git rev-parse HEAD` in each discovered repository, and it
will expect that command to complete successfully. Note that `fgr` avoids `git status`, because it
is potentially a slow operation in large repositories. One consequence of this is repositories with
zero commits will fail the paranoid check (they have no `HEAD`).

`fgr` does not bundle any `git` functionality. Using `--paranoid` requires you to have `git`
installed on your system.

## Verbose Mode

TODO

## Roadmap

- Build pipeline
- Package for: Homebrew, Chocolatey, NixOS, various Linux/BSD package managers
