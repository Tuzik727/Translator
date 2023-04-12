#!/bin/bash
rm -f /etc/systemd/system/prekladac.service
echo [Unit] >> /etc/systemd/system/prekladac.service
echo Description=Sitovy Prekladac P2P >> /etc/systemd/system/prekladac.service
echo After=network.target >> /etc/systemd/system/prekladac.service
echo >> /etc/systemd/system/prekladac.service
echo [Service] >> /etc/systemd/system/prekladac.service
echo ExecStart=$(pwd)/alfa4 >> /etc/systemd/system/prekladac.service
echo WorkingDirectory =$(pwd) >> /etc/systemd/system/prekladac.service
echo >> /etc/systemd/system/prekladac.service
echo [Install] >> /etc/systemd/system/prekladac.service
echo WantedBy=default.target >> /etc/systemd/system/prekladac.service
systemctl daemon-reload