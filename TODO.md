## MultipartParser

- [X] Implement `parse_options_header` function in to get the header value, and the options.
- [X] Send `File` and `Field` as `next_part()` in `MultipartParser`.
- [X] Make sure `Content-Disposition` is present in each the part, or raise an error.
- [ ] Limit the size of the part, and the size of the file.
- [ ] Implement `_charset_`.
- [X] Error on multiple `Content-Disposition` headers.
- [ ] Add `tracing` to the parser.

## MultipartBuilder

I want to implement a `MultipartBuilder` to build the `multipart/form-data` request.
