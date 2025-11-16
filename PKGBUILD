# Maintainer: Origin173 <https://github.com/Origin173>

pkgname=xmcl-bin
pkgdesc='A Minecraft launcher forked from SJMCL'
pkgver=0.1.0
pkgrel=1
arch=(x86_64)
license=(GPL-3.0)
url='https://github.com/Origin173/XMCL'
source=("https://github.com/Origin173/XMCL/releases/download/v${pkgver}/XMCL_${pkgver}_linux_x86_64.deb")
sha512sums=('SKIP')
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'libsoup' 'pango' 'webkit2gtk-4.1')
options=('!strip' '!emptydirs')
provides=('xmcl')
conflicts=('xmcl')

package() {
  bsdtar -xf data.tar.gz -C "${pkgdir}"
  chmod +x ${pkgdir}/usr/bin/XMCL
}