
# Stock gVim instance and Summon it

Build and Install

```
$ cargo install panty
$ ~/.cargo/bin/panty -h
```


# Usage

## Start gVim instance stocker

```
$ panty collector --stocks 5 --watch ~/.vimrc --watch ~/.gvimrc
```

**collector** stocks 5 gVim instances, and watch 2 files for renewal these instances.


## Summon stocked gVim instance

```
$ panty summon /etc/fstab /etc/whois.conf
```

Brings the stocked gVim instance to current desktop, then opens `/etc/fstab` and `/etc/whois.conf`.

## Renewal stocked gVim instance

```
$ panty renew
```

## Output logs

```
$ RUST_LOG=trace panty ...
```


# Requirements

- gVim
- Linux
- inotify
- X11
