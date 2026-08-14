TERMUX_PKG_HOMEPAGE=https://github.com/OfficialBiohub/padma-lang
TERMUX_PKG_DESCRIPTION="Padma Bangla-English programming language"
TERMUX_PKG_LICENSE="MIT"
TERMUX_PKG_MAINTAINER="OfficialBiohub"
TERMUX_PKG_VERSION=0.1.0
TERMUX_PKG_SRCURL="https://github.com/OfficialBiohub/padma-lang/archive/refs/tags/v${TERMUX_PKG_VERSION}.tar.gz"
# Replace this placeholder with the SHA256 of the immutable v${TERMUX_PKG_VERSION} archive before submitting upstream.
TERMUX_PKG_SHA256=REPLACE_WITH_RELEASE_SHA256
TERMUX_PKG_DEPENDS="libc++"
TERMUX_PKG_BUILD_DEPENDS="rust"
TERMUX_PKG_BUILD_IN_SRC=true

termux_step_make() {
    termux_setup_rust
    cargo build --jobs "${TERMUX_PKG_MAKE_PROCESSES}" --target "${CARGO_TARGET_NAME}" --release --locked
}

termux_step_make_install() {
    install -Dm755 target/release/padma "$TERMUX_PREFIX/bin/padma"
}
