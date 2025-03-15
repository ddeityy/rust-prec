Go use [PeachREC](https://github.com/PapaPeach/PeachREC)

~~# P-REC for linux users and P-REC replacement for windows users~~

~~## Features~~

- Automatically record competitive demos
- Automatically categorize demos - demos/year/year-month/demo.demo
- Better formatting for _events.txt:

  from

    ```text
    >
    [2023/11/08 23:48] Bookmark ("2023-11-08_23-32-45" at 20000)
    [2023/11/08 23:48] Killstreak 4 ("2023-11-08_23-32-45" at 40000)
    [2023/11/08 23:48] Killstreak 5 ("2023-11-08_23-32-45" at 40589)
    >
    ```

  to

    ```text
    >
    [2023-11-05] pl_upward_f10 sniper                                playdemo demos/2023/2023-11/2023-11-05_22-11-34-pl_upward_f10
    [2023-11-05] Killstreak 5 38690-40589 [28.48 seconds]
    [2023-11-05] Bookmark at 41189
    >
    ```

## Installation

Add these lines to your tf/cfg/autoexec.cfg or tf/cfg/overrides/autoexec.cfg for mastercomfig.

```txt
ds_enable 0
ds_log 1
ds_notify 2
ds_sound 1

ip 0.0.0.0
rcon_password prec
net_start
```

Bind ```ds_mark``` for bookmarks

Add this to your TF2 launch options:

```-condebug -conclearlog -usercon```

## Linux

Run

```bash
sudo ./install.sh
```

## Windows

Make it run on startup in any way you want

If you have any ideas for automating this - PRs are welcome

## Building

This project is build using rust and requires `cargo` and friends, see [the rust website](https://www.rust-lang.org/)
for how to get started.

Once rust is setup building is as simple as

```bash
cargo build --release
```

which will place the binary at `target/release/rust-prec`

Then install as a systemd service:

```bash
sudo ./install.sh
```

### Credit for the idea of using a local RCON goes to [@Bash-09](https://github.com/Bash-09)

### Thanks to [@icewind1991](https://github.com/icewind1991/) for help and the [demo parser](https://github.com/demostf/parser)
