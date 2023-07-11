use std::io;

use crate::ecma::{writer::EcmaWriter, Program};

pub struct HtmlWriter<W> {
    writer: W,
}

impl<W> HtmlWriter<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_element(&mut self, element: &Element) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.writer.write(&[b'<'])?;
        bytes_written += self.writer.write(element.name.as_bytes())?;
        bytes_written += self.writer.write(&[b'>'])?;

        bytes_written += element
            .children
            .iter()
            .map(|child| self.write_child(child))
            .sum::<io::Result<usize>>()?;

        bytes_written += self.writer.write(&[b'<', b'/'])?;
        bytes_written += self.writer.write(element.name.as_bytes())?;
        bytes_written += self.writer.write(&[b'>'])?;

        Ok(bytes_written)
    }

    fn write_child(&mut self, child: &Child) -> io::Result<usize> {
        match child {
            Child::Element(element) => self.write_element(element),
            Child::Text(text) => self.writer.write(text.as_bytes()),
            Child::Script(program) => {
                let mut bytes_written = 0;

                bytes_written += self.writer.write(b"<script>")?;

                let mut ecma_writer = EcmaWriter::new(&mut self.writer);
                bytes_written += ecma_writer.write_program(program)?;

                bytes_written += self.writer.write(b"</script>")?;

                Ok(bytes_written)
            }
        }
    }
}

pub struct Element {
    pub name: String,
    pub children: Vec<Child>,
}

pub enum Child {
    Element(Element),
    Text(String),
    Script(Program),
}
