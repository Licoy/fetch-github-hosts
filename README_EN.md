[简体中文](./README.md) | English

<div align="center">
<h2>Fetch GitHub Hosts</h2>

![LOGO](assets/public/logo.png)

`fetch-github-hosts` is a `Github Hosts` synchronization tool mainly provided to solve the problem of slow access to `Github` or other problems for research and learning personnel.

[![Release](https://img.shields.io/github/v/release/Licoy/fetch-github-hosts.svg?logo=git)](https://github.com/Licoy/fetch-github-hosts)
[![Build Linux & Windows](https://github.com/Licoy/fetch-github-hosts/workflows/Build%20for%20Linux%20&%20Windows/badge.svg)](https://github.com/Licoy/fetch-github-hosts)
[![Build MacOS](https://github.com/Licoy/fetch-github-hosts/workflows/Build%20for%20MacOS/badge.svg)](https://github.com/Licoy/fetch-github-hosts)

</div>

## no-gui build

```bash
go build -tags="no_gui"
```

## Principle

This project obtains the `hosts` of `github.com` by deploying the server of the project itself, rather than through a third-party IP address interface, such as `ipaddress.com`, etc.

## Instructions
### Graphical interface
Go to [Releases](https://github.com/Licoy/fetch-github-hosts/releases) to download your system version (currently supports `Windows`/`Linux`/`MacOS`
)

After the download is completed, unzip the `tar.gz` compressed package and run the executable file of the corresponding platform to run (⚠️ Note: Linux needs to be started with `sudo`, Windows and MacOS will automatically perform privilege escalation operations.)

#### Client mode
![client](assets/public/docs/client.png)

#### Client startup
![client-start](assets/public/docs/client-start.png)

#### Client hosts source selection
![client-select](assets/public/docs/client-select.png)

#### Client hosts are derived from customization
![client-custom](assets/public/docs/client-custom.png)

#### Server mode
![server](assets/public/docs/server.png)

### Command line terminal

Go to [Releases](https://github.com/Licoy/fetch-github-hosts/releases) to download your system version (currently supports `Windows`/`Linux`/`MacOS`
)

#### Parameters

| Parameter name | Abbreviation | Default value                        | Required | Description                                            |
|----------------|--------------|--------------------------------------|----------|--------------------------------------------------------|
| `mode`         | `m`          | None                                 | Yes      | Startup mode `server` / `client`                       |
| `interval`     | `i`          | 60                                   | No       | Get the record value interval (minutes)                |
| `port`         | `p`          | 9898                                 | No       | Service mode listening port to access the HTTP service |
| `url`          | `u`          | `https://hosts.gitcdn.top/hosts.txt` | No       | Client mode remote hosts get link                      |
| `lang`         | `l`          | `zh-CN`                              | No       | Interface language                                     |

#### Start the client:

> Note:
>
> You need to use `sudo` to run under Linux;
>
> Windows and MacOS will automatically perform privilege escalation operations.

- run directly

```bash
#Linux/Macos
sudo fetch-github-hosts -m=client

# Windows
fetch-github-hosts.exe -m=client
```

- Customize the acquisition time interval

```bash
# Linux/Macos (obtained once every 10 minutes)
sudo fetch-github-hosts -i=10

# Windows (obtained once every 10 minutes)
fetch-github-hosts.exe -i=10
```

- Customized get link

```bash
#Linux/Macos
sudo fetch-github-hosts -u=http://127.0.0.1:9898/hosts.json

# Windows
fetch-github-hosts.exe -u=http://127.0.0.1:9898/hosts.json
```

#### Start the server:

- run directly

```bash
#Linux/Macos
fetch-github-hosts -m=server

# Windows
fetch-github-hosts.exe -m=server
```

- Custom listening port

```bash
#Linux/Macos
fetch-github-hosts -m=server -p=6666

# Windows
fetch-github-hosts.exe -m=server -p=6666
```

### Manual

#### Add hosts

Visit [https://hosts.gitcdn.top/hosts.txt](https://hosts.gitcdn.top/hosts.txt) ,
Paste its entire contents into your hosts file.

- `Linux/MacOS` hosts path: `/etc/hosts`
- `Windows` hosts path: `C:\Windows\System32\drivers\etc\hosts`

#### Refresh takes effect

- `Linux`: `/etc/init.d/network restart`
- `Windows`: `ipconfig /flushdns`
- `Macos`: `sudo killall -HUP mDNSResponder`

#### Unix/Linux one-click use

```shell
sed -i "/# fetch-github-hosts begin/Q" /etc/hosts && curl https://hosts.gitcdn.top/hosts.txt >> /etc/hosts
```

> Tip: You can set up a crontab scheduled task to get updates regularly, freeing your hands!

## Private deployment

Download the latest release (go to [Releases](https://github.com/Licoy/fetch-github-hosts/releases) to download)
, and select the corresponding version of your system, and run it directly in service mode: `fetch-github-hosts -m=server -p=9898`, which will automatically monitor `0.0.0.0:9898`, and you can access it directly with a browser `http://127.0.0.1:9898`
to access your customized services.
(For specific methods, please refer to the section [Start Server] for detailed instructions)

> Note: Due to network influence, try to deploy to overseas server nodes!

## Trend
[![Stargazers over time](https://starchart.cc/Licoy/fetch-github-hosts.svg)](https://starchart.cc/Licoy/fetch-github-hosts)

## Open Source Agreement

[GPL 3.0](https://github.com/Licoy/fetch-github-hosts/blob/main/LICENSE)