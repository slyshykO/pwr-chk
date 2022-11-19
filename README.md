# pwr-chk


## description

This is a simple Linux utility that checks (by ping) for the accessibility of a given IP address, and if it disappears, _pwr-chk_ calls *power off*.

## usage

```sh
pwr-chk [OPTIONS] --ping-ip <PING_IP>

Options:
  -p, --ping-ip <PING_IP>
  -d, --delay-s <DELAY_S>  [default: 90]
  -h, --help               Print help information
  -V, --version            Print version information
```