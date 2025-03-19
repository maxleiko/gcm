# gcm

A simple GreyCat installer. Think `rustup` but for [GreyCat](https://greycat.io).

## Help
```sh
gcm -h
GreyCat Manager - update, install, discover packages

Usage: gcm <COMMAND>

Commands:
  list     Lists a package branches and/or versions
           eg. gcm list core, gcm list sdk/web testing
  install  Installs a package
  update   Updates the currently installed packages on the same branch
           If no installation found, installs latest 'stable'
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Install
### Latest stable:
```sh
gcm install
```
> installs `core@stable`, `lang@stable` and `explorer@stable`

### Latest of a specific branch:
```sh
gcm install <branch>
```
> `gcm install dev` installs `core@dev`, `lang@dev` and `explorer@dev`

### Specific package in a specific version:
```sh
gcm install <package> <version>
```
> `gcm install explorer 1.2.3-testing` installs `explorer@1.2.3-testing`

### Install the latest package of a specific branch:
```sh
gcm install <package> <branch>
```
> `gcm install core dev` installs `core@dev`


## Update
```sh
gcm update
```
> updates currently installed `<branch>` to latest or install latest `stable`

### List
```sh
gcm list
```
> lists all available packages


# TODOs:
- [ ] Properly handle the new `noarch` for all packages
- [ ] Add a way to `gcm i 6.10` which would try and install `core`, `lang` and `explorer` matching `^6.10`.
      If `lang` or `explorer` is not found, then just log it but as long as `core` exists, it should not be an error