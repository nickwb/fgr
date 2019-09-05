# fgr - Find Git Repositories
`fgr` is a tiny command line utility which recursively searches a directory for git repositories. Use it as a building block for shell scripts, terminal aliases and build pipelines. It's designed to be a simple, cross-platform, and as fast as possible.

## Usage

```
$ fgr --help
fgr 0.1
Nick Young
A simple utility for finding git repositories.

USAGE:
    fgr [FLAGS] [PATH]

FLAGS:
    -a, --all                Do not ignore directories starting with `.`
    -h, --help               Prints help information
    -p, --paranoid           Be extra certain that a directory is a git repository.
    -s, --follow-symlinks    Follow symlinks rather than ignoring them
    -V, --version            Prints version information
    -v, --verbose            Output detailed messages to standard error

ARGS:
    <PATH>    The directory where the search will begin
```


## Roadmap
- Implement max search depth
- Package for: Homebrew, Chocolatey, NixOS, various Linux/BSD package managers