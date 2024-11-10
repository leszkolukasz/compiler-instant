use std::cmp::max;
use crate::common::LineCol;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

pub struct Error {
    pub msg: String,
    pub pos: LineCol,
}

pub fn report_error(filename: String, src: &str, error: Error) {
    let source = Source::from(src);
    let line = source.line(max(0, error.pos.line as usize - 1));
    let span = line.map_or(0..1, |l| l.span());

    Report::build(ReportKind::Error, filename.clone(), span.start)
        .with_message(format!("[{},{}]: {}", error.pos.line, error.pos.col, error.msg))
        .with_label(
            Label::new((filename.clone(), span))
                .with_message(format!("{}", "Somewhere here".fg(Color::Yellow)))
                .with_color(Color::Red)
        )
        .finish().print((filename, source)).unwrap();
}