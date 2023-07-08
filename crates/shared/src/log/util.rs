use log4rs::encode;
use std::io;
use std::io::Write;

/// Temporary buffer writer.
/// Needed to apply the correct style to the written log.
#[derive(Debug)]
pub(super) struct BufWriter(io::BufWriter<Vec<u8>>);

impl BufWriter {
    pub fn buffer(&self) -> &[u8] {
        self.0.get_ref().as_slice()
    }
}

impl Default for BufWriter {
    fn default() -> Self {
        Self(io::BufWriter::new(vec![]))
    }
}

impl Write for BufWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl encode::Write for BufWriter {}

#[cfg(test)]
mod tests {
    use crate::log::util::BufWriter;
    use anyhow::Result;
    use std::io::Write;

    #[test]
    pub fn buffer_is_empty_on_creation() {
        let writer = BufWriter::default();
        assert!(writer.buffer().is_empty());
    }

    #[test]
    pub fn writes_are_buffered() -> Result<()> {
        let mut writer = BufWriter::default();
        write!(&mut writer, "This needs to be written buffered")?;
        assert!(writer.buffer().is_empty());

        writer.flush()?;
        assert!(!writer.buffer().is_empty());

        Ok(())
    }
}
