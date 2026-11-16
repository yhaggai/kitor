This macOS xdelta3 3.2.0 binary is built from the [upstream release source](https://github.com/jmacd/xdelta/tree/v3.2.0/xdelta3) under the included Apache 2.0 license. Source archive SHA-256: `ba2c9676b325f1958e504a60a20340145b8073d5f8664092de17389e15a93199`.

To rebuild, extract the release source and run the following from its `xdelta3` directory. Set `XZ_HEADERS` to the directory containing the liblzma headers, such as `$(brew --prefix xz)/include`. Only the headers are used from that directory. The executable links the macOS SDK's system liblzma.

```sh
xcrun clang -arch arm64 -arch x86_64 -mmacosx-version-min=10.13 \
  -isysroot "$(xcrun --show-sdk-path)" -O2 \
  -DSIZEOF_SIZE_T=8 -DSIZEOF_UNSIGNED_INT=4 \
  -DSIZEOF_UNSIGNED_LONG=8 -DSIZEOF_UNSIGNED_LONG_LONG=8 \
  -DHAVE_LZMA_H=1 -DXD3_MAIN=1 -DSECONDARY_DJW=1 -DSECONDARY_FGK=1 \
  -DEXTERNAL_COMPRESSION=0 -DREGRESSION_TEST=0 \
  -I"$XZ_HEADERS" xdelta3.c -llzma -o xdelta3
otool -L xdelta3
```
