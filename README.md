[![Build Status](https://dev.azure.com/morpork73/qrgen/_apis/build/status/ian-hamlin.qrgen?branchName=master)](https://dev.azure.com/morpork73/qrgen/_build/latest?definitionId=1&branchName=master)

# About

A simple command line tool to bulk-generate QR Codes from one or more CSV files.

Allows control over the following QR configuration options:

* QR max and min version
* Error correction level
* Mask

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
    -x, --max <QR version max>        The maximum version number supported in the QR Code Model 2 standard, or 40 if not
                                      specified. [default: 40]
    -m, --min <QR version min>        The minimum version number supported in the QR Code Model 2 standard, or 1 if not
                                      specified. [default: 1]
    -b, --border <border>             The size of the border on the generated QR Code, defaults to 4 if not specified.
                                      [default: 4]
    -c, --chunk <chunk size>          The number of lines to try and process in parallel, if not specified defaults to 1
                                      and file is processed line by line. [default: 1]
    -e, --error <error correction>    The error correction level used in this QR Code, or High if not specified. "Low"
                                      The QR Code can tolerate about  7% erroneous codewords. "Medium" The QR Code can
                                      tolerate about 15% erroneous codewords. "Quartile" The QR Code can tolerate about
                                      25% erroneous codewords. "High" The QR Code can tolerate about 30% erroneous
                                      codewords. [default: High]
    -k, --mask <mask>                 The mask value to apply to the QR Code, between 0 and 7 (inclusive).
    -o, --output <output>             Output path, or current working directory if not specified or - provided.
                                      [default: -]

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
$ ./qrgen wiktionary_small.csv -s // This file has headers to the first line will now be skipped.
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
real	0m4.192s
user	0m24.714s
sys	0m2.047s

$ time ./qrgen wiktionary.csv
real	0m18.590s
user	0m16.928s
sys	0m1.539s

```