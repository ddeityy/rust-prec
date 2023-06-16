# Long awaited "prec" for linux users.

### Credit for the idea of using a local RCON goes to [@Bash-09](https://github.com/Bash-09).
### Thanks to [@icewind1991](https://github.com/icewind1991/) for help.

# Usage

Add these lines to your autoexec.cfg

```
ip 0.0.0.0
rcon_password prec
net_start
```

Add these to your launch options:

```-condebug -conclearlog -usercon```

Run ```install.sh```

Run ```sudo systemctl enable --now prec.service```

Check with ```systemctl status prec.service```