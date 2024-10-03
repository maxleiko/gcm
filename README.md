# Install
```sh
gcm install
```
=> installs `core@stable`, `lang@stable` and `apps/explorer@stable`

```sh
gcm install dev
```
=> installs `core@dev`, `lang@dev` and `apps/explorer@dev`

```sh
gcm install apps/explorer 1.2.3-testing
```
=> installs `apps/explorer@1.2.3-testing`

```sh
gcm install core dev
```
=> installs `core@dev`

```sh
gcm install lang testing
```
=> installs `lang@testing`

### Update
```sh
gcm update
```
=> updates currently installed `<branch>` to latest or install latest `stable`

### List
```sh
gcm list
```
=> lists all available packages


# TODOs:
- [ ] Add a way to `gcm i 6.10` which would try and install `core`, `lang` and `apps/explorer` matching `^6.10`.
      If `lang` or `explorer` is not found, then just log it but as long as `core` exists, it should not be an error