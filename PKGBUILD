# Maintainer: gator <gator@example.com>
# Modifica 'url' y 'source' con tu repositorio de Git real antes de subirlo al AUR.

pkgname=gato-music-git
_pkgname=gato-music
pkgver=0.1.0.r0.g8476251
pkgrel=1
pkgdesc="Descargador de música CLI con estética de gato que gestiona yt-dlp e integra letras de LRCLIB (Git)"
arch=('x86_64' 'aarch64')
url="https://github.com/elgatolinux/gato-music" # <-- Reemplaza con tu URL
license=('MIT')
depends=('ffmpeg')
makedepends=('cargo' 'git')
provides=("$_pkgname")
conflicts=("$_pkgname")
source=("$_pkgname::git+${url}.git")
md5sums=('SKIP')

pkgver() {
  cd "$_pkgname"
  ( set -o pipefail
    git describe --long --tags 2>/dev/null | sed 's/\([^-]*-\x1b\)/r\1/;s/-/./g' ||
    printf "0.1.0.r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
  )
}

prepare() {
  cd "$_pkgname"
  export CARGO_HOME="$srcdir/cargo-home"
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "$_pkgname"
  export CARGO_HOME="$srcdir/cargo-home"
  CARGO_PROFILE_RELEASE_LTO=true cargo build --frozen --release
}

package() {
  cd "$_pkgname"
  install -Dm755 "target/release/$_pkgname" "$pkgdir/usr/bin/$_pkgname"
  install -Dm644 "README.md" "$pkgdir/usr/share/doc/$_pkgname/README.md"
}
