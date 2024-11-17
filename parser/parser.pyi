import enum

class MultipartState(enum.IntEnum):
    PREAMBLE = 0
    HEADER = 1
    BODY = 2
    END = 3

# TODO: Is this the best type hint we can get?
class MultipartPart:
    class Header:
        @property
        def name(self) -> bytes: ...
        @property
        def value(self) -> bytes: ...

    class Body:
        @property
        def data(self) -> bytes: ...
        @property
        def done(self) -> bool: ...

class FormData:
    class File:
        @property
        def name(self) -> str: ...
        @property
        def filename(self) -> str: ...
        @property
        def content_type(self) -> str: ...
        @property
        def charset(self) -> str: ...
        @property
        def data(self) -> bytes: ...

    class Field:
        @property
        def name(self) -> str: ...
        @property
        def content_type(self) -> str: ...
        @property
        def charset(self) -> str: ...
        @property
        def data(self) -> bytes: ...

class MultipartParser:
    """A state-machine Sans-IO multipart parser.

    The parser is designed to be used in a streaming fashion, where data is fed to the parser in chunks
    and the parser emits events as it parses the data.

    The states are as follows:
    - PREAMBLE: The preamble of the multipart message.
        Which can become either `HEADER` (for the first part header), `BODY` (for the first part body), or `END`.
    - HEADER: A part header.
        Which can become either `HEADER` (for the next header), `BODY` (for the body of the part), or `END`.
    - BODY: A part body.
        Which can become either `HEADER` (for the next part), `BODY` (for the next part), or `END`.
    - END: The end of the multipart message.
        Which is the final state.
    """

    state: int

    def __init__(self, boundary: bytes, max_size: int | None = None, header_charset: str = "utf8") -> None: ...
    """Create a new multipart parser instance.

    Args:
        boundary: The boundary to use for parsing.
        max_size: The maximum size of the body data. If None, no limit is enforced.
        header_charset: The charset to use for decoding header values.
    """
    def parse(self, data: bytes) -> None: ...
    def next_part(self) -> FormData: ...
    def next_event(self) -> MultipartPart.Header | MultipartPart.Body | None: ...
