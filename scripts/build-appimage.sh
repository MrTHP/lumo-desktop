#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../src-tauri"

# Fedora ships the libappindicator runtime but the -devel package (and its
# pkg-config file) is sometimes absent. Tauri's AppImage bundler only needs the
# library path, so provide a minimal .pc file when the runtime library exists.
if ! pkg-config --exists appindicator3-0.1 2>/dev/null && [ -f /usr/lib64/libappindicator3.so.1 ]; then
  mkdir -p "$HOME/.local/share/pkgconfig"
  cat > "$HOME/.local/share/pkgconfig/appindicator3-0.1.pc" <<'EOF'
prefix=/usr
exec_prefix=${prefix}
libdir=/usr/lib64
includedir=/usr/include

Name: appindicator3
Description: AppIndicator and KStatusNotifierItem support
Version: 12.10.1
Libs: -L${libdir} -lappindicator3
Cflags: -I${includedir}
EOF
  export PKG_CONFIG_PATH="$HOME/.local/share/pkgconfig:${PKG_CONFIG_PATH:-}"
fi

cargo tauri build --bundles appimage
