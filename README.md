income
===========================

[<img alt="github" src="https://img.shields.io/badge/github-wcampbell0x2a/income-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/wcampbell0x2a/income)
[<img alt="crates.io" src="https://img.shields.io/crates/v/income.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/income)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-income-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/income)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/wcampbell0x2a/income/main.yml?branch=master&style=for-the-badge" height="20">](https://github.com/wcampbell0x2a/income/actions?query=branch%3Amaster)

Library and binary for the reading of [UBI](https://www.kernel.org/doc/html/latest/filesystems/ubifs.html) volumes.

```console
$ ./income [IMAGE_PATH]
```

## Testing
Unit test do not exist for this project yet!
```console
$ wget -L https://github.com/onekey-sec/unblob/raw/1965107de2d813c31ce9d0a28ea47649cd5a81a4/tests/integration/filesystem/ubi/ubi/__input__/fruits.ubi
$ unblob fruits.ubi
$ hexyl fruits.ubi_extract/img-1180426539_vol-data.ubifs
┌────────┬─────────────────────────┬─────────────────────────┬────────┬────────┐
│00000000│ 63 68 65 72 72 79 31 0a ┊ ff ff ff ff ff ff ff ff │cherry1_┊××××××××│
│00000010│ ff ff ff ff ff ff ff ff ┊ ff ff ff ff ff ff ff ff │××××××××┊××××××××│
│*       │                         ┊                         │        ┊        │
│00000380│                         ┊                         │        ┊        │
└────────┴─────────────────────────┴─────────────────────────┴────────┴────────┘
$ cargo run --release -- fruits.ubi
2 named volumes, 3 physical volumes, blocksize=0x400
Extracting volume: "apple"
wrote: ubi-root/img-1180426539_vol-apple.ubifs
Extracting volume: "data"
wrote: ubi-root/img-1180426539_vol-data.ubifs
$ hexyl ubi-root/img-1180426539_vol-data.ubifs
┌────────┬─────────────────────────┬─────────────────────────┬────────┬────────┐
│00000000│ 63 68 65 72 72 79 31 0a ┊ ff ff ff ff ff ff ff ff │cherry1_┊××××××××│
│00000010│ ff ff ff ff ff ff ff ff ┊ ff ff ff ff ff ff ff ff │××××××××┊××××××××│
│*       │                         ┊                         │        ┊        │
│00000380│                         ┊                         │        ┊        │
└────────┴─────────────────────────┴─────────────────────────┴────────┴────────┘
```
