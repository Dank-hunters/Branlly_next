#!/usr/bin/env bash
set -euo pipefail

REPOSITORY="Dank-hunters/Branlly_next"
API="https://api.github.com/repos/$REPOSITORY/releases/latest"
INSTALL_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/branlly-next"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"

if [[ $(uname -s) != Linux ]]; then
  echo "Cet installateur est réservé à Linux." >&2
  exit 1
fi
if [[ $(uname -m) != x86_64 ]]; then
  echo "Architecture non prise en charge : $(uname -m). Version x86_64 requise." >&2
  exit 1
fi
for command in curl sha256sum; do
  command -v "$command" >/dev/null || { echo "Commande requise absente : $command" >&2; exit 1; }
done

release_json=$(curl --fail --silent --show-error --location "$API")
url=$(printf '%s' "$release_json" | grep 'browser_download_url' | cut -d '"' -f 4 | grep -Ei '(amd64|x86_64).*\.AppImage$' | head -n 1)
if [[ -z ${url:-} ]]; then
  echo "Aucun AppImage Linux trouvé dans la dernière release." >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR" "$BIN_DIR" "$DESKTOP_DIR"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
filename=$(basename "$url")
curl --fail --show-error --location "$url" --output "$temporary/$filename"
curl --fail --show-error --location "$url.sha256" --output "$temporary/$filename.sha256"
(cd "$temporary" && sha256sum --check "$filename.sha256")

install -m 0755 "$temporary/$filename" "$INSTALL_DIR/branlly-next.AppImage"
ln -sfn "$INSTALL_DIR/branlly-next.AppImage" "$BIN_DIR/branlly-next"
cat > "$DESKTOP_DIR/branlly-next.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Branlly Next
Comment=Assistant de bureau local
Exec=$INSTALL_DIR/branlly-next.AppImage
Terminal=false
Categories=Utility;
StartupNotify=false
EOF
chmod 0644 "$DESKTOP_DIR/branlly-next.desktop"

echo "Branlly Next est installé. Lancez 'branlly-next' ou utilisez le menu des applications."
missing=()
for command in wmctrl xdotool nmcli bluetoothctl lsusb; do
  command -v "$command" >/dev/null || missing+=("$command")
done
if (( ${#missing[@]} > 0 )); then
  echo "Fonctions système optionnelles absentes : ${missing[*]}"
  echo "Sur Ubuntu/Debian : sudo apt install wmctrl xdotool network-manager bluez usbutils"
fi
