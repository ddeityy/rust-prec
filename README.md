# Long awaited "P-REC" for linux users.

### Credit for the idea of using a local RCON goes to [@Bash-09](https://github.com/Bash-09).
### Thanks to [@icewind1991](https://github.com/icewind1991/) for help.

# Usage

Add these lines to your tf/cfg/autoexec.cfg or tf/cfg/overrides/autoexec.cfg in case of mastercomfig.

```
ip 0.0.0.0
rcon_password prec
net_start
```

Add this to your TF2 launch options:

```-condebug -conclearlog -usercon```

Run ```sudo ./install.sh```
