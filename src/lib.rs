use pyo3::prelude::*;

mod form_data;
mod headers;
mod multipart;

#[pymodule]
fn parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<multipart::MultipartParser>()?;
    m.add_class::<multipart::MultipartState>()?;
    m.add_class::<multipart::MultipartPart>()?;

    m.add_class::<form_data::FormData>()?;

    Ok(())
}
