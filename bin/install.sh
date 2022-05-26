#!/bin/bash

cp ap_scanner /usr/local/bin/
cp ap_scanner.service /etc/systemd/system/
systemctl enable ap_scanner.service