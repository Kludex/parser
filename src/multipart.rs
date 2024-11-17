//! Sans-IO parser for [RFC 7578](https://datatracker.ietf.org/doc/html/rfc7578) `multipart/form-data`.
//!
//! This parser works based on the following states:
//!
//! ```not-rust
//! State 1: Preamble
//!     State 2: Header
//!     State 4: End (final)
//! State 2: Header
//!     State 3: Body
//!     State 4: End (final)
//! State 3: Body
//!     State 2: Header
//!     State 4: End (final)
//! ```

use core::fmt;
use std::{collections::HashMap, str};

use log::debug;
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PyBytes,
};

use crate::form_data::FormData;

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const CRLF: [u8; 2] = [CR, LF];

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq, Debug)]
pub enum MultipartState {
    #[pyo3(name = "PREAMBLE")]
    Preamble,
    #[pyo3(name = "HEADER")]
    Header,
    #[pyo3(name = "BODY")]
    Body,
    #[pyo3(name = "END")]
    End,
}

#[derive(Debug, Clone, FromPyObject)]
pub struct BytesWrapper(Vec<u8>);

impl IntoPy<PyObject> for BytesWrapper {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyBytes::new_bound(py, &self.0).into_any().unbind()
    }
}

impl fmt::Display for BytesWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum MultipartPart {
    Header { name: String, value: String },
    Body { data: BytesWrapper, complete: bool },
}

impl MultipartPart {
    fn build_header(data: &[u8]) -> PyResult<Self> {
        let parts = match data.iter().position(|&c| c == b':') {
            Some(index) => index,
            None => return Err(PyValueError::new_err("Malformed header")),
        };

        let key = &data[..parts];
        let value = &data[parts + 1..];

        // TODO: The encoding should be determined by the HTTP Content-Type header.
        let key = str::from_utf8(key).map_err(|_| PyValueError::new_err("Invalid key"))?.trim();
        let value = str::from_utf8(value).map_err(|_| PyValueError::new_err("Invalid value"))?.trim();

        Ok(MultipartPart::Header {
            name: key.to_lowercase(),
            value: value.to_string(),
        })
    }
}

#[pymethods]
impl MultipartPart {
    fn __repr__(&self) -> String {
        match self {
            Self::Header { name, value } => format!("Header(name=\"{name}\", value=\"{value}\")"),
            Self::Body { data, complete } => format!("Body(data=\"{data}\", complete={complete})"),
        }
    }
}

#[pyclass]
pub struct MultipartParser {
    _boundary: Vec<u8>,
    max_size: Option<usize>,

    // TODO: How can I use `str` instead of `String` here?
    /// The charset to use when decoding headers.
    _header_charset: String,

    _state: MultipartState,
    _buffer: Vec<u8>,

    /// The boundary with a leading `--`.
    _dash_boundary: Vec<u8>,

    /// The combination of CRLF + `--` + boundary.
    _delimiter: Vec<u8>,

    _offset: usize,
    _events: Vec<MultipartPart>,
    _need_data: bool,

    /// The headers of the current part.
    _current_headers: HashMap<String, String>,

    /// The current part being parsed.
    _current_part: Option<FormData>,

    /// The parsed parts.
    _parts: Vec<FormData>,
}

