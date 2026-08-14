TERMUX_PKG_HOMEPAGE=https://github.com/OfficialBiohub/padma-lang
TERMUX_PKG_DESCRIPTION="Padma Bangla-English programming language"
TERMUX_PKG_LICENSE="MIT"
TERMUX_PKG_MAINTAINER="OfficialBiohub"
TERMUX_PKG_VERSION=0.1.0
TERMUX_PKG_SRCURL=https://github.com/OfficialBiohub/padma-lang/archive/refs/heads/main.tar.gz
TERMUX_PKG_SHA256=SKIP_CHECKSUM
TERMUX_PKG_BUILD_IN_SRC=true

termux_step_make() {
    cargo build --release
}

termux_step_make_install() {
    install -Dm755 target/release/padma "$TERMUX_PREFIX/bin/padma"
}
