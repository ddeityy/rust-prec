#!/usr/bin/bash
echo "Deleting old binary"
rm /home/$SUDO_USER/.local/bin/rust-prec
echo "Copying binary"
cp target/release/rust-prec /home/$SUDO_USER/.local/bin/
echo "Creating service file"
sudo touch /etc/systemd/user/prec.service
sudo bash -c 'cat' << EOF > /etc/systemd/user/prec.service
[Unit]
Description=Rust prec service
[Service]
ExecStart=/home/$SUDO_USER/.local/bin/rust-prec
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
