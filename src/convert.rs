use crate::content::Content;
use crate::error::EyreToAnyhow;
use anyhow::{Context, Result};
use epub_builder::{EpubBuilder, ZipCommand};
use lazy_regex::Regex;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

type Epub = EpubBuilder<ZipCommand>;
type RegexRef = Cow<'static, Regex>;

pub struct ContentConverter {
    txt_file: BufReader<File>,
    epub_builder: EpubBuilder<ZipCommand>,
    vol_regex: Cow<'static, Regex>,
    chap_regex: Cow<'static, Regex>,
    content_buffer: Option<Content>,
    epub_path: PathBuf,
}

impl ContentConverter {
    pub fn new(txt_path: &PathBuf, builder: Epub, vol: RegexRef, chap: RegexRef) -> Result<Self> {
        Ok(Self {
            txt_file: BufReader::new(File::open(txt_path)?),
            epub_builder: builder,
            vol_regex: vol,
            chap_regex: chap,
            content_buffer: None,
            epub_path: txt_path.with_extension("epub"),
        })
    }

    fn read_line(&mut self) -> Result<Option<String>> {
        let mut buffer = String::new();
        match self.txt_file.read_line(&mut buffer)? {
            0 => Ok(None),
            _ => Ok(Some(buffer.trim().to_string())),
        }
    }

    fn flush_last_content_buffer(&mut self) -> Result<()> {
        if let Some(content) = self.content_buffer.take() {
            self.epub_builder
                .add_content(content.into_epub_content()?)
                .into_anyhow()?;
        }
        Ok(())
    }

    fn create_content_buffer(&mut self, title: String, level: i32) -> Result<()> {
        let is_first = self.content_buffer.is_none();
        self.flush_last_content_buffer()?;

        let mut content = Content::new(title, level).context("create epub content fail.")?;
        if is_first {
            content.set_beginning_of_real_content();
        }
        self.content_buffer = Some(content);
        Ok(())
    }

    pub fn covert_content_into_epub(mut self) -> Result<()> {
        while let Some(line) = self.read_line()? {
            // 卷标题行
            if self.vol_regex.is_match(&line) {
                self.create_content_buffer(line, 0)?;
                continue;
            }

            // 章标题行
            if self.chap_regex.is_match(&line) {
                self.create_content_buffer(line, 1)?;
                continue;
            }

            // 正文前没有卷标题或章标题使用「默认章节」作为标题
            if self.content_buffer.is_none() {
                self.create_content_buffer("默认章节".into(), 1)?;
            }

            // 写入正文
            self.content_buffer
                .as_mut()
                .unwrap()
                .write_line(line)
                .context("write epub content fail.")?;
        }

        // 将缓冲区清空
        self.flush_last_content_buffer()?;

        let epub_file = File::create_new(self.epub_path).context("create epub file fail.")?;
        self.epub_builder
            .generate(epub_file)
            .into_anyhow()
            .context("write epub file fail.")
    }
}
