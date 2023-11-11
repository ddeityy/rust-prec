#!/usr/bin/bash
echo "Copying the executable"
cp target/release/rust-prec /home/$SUDO_USER/.local/share/
echo "Creating service file"
sudo touch /etc/systemd/user/rust-prec.service
sudo bash -c 'cat' << EOF > /etc/systemd/user/prec.service
[Unit]
Description=Rust prec service
[Service]
ExecStart=/home/$SUDO_USER/.local/share/rust-prec
Restart=on-failure
[Install]
WantedBy=default.target
EOF
echo "Reloading systemd daemon"
systemctl --user -M $SUDO_USER@ daemon-reload
echo "Enabling and starting the service"
echo ------------------------------------
echo | cat /etc/systemd/user/prec.service
echo ------------------------------------
systemctl --user -M $SUDO_USER@ enable --now prec.service
systemctl --user -M $SUDO_USER@ status prec.service
