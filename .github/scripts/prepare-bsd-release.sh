#!/bin/sh
set -eu
# Native OS tools only; package-manager curl never comes from a pipeline environment.
case "$(uname -s)" in
    FreeBSD)
        pkg install -y bash git pkgconf curl
        pkg install -y sccache || echo 'sccache unavailable; using Cargo artifact cache'
        ;;
    NetBSD)
        export PATH=/usr/pkg/bin:/usr/pkg/sbin:/usr/bin:/usr/sbin:/bin:/sbin
        if [ ! -x /usr/pkg/bin/pkgin ]; then /usr/sbin/pkg_add pkgin; fi
        repositories=$(sed -n 's/^PKG_PATH=//p' /etc/pkg_install.conf)
        test -n "$repositories"
        printf '%s\n' "$repositories" | tr ';' '\n' > /usr/pkg/etc/pkgin/repositories.conf
        /usr/pkg/bin/pkgin -y update
        /usr/pkg/bin/pkgin -y install bash git pkgconf curl xz
        /usr/pkg/bin/pkgin -y install sccache || echo 'sccache unavailable; using Cargo artifact cache'
        ;;
    OpenBSD)
        pkg_add -I bash rust git pkgconf curl
        pkg_add -I sccache || echo 'sccache unavailable; using Cargo artifact cache'
        ;;
    *) echo 'This script requires a native BSD guest' >&2; exit 1 ;;
esac
