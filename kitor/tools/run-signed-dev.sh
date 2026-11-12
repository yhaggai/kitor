#!/bin/sh
set -eu

# Match the default bundle's identifier so Keychain recognizes both builds.
/usr/bin/codesign --force --timestamp=none \
  --sign "$KITOR_DEV_SIGNING_IDENTITY" --identifier "$KITOR_DEV_SIGNING_IDENTIFIER" "$1"

exec "$@"