#[pymethods]
impl MultipartParser {
    // TODO: Can `header_charset` be only `&str`?
    #[new]
    #[pyo3(signature = (boundary, max_size = None, header_charset = "utf8"))]
    fn new(boundary: Vec<u8>, max_size: Option<usize>, header_charset: Option<&str>) -> PyResult<Self> {
        // According to https://www.rfc-editor.org/rfc/rfc2046.html#section-5.1.1, the boundary
        // should be between 1 and 70 bytes.
        if boundary.len() < 1 || boundary.len() > 70 {
            return Err(PyValueError::new_err("Boundary length must be between 1 and 70 characters."));
        }

        // TODO: Implement more header charset support.
        if header_charset != Some("utf8") {
            return Err(PyRuntimeError::new_err("The only supported charset is 'utf8'."));
        }

        let _dash_boundary = [b"--".as_slice(), &boundary].concat();
        let _delimiter = [b"\r\n".as_slice(), &_dash_boundary].concat();

        Ok(MultipartParser {
            _boundary: boundary,
            max_size: max_size,
            _header_charset: header_charset.unwrap_or("utf8").to_string(),
            _state: MultipartState::Preamble,
            _buffer: Vec::new(),
            _dash_boundary,
            _delimiter,
            _offset: 0,
            _events: Vec::new(),
            _need_data: false,

            _current_headers: HashMap::new(),
            _current_part: None,
            _parts: Vec::new(),
        })
    }

    #[getter]
    fn state(&self) -> PyResult<MultipartState> {
        Ok(self._state.clone())
    }

    fn parse(&mut self, data: Vec<u8>) -> PyResult<()> {
        if self._state == MultipartState::End {
            return Err(PyRuntimeError::new_err("Parser is in the end state."));
        }

        if self.max_size.is_some() && self._buffer.len() + data.len() > self.max_size.unwrap() {
            return Err(PyRuntimeError::new_err("Data exceeds maximum size."));
        }

        self._buffer.extend(data);
        self._need_data = false;

        loop {
            self._state = match self._state {
                MultipartState::Preamble => self.handle_preamble(),
                MultipartState::Header => self.handle_header(),
                MultipartState::Body => self.handle_body(),
                MultipartState::End => break,
            }?;

            // TODO: Do we need create this `_need_data` flag?
            if self._need_data {
                break;
            }
        }

        Ok(())
    }

    fn next_part(&mut self) -> PyResult<Option<FormData>> {
        match self._parts.is_empty() {
            true => Ok(None),
            false => Ok(Some(self._parts.remove(0))),
        }
    }

    fn next_event(&mut self) -> PyResult<Option<MultipartPart>> {
        match self._events.is_empty() {
            true => Ok(None),
            false => Ok(Some(self._events.remove(0))),
        }
    }
}

impl MultipartParser {
    fn handle_preamble(&mut self) -> PyResult<MultipartState> {
        let delimiter = self._dash_boundary.clone();
        let delimiter_len = delimiter.len();
        let buffer = self._buffer[self._offset..].to_vec();

        if let Some(index) = buffer.windows(delimiter_len).position(|window| window == delimiter) {
            if let Some(after_delimiter) = buffer.get(index + delimiter_len..) {
                let tail = after_delimiter.get(..2).unwrap_or_default();

                // First delimiter found -> End of preamble
                if tail == CRLF {
                    self._offset += index + delimiter_len + 2;
                    return Ok(MultipartState::Header);
                }

                // First delimiter is terminator -> Empty multipart stream
                if tail == b"--" {
                    return Ok(MultipartState::End);
                }

                // Bad newline after valid delimiter -> Broken client
                if tail == b"\n" {
                    return Err(PyValueError::new_err("Invalid line break after delimiter"));
                }

                // CR found after delimiter, but next byte is not LF -> Move offset
                if tail.len() > 1 && tail[0] == CR {
                    self._offset += index + delimiter_len + 1;
                    return Ok(MultipartState::Preamble);
                }
            }
        }

        // Delimiter not found -> Skip data
        self._offset = self._offset.max(self._buffer.len().saturating_sub(delimiter_len + 4));
        self._need_data = true;
        Ok(MultipartState::Preamble)
    }

