Kitor ships innoextract 1.9 by Daniel Scharrer under the zlib license. The
third-party notices in `licenses/` are from the official Windows release.

- [Upstream source and Windows release](https://constexpr.org/innoextract/files/innoextract-1.9/)
- Source archive SHA-256: `6344a69fc1ed847d4ed3e272e0da5998948c6b828cb7af39c6321aba6cf88126`
- Windows ZIP SHA-256: `6989342c9b026a00a72a38f23b62a8e6a22cc5de69805cf47d68ac2fec993065`

The Windows executable is the upstream 32-bit build, which runs on supported
Windows x64 systems. The two macOS executables are built from the same source for
Apple Silicon and Intel. Each executable links Boost statically and uses macOS system
libraries for LZMA, bzip2, zlib and character conversion.

To rebuild the Mac tool, use CMake 3.31, the Xcode command-line tools,
[Boost 1.85.0](https://archives.boost.io/release/1.85.0/source/boost_1_85_0.tar.bz2)
(SHA-256 `7009fe1faa1697476bdc7027703a2badb84e849b7b0baad5086b087b971f8617`),
and XZ development headers. Run the following from the extracted Boost source.

```sh
./bootstrap.sh --with-libraries=filesystem,iostreams,date_time,program_options,system
MACOSX_DEPLOYMENT_TARGET=11.0 ./b2 variant=release link=static threading=multi architecture=arm+x86 \
  address-model=64 -sNO_LZMA=1 -sNO_ZSTD=1 \
  --stagedir=stage-fat stage
```

From the extracted innoextract source, set `BOOST_SOURCE` to the absolute path
to the Boost source and `XZ_HEADERS` to the directory containing `lzma.h`.
Add `#include <boost/filesystem/directory.hpp>` to `src/stream/slice.cpp` after
the existing Boost filesystem include. Boost 1.85 requires this explicit
include for directory iteration. Then build and verify the executables.

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  '-DCMAKE_OSX_ARCHITECTURES=arm64;x86_64' -DCMAKE_OSX_DEPLOYMENT_TARGET=11.0 \
  -DBoost_NO_BOOST_CMAKE=ON -DBoost_USE_STATIC_LIBS=ON \
  -DBOOST_ROOT="$BOOST_SOURCE" -DBOOST_LIBRARYDIR="$BOOST_SOURCE/stage-fat/lib" \
  -DLZMA_INCLUDE_DIR="$XZ_HEADERS" -DLZMA_LIBRARY="$(xcrun --show-sdk-path)/usr/lib/liblzma.tbd"
cmake --build build --parallel
for target in arm64:aarch64 x86_64:x86_64; do
  lipo build/innoextract -thin "${target%:*}" -output "innoextract-${target#*:}-apple-darwin"
  tool="innoextract-${target#*:}-apple-darwin"
  install_name_tool -delete_rpath "$BOOST_SOURCE/stage-fat/lib" "$tool"
  codesign --force --sign - "$tool"
  otool -L "$tool"
done
```
