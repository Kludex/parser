import logging

import pytest

from parser import MultipartParser, MultipartState, MultipartPart

logging.getLogger().setLevel(logging.DEBUG)


@pytest.fixture(scope="function")
def parser() -> MultipartParser:
    return MultipartParser(b"boundary")


def test_parser_preamble(parser: MultipartParser):
    parser.parse(b"\r\n--boundary")
    assert parser.state == MultipartState.PREAMBLE, "We should be at the 'Preamble' state, and be waiting for CRLF."


def test_parser_preamble_crlf(parser: MultipartParser):
    parser.parse(b"\r\n--boundary\r\n")
    assert parser.state == MultipartState.HEADER, "We should be at the 'Header' state, and be waiting for a header."


# I need to read the specification. Should this be an error? This is an error on python-multipart.
@pytest.mark.skip(reason="Unknown expected behavior")
def test_parser_preamble_expected_boundary_character(parser: MultipartParser):
    parser.parse(b"--Boundary\r\n")


def test_parser_preamble_invalid_line_break_after_delimiter(parser: MultipartParser):
    with pytest.raises(ValueError, match="Invalid line break after delimiter"):
        parser.parse(b"--boundary\rfoobar")


# I need to read the specification. Should this be an error, or should we wait for a delimiter that ends with CRLF?
@pytest.mark.skip(reason="Unknown expected behavior")
def test_parser_preamble_random_characters_after_delimiter(parser: MultipartParser):
    parser.parse(b"--boundaryfoobar")


def test_parser_preamble_end(parser: MultipartParser):
    parser.parse(b"\r\n--boundary--")
    assert parser.state == MultipartState.END, "We should be at the 'End' state, and be done parsing."


def test_parser_header(parser: MultipartParser):
    parser.parse(b"\r\n--boundary\r\nContent-Type: text/plain\r\n\r\n")
    assert parser.state == MultipartState.BODY, "We should be at the 'Body' state, and be waiting for a body."

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "Content-Type"
    assert event.value == "text/plain"


def test_parser_multiple_headers(parser: MultipartParser):
    parser.parse(b"\r\n--boundary\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\n")
    assert parser.state == MultipartState.BODY, "We should be at the 'Body' state, and be waiting for a body."

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "Content-Type"
    assert event.value == "text/plain"

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "Content-Length"
    assert event.value == "5"


def test_parser_body(parser: MultipartParser):
    parser.parse(b"\r\n--boundary\r\nContent-Type: text/plain\r\n\r\nHello World!--boundary--")
    assert parser.state == MultipartState.END, "We should be at the 'END' state, and be done parsing."

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Header)
    assert event.name == "Content-Type"
    assert event.value == "text/plain"

    event = parser.next_event()
    assert isinstance(event, MultipartPart.Body)
    assert event.data == b"Hello World!"
