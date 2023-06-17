#!/bin/bash
sudo cp target/release/rust-prec /home/deity/.local/bin/
echo "Creating service file"
sudo rm /etc/systemd/system/rust-prec.service
sudo touch /etc/systemd/system/rust-prec.service
sudo bash -c 'cat' << EOF > /etc/systemd/system/rust-prec.service
[Unit]
Description=Rust prec service
[Service]
ExecStart=/home/deity/.local/bin/rust-prec
Restart=on-failure
[Install]
WantedBy=multi-user.target"
EOF
cat /etc/systemd/system/rust-prec.service
