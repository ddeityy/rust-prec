#!/usr/bin/bash
echo "Deleting previous install"
sudo rm /home/$SUDO_USER/.local/bin/rust-prec
sudo rm /etc/systemd/system/rust-prec.service
sudo rm /usr/local/bin/prec.sh
echo "Copying the executable and script"
sudo cp target/x86_64-unknown-linux-gnu/release/rust-prec /home/$SUDO_USER/.local/bin/
echo "Creating service file"
sudo touch /etc/systemd/system/rust-prec.service
sudo bash -c 'cat' << EOF > /etc/systemd/system/rust-prec.service
[Unit]
Description=Rust prec service
[Service]
ExecStart=/usr/bin/bash /usr/local/bin/prec.sh
User=$SUDO_USER
[Install]
WantedBy=multi-user.target
EOF
sudo touch prec.sh
sudo bash -c 'cat' << EOF > prec.sh
#!/usr/bin/bash
/home/$SUDO_USER/.local/bin/rust-prec
EOF
sudo cp prec.sh /usr/local/bin/
echo "Reloading systemd daemon"
sudo systemctl daemon-reload
echo "Enabling and starting the service"
sudo systemctl start rust-prec.service
sudo systemctl status rust-prec.service
