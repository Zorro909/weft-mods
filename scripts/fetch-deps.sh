#!/usr/bin/env bash
# For hosts that can't install alsa-lib-devel (Fedora Atomic: Kinoite, Silverblue): fetches the
# Fedora package matching the installed alsa-lib and unpacks it into .deps/alsa, which
# scripts/build.sh then uses. Nothing outside this repository changes.
set -euo pipefail
cd "$(dirname "$0")/.."

ver=$(rpm -q --qf '%{VERSION}-%{RELEASE}' alsa-lib)
fedora=$(rpm -E %fedora)
dir=.deps/alsa
mkdir -p .deps/dl "$dir"
for repo in "updates/$fedora/Everything/x86_64/Packages/a" "releases/$fedora/Everything/x86_64/os/Packages/a"; do
    url="https://dl.fedoraproject.org/pub/fedora/linux/$repo/alsa-lib-devel-$ver.x86_64.rpm"
    if curl -sfo .deps/dl/alsa-lib-devel.rpm "$url"; then break; fi
done
rpm -K --nosignature .deps/dl/alsa-lib-devel.rpm
(cd "$dir" && rpm2cpio ../dl/alsa-lib-devel.rpm | cpio -idm --quiet)
ln -sf /usr/lib64/libasound.so.2 "$dir/usr/lib64/libasound.so"
abs="$PWD/$dir/usr"
sed -i "s|^prefix=.*|prefix=$abs|; s|^exec_prefix=.*|exec_prefix=$abs|; s|^libdir=.*|libdir=$abs/lib64|; s|^includedir=.*|includedir=$abs/include|" "$dir/usr/lib64/pkgconfig/alsa.pc"
echo "ALSA development files in $dir"
