# Rust Parser for `multipart/form-data`

> [!WARNING]
> Disclaimer: This is a work in progress and not ready for production use.

This package provides a Sans-IO parser for [RFC 7578](https://datatracker.ietf.org/doc/html/rfc7578) `multipart/form-data`.

> [!NOTE]
> This package is heavily inspired by [defnull/multipart](https://github.com/defnull/multipart).

## Installation

```bash
pip install multipart-parser
```

## Usage

```py
from multipart_parser import MultipartParser, MultipartPart, Field

parser = MultipartParser(boundary=b"boundary")
parser.parse(b'\r\n--boundary\r\nContent-Disposition: form-data; name="user"\r\n\r\nPotato\r\n--boundary--\r\n')

field = parser.next_part()
assert isinstance(field, Field)
assert field.name == '"user"'
assert field.data == "Potato"
```

## Contribute

I run the project like this:

```bash
uv run maturin develop && pytest -vvvs
```

## License

This project is licensed under the MIT License.
