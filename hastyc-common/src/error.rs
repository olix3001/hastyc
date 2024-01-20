use colored::*;

use crate::{source::SourceFile, span::Span};

/// Error formatter for hasty errors.
pub struct ErrorFmt<'a> {
    segments: Vec<Box<dyn ErrorFmtSegment + 'a>>
}

impl<'a> ErrorFmt<'a> {
    pub fn new() -> Self {
        Self {
            segments: Vec::new()
        }
    }

    pub fn seg(&mut self, seg: impl ErrorFmtSegment + 'a) -> &mut Self {
        self.segments.push(Box::new(seg));
        self
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.seg(ErrorTitleSegment {
            text: title.to_string()
        });
        self
    }

    pub fn source(&mut self, source: &'a SourceFile, span: Span) -> &mut Self {
        self.seg(ErrorSourceSegment {
            source,
            span
        });
        self
    }

    pub fn cause(&mut self, message: &'a str) -> &mut Self {
        self.seg(ErrorCauseSegment {
            message
        });
        self
    }

    pub fn help(&mut self, message: &'a str) -> &mut Self {
        self.seg(HelpMessageSegment {
            message
        });
        self
    }

    pub fn build(&mut self) -> String {
        let mut result = String::new();
        for seg in self.segments.iter() {
            result.push_str(&seg.stringify());
            result.push('\n');
        }
        result
    }
}

pub trait ErrorDisplay<'ctx, Context> {
    fn fmt(&self, fmt: &mut ErrorFmt<'ctx>, ctx: &'ctx Context);
    fn fmt_error(&self, ctx: &'ctx Context) -> String {
        let mut fmt = ErrorFmt::new();
        self.fmt(&mut fmt, ctx);
        fmt.build()
    }
}

pub struct CommonErrorContext<'a> {
    pub source: &'a SourceFile
}

pub trait ErrorFmtSegment {
    fn stringify(&self) -> String;
}

pub struct ErrorTitleSegment {
    text: String
}

impl ErrorFmtSegment for ErrorTitleSegment {
    fn stringify(&self) -> String {
        format!(
            "{}{} {}",
            "error".red().bold(),
            ":".bold(),
            self.text.bold()
        )
    }
}


pub struct ErrorSourceSegment<'a> {
    source: &'a SourceFile,
    span: Span
}

impl<'a> ErrorFmtSegment for ErrorSourceSegment<'a> {
    fn stringify(&self) -> String {
        let src_relative_span = self.span.to_relative(&self.source);
        let source = format!(
            "{} {}:{}.{}",
            "-->".blue(),
            self.source.name,
            src_relative_span.0,
            src_relative_span.1
        );

        let src_line = self.span.get_line(&self.source);
        let num_width = src_relative_span.0.to_string().len();
        let line = format!(
            "{} {} {}",
            src_relative_span.0.to_string().blue(),
            "|".blue(),
            src_line.0
        );

        let highlight_underline = format!(
            "{}{}",
            " ".repeat(src_line.1 as usize),
            "^".repeat(self.span.len() as usize).red()
        );
        let highlight = format!(
            "{} {} {}",
            " ".repeat(num_width),
            "|".blue(),
            highlight_underline
        );

        format!(
            "{}\n{}\n{}",
            source,
            line,
            highlight
        )
    }
}

pub struct ErrorCauseSegment<'a> {
    message: &'a str
}

impl<'a> ErrorFmtSegment for ErrorCauseSegment<'a> {
    fn stringify(&self) -> String {
        format!(
            "{} {}",
            "cause:".purple().bold(),
            self.message.red().bold()
        )
    }
}

pub struct HelpMessageSegment<'a> {
    message: &'a str
}

impl<'a> ErrorFmtSegment for HelpMessageSegment<'a> {
    fn stringify(&self) -> String {
        format!(
            "{} {}",
            "help:".yellow().bold(),
            self.message.bold()
        )
    }
}