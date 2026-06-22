# Maintainer: measles <denkori80@gmail.com>
pkgname=mcal
pkgver=0.8.0
pkgrel=1
pkgdesc="A modern, fast, and standalone CLI calendar utility written in Rust with borders, interactive navigation, todo txt integration and localization support"
arch=('x86_64' 'aarch64')
url="https://github.com/1mesles1/mcal"
license=('GPL3')
depends=('gcc-libs')
makedepends=('rust')
source=("git+$url.git")
sha256sums=('SKIP')

prepare() {
  cd "$pkgname"
  cargo fetch --target "$CARCH-unknown-linux-gnu"
}

build() {
  cd "$pkgname"
  cargo build --release --offline
}

package() {
  cd "$pkgname"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}

