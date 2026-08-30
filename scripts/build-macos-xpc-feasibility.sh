#!/bin/sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "macOS XPC feasibility build is not applicable on this host" >&2
  exit 2
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
source_root="$repository_root/platform/macos-xpc-feasibility"
output_root="$repository_root/target/iar-macos-xpc-feasibility"
app="$output_root/ImpresariContextSandboxProbe.app"
xpc="$app/Contents/XPCServices/studio.boldthaus.impresari-context.SandboxProbe.xpc"

case "$output_root" in
  "$repository_root"/target/iar-macos-xpc-feasibility) ;;
  *) echo "refusing unsafe feasibility output path" >&2; exit 3 ;;
esac

if [ -e "$output_root" ]; then
  rm -rf -- "$output_root"
fi
mkdir -p \
  "$app/Contents/MacOS" \
  "$xpc/Contents/MacOS" \
  "$output_root/bin" \
  "$output_root/module-cache"

export CLANG_MODULE_CACHE_PATH="$output_root/module-cache"
export SWIFT_MODULECACHE_PATH="$output_root/module-cache"

cp "$source_root/Resources/Host-Info.plist" "$app/Contents/Info.plist"
cp "$source_root/Resources/Service-Info.plist" "$xpc/Contents/Info.plist"

xcrun clang \
  -Wall \
  -Wextra \
  -Werror \
  -c "$source_root/Sources/Service/ResourceProbe.c" \
  -o "$output_root/resource-probe.o"

xcrun swiftc \
  -warnings-as-errors \
  -parse-as-library \
  -import-objc-header "$source_root/Sources/Service/ResourceProbe.h" \
  -framework Foundation \
  "$source_root/Sources/Shared/ProbeProtocol.swift" \
  "$source_root/Sources/Service/main.swift" \
  "$output_root/resource-probe.o" \
  -o "$xpc/Contents/MacOS/studio.boldthaus.impresari-context.SandboxProbe"

xcrun swiftc \
  -warnings-as-errors \
  -parse-as-library \
  -framework Foundation \
  "$source_root/Sources/Shared/ProbeProtocol.swift" \
  "$source_root/Sources/Host/main.swift" \
  -o "$app/Contents/MacOS/impresari-context-sandbox-probe"

xcrun swiftc \
  -warnings-as-errors \
  -parse-as-library \
  -framework Foundation \
  "$source_root/Sources/Listener/main.swift" \
  -o "$output_root/bin/synthetic-loopback-listener"

xcrun clang \
  -Wall \
  -Wextra \
  -Werror \
  "$source_root/Sources/DeviceCanary/main.c" \
  -o "$output_root/bin/synthetic-device-canary"

codesign --force --sign - --timestamp=none \
  --entitlements "$source_root/Resources/Service.entitlements" \
  "$xpc"
codesign --force --sign - --timestamp=none \
  --entitlements "$source_root/Resources/Host.entitlements" \
  "$app"

codesign --verify --deep --strict --verbose=2 "$app"
printf '%s\n' "$app"
