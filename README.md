[![Build Status][azure-badge]][azure-url]
[![MIT licensed][license-badge]][license-url]
[![dependency status][dependency-badge]][dependency-url]

## About

A simple command line tool to bulk-generate QR Codes from one or more CSV files.

Allows control over the following QR configuration options:

* QR max and min version
* Error correction level
* Mask

### Latest Release

[Latest Release Page][latest-release]

#### Downloads

* [x86_64-apple-darwin.zip][mac-release]
* [x86_64-pc-windows-msvc.zip][windows-release]
* [x86_64-unknown-linux-musl.zip][linux-release]

## Description

Given one or more input files, each line will be read and turned into a QR code.  The file must have the following CSV
format:

```CSV
file name,content
site_url,"https://ihamlin.co.uk"
productid,9998819191919191
```

The headers are optional, but the relevant flag will need to be passed to the tool in order to ensure correct processing.

## Usage

```console
USAGE:
    qrgen [FLAGS] [OPTIONS] <infile>...

FLAGS:
    -s, --skip       A flag indicating if the first line of the CSV is a header and should be skipped, defaults to false
                     if not specified.
    -h, --help       Prints help information
    -l, --log        A flag indicating if output will be logged, defaults to false if not specified.
    -V, --version    Prints version information
    -v, --verbose    Verbose logging mode (-v, -vv, -vvv)

OPTIONS:
    -x, --max <QR version max>              The maximum version number supported in the QR Code Model 2 standard, or 40
                                            if not specified. [default: 40]
    -m, --min <QR version min>              The minimum version number supported in the QR Code Model 2 standard, or 1
                                            if not specified. [default: 1]
    -b, --border <border>                   The size of the border on the generated QR Code, defaults to 4 if not
                                            specified. [default: 4]
    -c, --chunk <chunk size>                The number of lines to try and process in parallel, if not specified
                                            defaults to 1 and file is processed line by line. [default: 1]
    -e, --error <error correction level>    The error correction level used in this QR Code, or High if not specified.
                                            "Low" The QR Code can tolerate about  7% erroneous codewords. "Medium" The
                                            QR Code can tolerate about 15% erroneous codewords. "Quartile" The QR Code
                                            can tolerate about 25% erroneous codewords. "High" The QR Code can tolerate
                                            about 30% erroneous codewords. [default: High]
    -k, --mask <mask>                       The mask value to apply to the QR Code, between 0 and 7 (inclusive).
    -f, --format <output format type>       The target output format.  Defaults to SVG if not specified. [default: SVG]
    -o, --output <output path>              Output path, or current working directory if not specified or - provided.
                                            [default: -]
    -a, --scale <scale>                     The side length (measured in pixels, must be positive) of each module,
                                            defaults to 8. This value only applies when using the PNG format. Must be
                                            between 1 and 255 (inclusive) [default: 8]

ARGS:
    <infile>...    Input file, must be specified.
```

## Examples

### Basic

The most basic usage is to pass a single CSV file.  This would generate QR Codes using the defaults and saving the
output to the current working directory.

```console
$ # macOS
$ ./qrgen wiktionary.csv
$ ./qrgen wiktionary_small.csv -s // This file has headers so the first line will now be skipped.
```

### Logging

Logging can be turned on with the --log/-l flag combined with zero or more -v options.

```console
$ # macOS
$ ./qrgen wiktionary.csv -l // Warn level
$ ./qrgen wiktionary.csv -l -v // Info level
$ ./qrgen wiktionary.csv -l -vv // Debug level
$ ./qrgen wiktionary.csv -l -vvv // Trace level
```

### Parallelism

For larger CSV files you can try changing the chunk size.  The tool will then try to process N rows in parallel, 
this can lead to speed improvements.

```console
$ # macOS
$  time ./qrgen wiktionary.csv -c 1000
real    0m4.192s
user    0m24.714s
sys     0m2.047s

$ time ./qrgen wiktionary.csv -c 1000 -f png //PNG output is somewhat slower.
real     0m25.891s
user     2m44.867s
sys     0m7.801s

$ time ./qrgen wiktionary.csv
real    0m18.590s
user    0m16.928s
sys     0m1.539s
```

### Roadmap

- Add support to zip the output.

[azure-badge]: https://dev.azure.com/morpork73/qrgen/_apis/build/status/ian-hamlin.qrgen?branchName=master
[azure-url]: https://dev.azure.com/morpork73/qrgen/_build/latest?definitionId=1&branchName=master
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: LICENSE
[dependency-badge]: https://deps.rs/repo/github/ian-hamlin/qrgen/status.svg
[dependency-url]: https://deps.rs/repo/github/ian-hamlin/qrgen
[latest-release]: https://github.com/ian-hamlin/qrgen/releases/latest
[mac-release]: https://github.com/ian-hamlin/qrgen/releases/latest/download/x86_64-apple-darwin.zip
[windows-release]: https://github.com/ian-hamlin/qrgen/releases/latest/download/x86_64-pc-windows-msvc.zip
[linux-release]: https://github.com/ian-hamlin/qrgen/releases/latest/download/x86_64-unknown-linux-musl.zip