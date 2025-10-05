# Maintainer: windx-foobar <bezalkogoln1ycoder[at]gmail[dot]com>

pkgname=mts-linkchats-launcher
pkgver=0.1.0
pkgrel=1
pkgdesc="Client for mts-linkchats apt repository in Rust for Arch Linux"
url='https://github.com/windx-foobar/mts-linkchats-launcher'
arch=('x86_64')
license=('MIT')
depends=('zenity' 'openssl')
makedepends=('cargo')
backup=('etc/mts-linkchats-launcher.conf')

build() {
  cd ..
  cargo build --release --locked
}

package() {
  cd ..
  # install -Dm 755 -t "${pkgdir}/usr/bin" \
  #   target/release/mts-linkchats-launcher
  #
  # install -Dm644 contrib/mts-linkchats-launcher.desktop -t "${pkgdir}/usr/share/applications"
  # install -Dm644 contrib/icons/linkchats-linux-512.png "${pkgdir}/usr/share/pixmaps/mts-linkchats-launcher.png"
  # install -Dm644 contrib/mts-linkchats-launcher.conf -t "${pkgdir}/etc"
  #
  # for size in 512; do
  #   install -Dm644 "contrib/icons/linkchats-linux-${size}.png" \
  #     "${pkgdir}/usr/share/icons/hicolor/${size}x${size}/apps/mts-linkchats-launcher.png"
  # done
}

# vim: ts=2 sw=2 et:
