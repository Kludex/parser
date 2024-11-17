use log::debug;
use std::fmt;
use std::{collections::HashMap, convert::TryFrom};

use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};

use crate::headers;

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
pub enum FormData {
    Field {
        /// The name of the form field. This field MUST be present.
        /// [RFC 7578 - Section 4.2](https://datatracker.ietf.org/doc/html/rfc7578#section-4.2)
        name: String,

        /// Each part MAY have a Content-Type header field, which defaults to "text/plain".
        /// [RFC 7578 - Section 4.4](https://datatracker.ietf.org/doc/html/rfc7578#section-4.4)
        content_type: String,

        /// The charset to use when decoding the field part.
        /// [RFC 7578 - Section 4.2](https://datatracker.ietf.org/doc/html/rfc7578#section-4.2)
        charset: String,

        /// The data of the field part.
        /// [RFC 7578 - Section 4.2](https://datatracker.ietf.org/doc/html/rfc7578#section-4.2)
        data: BytesWrapper,
    },
    File {
        /// The name of the form field. This field MUST be present.
        /// [RFC 7578 - Section 4.2](https://datatracker.ietf.org/doc/html/rfc7578#section-4.2)
        name: String,

        /// The filename of the file being uploaded.
        /// [RFC 7578 - Section 4.2](https://datatracker.ietf.org/doc/html/rfc7578#section-4.2)
        filename: String,

        /// Each part MAY have a Content-Type header field, which defaults to "text/plain".
        /// [RFC 7578 - Section 4.4](https://datatracker.ietf.org/doc/html/rfc7578#section-4.4)
        content_type: String,

        /// The charset to use when decoding the file part.
        /// [RFC 7578 - Section 4.2](https://datatracker.ietf.org/doc/html/rfc7578#section-4.2)
        charset: String,

        /// The data of the file part.
        /// [RFC 7578 - Section 4.2](https://datatracker.ietf.org/doc/html/rfc7578#section-4.2)
        data: BytesWrapper,
    },
}

impl FormData {
    pub fn append_data(&mut self, data: Vec<u8>) {
        match self {
            FormData::Field { data: field_data, .. } => field_data.0.extend(data),
            FormData::File { data: file_data, .. } => file_data.0.extend(data),
        }
    }
}

impl TryFrom<HashMap<String, String>> for FormData {
    type Error = PyErr;

    fn try_from(headers: HashMap<String, String>) -> PyResult<Self> {
        let (content_type, params) = match headers.get("content-type") {
            Some(value) => match headers::parse_options_header(value.to_string()) {
                Ok((content_type, params)) => (content_type, params),
                Err(e) => return Err(PyValueError::new_err(e)),
            },
            None => ("text/plain".to_string(), HashMap::new()),
        };

        let charset = params.get("charset").unwrap_or(&"utf-8".to_string()).to_string();

        let (content_disposition, params) = match headers.get("content-disposition") {
            Some(value) => match headers::parse_options_header(value.to_string()) {
                Ok((content_disposition, params)) => (content_disposition, params),
                Err(e) => return Err(PyValueError::new_err(e)),
            },
            None => return Err(PyValueError::new_err("Missing content-disposition header")),
        };

        if content_disposition != "form-data" {
            return Err(PyValueError::new_err("Invalid content-disposition"));
        }

        let name = match params.get("name") {
            Some(name) => name,
            None => return Err(PyValueError::new_err("Parameter 'name' not found in content-disposition.")),
        };

        debug!("Name: {:?}", name);
        debug!("File name: {:?}", params);

        match params.get("filename") {
            Some(filename) => Ok(FormData::File {
                name: name.clone(),
                filename: filename.clone(),
                content_type,
                charset,
                data: BytesWrapper(Vec::new()),
            }),
            None => Ok(FormData::Field {
                name: name.clone(),
                content_type,
                charset,
                data: BytesWrapper(Vec::new()),
            }),
        }
    }
}
