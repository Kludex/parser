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

class MultipartParser:
    """A Sans-IO multipart parser."""

    state: int

    def __init__(self, boundary: bytes, max_size: int | None = None, header_charset: str = "utf8") -> None: ...
    """Create a new multipart parser instance.

    Args:
        boundary: The boundary to use for parsing.
        max_size: The maximum size of the body data. If None, no limit is enforced.
        header_charset: The charset to use for decoding header values.
    """
    def parse(self, data: bytes) -> None: ...
    def next_event(self) -> MultipartPart.Header | MultipartPart.Body | None: ...
