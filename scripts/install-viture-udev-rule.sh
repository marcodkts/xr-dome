#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  exec sudo "$0" "$@"
fi

RULE_PATH="/etc/udev/rules.d/99-viture-xr-pro.rules"

cat <<'EOF' > "$RULE_PATH"
# VITURE Pro XR Glasses
SUBSYSTEM=="usb", ATTR{idVendor}=="35ca", ATTR{idProduct}=="101d", TAG+="uaccess", GROUP="plugdev", MODE="0660"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="35ca", ATTRS{idProduct}=="101d", TAG+="uaccess", GROUP="plugdev", MODE="0660"
EOF

udevadm control --reload-rules
udevadm trigger

echo "Installed $RULE_PATH"
echo "Reconnect the glasses or log out/in if access is still denied."
