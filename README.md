# parsebin

A very simple CLI tool to read and parse binary file and print values as primitive type to standard output.

The tool can be used in conjunction with other tools to process, filter and plot via commandline.

## Usage

```shell
parsebin <file> <type>
```

```shell
parsebin -h
Usage: parsebin.exe [OPTIONS] <TYPE> <FILE>

Arguments:
  <TYPE>  [possible values: u8, u16, u32, u64, i8, i16, i32, i64, f32, f64]
  <FILE>

Options:
  -o, --offset <OFFSET>          [default: 0]
  -n, --number <NUMBER>          [default: 9223372036854775807]
  -b, --byte-order <BYTE_ORDER>  [default: little-endian] [possible values: little-endian, big-endian]
  -h, --help                     Print help
```

## Installation

```shell
cargo install parsebin
```
