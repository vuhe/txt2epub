use std::io::{Cursor, Seek, SeekFrom};

use anyhow::Result;
use epub_builder::{EpubContent, ReferenceType};
use lazy_regex::regex_is_match;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use uuid::Uuid;

pub struct Content {
    title: String,
    level: i32,
    reference_type: Option<ReferenceType>,
    writer: Writer<Cursor<Vec<u8>>>,
}

impl Content {
    pub fn new(title: String, level: i32) -> Result<Self> {
        let buffer = Cursor::new(Vec::new());
        let mut writer = Writer::new_with_indent(buffer, b' ', 2);

        // header
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
        let doc_type = r#"html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd""#;
        writer.write_event(Event::DocType(BytesText::from_escaped(doc_type)))?;

        // html start
        let html_start =
            BytesStart::new("html").with_attributes([("xmlns", "http://www.w3.org/1999/xhtml")]);
        writer.write_event(Event::Start(html_start))?;

        // html header
        writer.write_event(Event::Start(BytesStart::new("head")))?;
        writer.write_event(Event::Start(BytesStart::new("title")))?;
        writer.write_event(Event::Text(BytesText::new(&title)))?;
        writer.write_event(Event::End(BytesEnd::new("title")))?;
        let css = BytesStart::new("link").with_attributes([
            ("rel", "stylesheet"),
            ("type", "text/css"),
            ("href", "../css/page.css"),
        ]);
        writer.write_event(Event::Empty(css))?;
        writer.write_event(Event::End(BytesEnd::new("head")))?;

        // html body
        writer.write_event(Event::Start(BytesStart::new("body")))?;

        // 章节标题
        writer.write_event(Event::Start(BytesStart::new("h3")))?;
        writer.write_event(Event::Text(BytesText::new(&title)))?;
        writer.write_event(Event::End(BytesEnd::new("h3")))?;

        Ok(Self {
            title,
            level,
            reference_type: None,
            writer,
        })
    }

    pub fn set_beginning_of_real_content(&mut self) {
        self.reference_type = Some(ReferenceType::Text);
    }

    pub fn write_line(&mut self, mut text: String) -> Result<()> {
        text.retain(|c| match c {
            '\t' | '\n' | '\r' => true,
            '\0'..='\x1F' | '\x7F' => false,
            _ => true,
        });
        let text = text.trim();
        if text.is_empty() {
            // skip empty line
        } else if regex_is_match!(r"^(=+|-+|—+|\*+)$", text) {
            self.writer
                .write_event(Event::Empty(BytesStart::new("hr")))?;
        } else {
            self.writer
                .write_event(Event::Start(BytesStart::new("p")))?;
            self.writer.write_event(Event::Text(BytesText::new(text)))?;
            self.writer.write_event(Event::End(BytesEnd::new("p")))?;
        }
        Ok(())
    }

    pub fn into_epub_content(mut self) -> Result<EpubContent<Cursor<Vec<u8>>>> {
        self.writer.write_event(Event::End(BytesEnd::new("body")))?;
        self.writer.write_event(Event::End(BytesEnd::new("html")))?;
        let mut bytes = self.writer.into_inner();
        bytes.seek(SeekFrom::Start(0))?;

        let href = format!("xhtml/page_{}.xhtml", Uuid::new_v4().simple());
        let mut content = EpubContent::new(href, bytes)
            .title(self.title)
            .level(self.level);
        content.reftype = self.reference_type;

        Ok(content)
    }
}
