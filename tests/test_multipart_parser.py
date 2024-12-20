import logging

import pytest

from parser import Field, File, MultipartParser, MultipartState, MultipartPart

logging.getLogger().setLevel(logging.DEBUG)


def test_parser_size_boundary():
    MultipartParser(b"x" * 70)
    with pytest.raises(ValueError, match="Boundary length must be between 1 and 70 characters."):
        MultipartParser(b"x" * 71)


@pytest.fixture(scope="function")
def parser() -> MultipartParser:
    return MultipartParser(b"boundary")


def test_parser_preamble(parser: MultipartParser):
    parser.parse(b"\r\n--boundary")
    assert parser.state == MultipartState.PREAMBLE, "We should be at the 'Preamble' state, and be waiting for CRLF."


def test_parser_preamble_crlf(parser: MultipartParser):
    parser.parse(b"\r\n--boundary\r\n")
    assert parser.state == MultipartState.HEADER, "We should be at the 'Header' state, and be waiting for a header."


# NOTE: This errors on `python-multipart`, but on the RFC 2046, it says it shouldn't be an error.
# Ref.: https://www.rfc-editor.org/rfc/rfc2046.html#section-5.1.1
def test_parser_preamble_expected_boundary_character(parser: MultipartParser):
    parser.parse(b"--Boundary\r\n")
    assert parser.state == MultipartState.PREAMBLE, "We should be at the 'PREAMBLE' state."

    parser.parse(b"--boundary\r\n")
    assert parser.state == MultipartState.HEADER, "We should be at the 'HEADER' state."


def test_parser_preamble_cr_after_delimiter(parser: MultipartParser):
    parser.parse(b"--boundary\r")
    assert parser.state == MultipartState.PREAMBLE, "We should be at the 'PREAMBLE' state."

    parser.parse(b"--boundary\r\n")
    assert parser.state == MultipartState.HEADER, "We should be at the 'HEADER' state."


def test_parser_preamble_lf_after_delimiter(parser: MultipartParser):
    parser.parse(b"--boundary\n")
    assert parser.state == MultipartState.PREAMBLE, "We should be at the 'PREAMBLE' state."


def test_parser_preamble_random_characters_after_delimiter(parser: MultipartParser):
    parser.parse(b"--boundaryfoobar")
    assert parser.state == MultipartState.PREAMBLE, "We should be at the 'PREAMBLE' state."

    parser.parse(b"--boundary\r\n")
    assert parser.state == MultipartState.HEADER, "We should be at the 'HEADER' state."


def test_parser_preamble_end(parser: MultipartParser):
    parser.parse(b"\r\n--boundary--")
    assert parser.state == MultipartState.END, "We should be at the 'End' state, and be done parsing."


def test_parser_header(parser: MultipartParser):
    parser.parse(b'\r\n--boundary\r\nContent-Disposition: form-data; name="field1"\r\n\r\n')
    assert parser.state == MultipartState.BODY, "We should be at the 'Body' state, and be waiting for a body."

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "content-disposition"
    assert event.value == 'form-data; name="field1"'


def test_parser_multiple_headers(parser: MultipartParser):
    parser.parse(b'\r\n--boundary\r\nContent-Disposition: form-data; name="field1"\r\nContent-Type: text/plain\r\n\r\n')
    assert parser.state == MultipartState.BODY, "We should be at the 'Body' state, and be waiting for a body."

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "content-disposition"
    assert event.value == 'form-data; name="field1"'

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "content-type"
    assert event.value == "text/plain"


def test_parser_body(parser: MultipartParser):
    parser.parse(b'\r\n--boundary\r\nContent-Disposition: form-data; name="field1"\r\n\r\nHello World!\r\n--boundary--')
    assert parser.state == MultipartState.END, "We should be at the 'END' state, and be done parsing."

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "content-disposition"
    assert event.value == 'form-data; name="field1"'

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Body)
    assert event.data == b"Hello World!"


def test_parser_first_form_data(parser: MultipartParser):
    parser.parse(
        b"\r\n--boundary\r\n"
        b'content-disposition: form-data; name="field1"\r\n'
        b"content-type: text/plain;charset=UTF-8\r\n"
        b"\r\n"
        b"Joe owes =E2=82=AC100."
        b"\r\n--boundary--"
    )
    assert parser.state == MultipartState.END, "We should be at the 'END' state, and be done parsing."

    part = parser.next_part()
    assert isinstance(part, Field)
    assert part.name == '"field1"'
    assert part.data == b"Joe owes =E2=82=AC100."


def test_parser_multiple_form_data(parser: MultipartParser):
    parser.parse(
        b"\r\n--boundary\r\n"
        b'content-disposition: form-data; name="field1"\r\n'
        b"content-type: text/plain;charset=UTF-8\r\n"
        b"\r\n"
        b"Joe owes =E2=82=AC100.\r\n"
        b"--boundary\r\n"
        b'content-disposition: form-data; name="field2"\r\n'
        b"content-type: text/plain;charset=UTF-8\r\n"
        b"\r\n"
        b"Hello World!"
        b"\r\n--boundary--"
    )
    assert parser.state == MultipartState.END, "We should be at the 'END' state, and be done parsing."

    part = parser.next_part()
    assert isinstance(part, Field)
    assert part.name == '"field1"'
    assert part.data == b"Joe owes =E2=82=AC100."

    part = parser.next_part()
    assert isinstance(part, Field)
    assert part.name == '"field2"'
    assert part.data == b"Hello World!"


def test_parser_file_data(parser: MultipartParser):
    parser.parse(
        b"\r\n--boundary\r\n"
        b'content-disposition: form-data; name="file"; filename="example.txt"\r\n'
        b"content-type: text/plain;charset=UTF-8\r\n"
        b"\r\n"
        b"Hello World!"
        b"\r\n--boundary--"
    )
    assert parser.state == MultipartState.END, "We should be at the 'END' state, and be done parsing."

    part = parser.next_part()
    assert isinstance(part, File)
    assert part.name == '"file"'
    assert part.filename == '"example.txt"'
    assert part.data == b"Hello World!"


def test_parser_missing_content_disposition(parser: MultipartParser):
    with pytest.raises(ValueError, match="Missing content-disposition header"):
        parser.parse(
            b"\r\n--boundary"
            b"\r\n"
            b"content-type: text/plain;charset=UTF-8\r\n"
            b"\r\n"
            b"Big Hello World Message!"
            b"\r\n--boundary--"
        )


def test_parser_multiple_content_disposition(parser: MultipartParser):
    parser.parse(
        b"\r\n--boundary"
        b"\r\n"
        b'content-disposition: form-data; name="field1"\r\n'
        b'content-disposition: form-data; name="field2"\r\n'
        b"\r\n"
        b"Big Hello World Message!"
        b"\r\n--boundary--"
    )
    assert parser.state == MultipartState.END, "We should be at the 'END' state, and be done parsing."

    part = parser.next_part()
    assert isinstance(part, Field)
    assert part.name == '"field2"'
    assert part.data == b"Big Hello World Message!"
