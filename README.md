# mts-linkchats-launcher-launcher

Application to launch the latest version of [MTS LinkChats](https://help.mts-link.ru/article/19352?ubtcuid=2cd8323d-b798-4c39-bd40-221c94f8ed01&currentURL=https%3A%2F%2Fmts-link.ru%2Fapplication%2F&referrerURL=)

## Configuration

mts-linkchats-launcher-launcher is going to look for a configuration file at the following locations (in this order):

- `${XDG_CONFIG_HOME:-$HOME/.config}/mts-linkchats-launcher-launcher.conf`
- `/etc/mts-linkchats-launcher-launcher.conf`

If no config is found it's going to start with default settings.
Your configuration file may look like this:

```toml
[mts-linkchats]
## Pass extra arguments to the mts-linkchats executable
## You can test this with `mts-linkchat-launcher`
#extra_arguments = []
[launcher]
## Should check update [default = true]
#check_update = true
## How often to try to resume the download until giving up (0 for unlimited) [default = 5]
#download_attempts = 5
## How often do you need to check for updates (seconds) [default = 1 day]
#update_check_interval = 86400
```

## License

MIT
