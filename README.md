# Long awaited "prec" for linux users.

### Credit for the idea of using a local RCON goes to [@Bash-09](https://github.com/Bash-09)
<br>

// TODO
- :white_large_square: Convert config.toml to an external file 
- :white_large_square: Clear console.log in between rounds/games

# Usage

Add these lines to your autoexec.cfg

```
ip 0.0.0.0
rcon_password prec
net_start
```

Add these to your launch options:

```-condebug -conclearlog -usercon```

Run TF2, when you boot into the main menu - run ./rust-prec

Once you see your console is being mirrored in the output - you're all set.

When the server unloads SOAP DM it will start recording.
When demos.tf finishes uploading it will stop recording.
