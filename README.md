# mts-linkchats-launcher-launcher

TBD

## Configuration

mts-linkchats-launcher-launcher is going to look for a configuration file at the following locations (in this order):

- `${XDG_CONFIG_HOME:-$HOME/.config}/mts-linkchats-launcher-launcher.conf`
- `/etc/mts-linkchats-launcher-launcher.conf`

If no config is found it's going to start with default settings. Your configuration file may look like this:

```toml
[mts-linkchats]
## Pass extra arguments to the mts-linkchats executable
## You can test this with `mts-linkchats-launcher-launcher -v --skip-update --no-exec`
#extra_arguments = []
[launcher]
## Check update
#check_update = true
## How often to try to resume the download until giving up (0 for unlimited)
#download_attempts = 5
```

## License

MIT