    fn handle_header(&mut self) -> PyResult<MultipartState> {
        let buffer = self._buffer[self._offset..].to_vec();

        debug!("Buffer: {:?}", bytes_to_str(buffer.clone()));

        // We are looking for a CRLF sequence to separate headers from body.
        match buffer.windows(2).position(|window| window == CRLF) {
            Some(index) => {
                debug!("{:?}: header found at index: {}.", self._state, index);
                // Empty line found, move to body
                if index == 0 {
                    self._offset = self._offset + 2;

                    self._current_part = match FormData::try_from(self._current_headers.clone()) {
                        Ok(part) => Some(part),
                        Err(e) => return Err(e),
                    };
                    return Ok(MultipartState::Body);
                } else {
                    self._offset = self._offset + index + 2;
                    match MultipartPart::build_header(&buffer[..index]) {
                        Ok(MultipartPart::Header { name, value }) => {
                            self._events.push(MultipartPart::Header {
                                name: name.clone(),
                                value: value.clone(),
                            });
                            self._current_headers.insert(name.clone(), value.clone());
                            (name, value)
                        }
                        Err(e) => return Err(e),
                        _ => return Err(PyValueError::new_err("Invalid header")),
                    };

                    return Ok(MultipartState::Header);
                }
            }
            None => match buffer.windows(1).position(|window| window == &[LF]) {
                Some(_) => {
                    return Err(PyValueError::new_err("Invalid line break in header"));
                }
                // Wait for more data.
                None => {
                    self._need_data = true;
                    Ok(MultipartState::Header)
                }
            },
        }
    }

    fn handle_body(&mut self) -> PyResult<MultipartState> {
        let buffer = self._buffer[self._offset..].to_vec();
        let delimiter = self._delimiter.clone();
        let delimiter_len = delimiter.len();

        debug!("Buffer: {:?}", bytes_to_str(buffer.clone()));

        match buffer.windows(delimiter.len()).position(|window| window == delimiter) {
            Some(index) => {
                debug!("{:?}: delimiter found at index: {}.", self._state, index);
                match buffer.get(index + delimiter_len..index + delimiter_len + 2) {
                    Some(tail) => match tail {
                        [CR, LF] => {
                            debug!("{:?}: delimiter is CRLF.", self._state);
                            self._events.push(MultipartPart::Body {
                                data: BytesWrapper(buffer[..index].to_vec()),
                                complete: true,
                            });
                            self.insert_data(buffer[..index].to_vec(), true)?;
                            self._offset += index + delimiter_len + 2;
                            return Ok(MultipartState::Header);
                        }
                        // Delimiter was terminator, end of multipart stream.
                        [b'-', b'-'] => {
                            self.insert_data(buffer[..index].to_vec(), true)?;
                            self._events.push(MultipartPart::Body {
                                data: BytesWrapper(buffer[..index].to_vec()),
                                complete: true,
                            });
                            self._offset += index + delimiter_len + 2;
                            return Ok(MultipartState::End);
                        }
                        _ => {
                            self._need_data = true;
                            return Ok(MultipartState::Body);
                        }
                    },
                    None => {
                        self._need_data = true;
                        return Ok(MultipartState::Body);
                    }
                };
            }
            None => {
                // Delimiter not found, wait for more data.
                debug!("{:?}: delimiter not found.", self._state);
                if buffer.len() > delimiter_len + 3 {
                    self.insert_data(buffer[..buffer.len() - (delimiter_len + 3)].to_vec(), false)?;
                    self._events.push(MultipartPart::Body {
                        data: BytesWrapper(buffer[..buffer.len() - (delimiter_len + 3)].to_vec()),
                        complete: false,
                    });
                    self._offset = self._buffer.len() - (delimiter_len + 3);
                }
                self._need_data = true;
                Ok(MultipartState::Body)
            }
        }
    }

    fn insert_data(&mut self, data: Vec<u8>, complete: bool) -> PyResult<()> {
        match self._current_part.take() {
            Some(mut part) => {
                part.append_data(data);

                if complete {
                    self._parts.push(part);
                    self._current_part = None;
                }
            }
            None => return Err(PyValueError::new_err("Missing current part")),
        }
        Ok(())
    }
}

fn bytes_to_str(data: Vec<u8>) -> String {
    String::from_utf8(data).unwrap()
}
