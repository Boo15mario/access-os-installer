pkgname=access-os-installer-cli
pkgver=1.0.0
pkgrel=1
pkgdesc="Accessible CLI installer for access-OS"
arch=('x86_64')
url="https://github.com/Boo15mario/access-os-installer"
license=('custom')
depends=('sudo' 'polkit' 'networkmanager' 'arch-install-scripts')
makedepends=('cargo')
source=(
  "$pkgname::git+https://github.com/Boo15mario/access-os-installer.git"
)
sha256sums=('SKIP')

build() {
  cd "$srcdir/$pkgname"
  cargo build --release -p access-os-installer-cli
}

package() {
  cd "$srcdir/$pkgname"

  install -Dm755 "target/release/access-os-installer-cli" \
    "$pkgdir/usr/share/access-os-installer/install-access-real"
  install -Dm755 "packaging/install-access" \
    "$pkgdir/usr/bin/install-access"

  find profiles -type f -name '*.txt' -print0 | while IFS= read -r -d '' file; do
    install -Dm644 "$file" "$pkgdir/usr/share/access-os-installer/$file"
  done

  install -Dm644 README.md \
    "$pkgdir/usr/share/doc/$pkgname/README.md"
}
