use super::annotation::Annotation;
use super::line::{
    DisplayContentElement, DisplayLine, DisplayMark, DisplayMarkType, DisplayRawLine,
    DisplaySourceLine,
};
use crate::{InlineAnnotation, Slice, Snippet, SourceAnnotation};

#[derive(Debug, Clone)]
pub struct DisplayList<'d> {
    pub body: Vec<DisplayLine<'d>>,
}

fn get_header_pos(slice: &Slice) -> (Option<usize>, Option<usize>) {
    let line = slice.line_start;
    (line, None)
}

impl<'d> From<&Snippet<'d>> for DisplayList<'d> {
    fn from(snippet: &Snippet<'d>) -> Self {
        let mut body = vec![];

        if let Some(annotation) = &snippet.title {
            let label = annotation.label.unwrap_or_default();
            body.push(DisplayLine::Raw(DisplayRawLine::Annotation {
                annotation: Annotation {
                    annotation_type: annotation.annotation_type.clone(),
                    id: annotation.id,
                    label: &label,
                },
                source_aligned: false,
                continuation: false,
            }));
        }

        for slice in snippet.slices {
            let slice_dl: DisplayList = slice.into();
            body.extend(slice_dl.body);
        }
        DisplayList { body }
    }
}

impl<'d> From<&Slice<'d>> for DisplayList<'d> {
    fn from(slice: &Slice<'d>) -> Self {
        let mut body = vec![];

        if let Some(path) = slice.origin {
            body.push(DisplayLine::Raw(DisplayRawLine::Origin {
                path,
                pos: get_header_pos(slice),
            }));
        }

        body.push(DisplayLine::Source {
            lineno: None,
            inline_marks: vec![],
            line: DisplaySourceLine::Empty,
        });

        let mut annotations: Vec<&SourceAnnotation> = slice.annotations.iter().collect();
        let mut inline_annotations: Vec<&InlineAnnotation> =
            slice.inline_annotations.iter().collect();

        let mut line_start_pos = 0;

        let mut i = slice.line_start.unwrap_or(1);
        for line in slice.source.lines() {
            let line_range = line_start_pos..(line_start_pos + line.chars().count() + 1);

            let mut current_annotations = vec![];
            let mut current_inline_annotations = vec![];
            let mut inline_marks = vec![];

            annotations.retain(|ann| {
                if line_range.contains(&ann.range.start) && line_range.contains(&ann.range.end) {
                    // Annotation in this line
                    current_annotations.push(*ann);
                    false
                } else if line_range.contains(&ann.range.start)
                    && !line_range.contains(&ann.range.end)
                {
                    // Annotation starts in this line
                    inline_marks.push(DisplayMark {
                        mark_type: DisplayMarkType::AnnotationStart,
                        annotation_type: ann.annotation_type.clone(),
                    });
                    true
                } else if ann.range.start < line_range.start && ann.range.end > line_range.end {
                    // Annotation goes through this line
                    inline_marks.push(DisplayMark {
                        mark_type: DisplayMarkType::AnnotationThrough,
                        annotation_type: ann.annotation_type.clone(),
                    });
                    true
                } else if line_range.contains(&ann.range.end) {
                    // Annotation ends on this line
                    inline_marks.push(DisplayMark {
                        mark_type: DisplayMarkType::AnnotationThrough,
                        annotation_type: ann.annotation_type.clone(),
                    });
                    current_annotations.push(*ann);
                    false
                } else {
                    true
                }
            });

            inline_annotations.retain(|ann| {
                if line_range.contains(&ann.range.start) && line_range.contains(&ann.range.end) {
                    // Annotation in this line
                    current_inline_annotations.push(*ann);
                    false
                } else {
                    true
                }
            });

            let mut frag = vec![];

            let mut ptr = 0;
            for ann in current_inline_annotations {
                if ptr < ann.range.start {
                    frag.push(DisplayContentElement::Text(
                        &line[ptr..ann.range.start - line_range.start],
                    ));
                }
                frag.push(DisplayContentElement::AnnotatedText {
                    text: &line
                        [(ann.range.start - line_range.start)..(ann.range.end - line_range.start)],
                    annotation_type: ann.annotation_type.clone(),
                });
                ptr = ann.range.end - line_range.start;
            }

            if ptr < line_range.end {
                frag.push(DisplayContentElement::Text(&line[ptr..]));
            }

            body.push(DisplayLine::Source {
                lineno: Some(i),
                inline_marks,
                line: DisplaySourceLine::Content(frag),
            });
            for ann in current_annotations {
                let start = if ann.range.start >= line_start_pos {
                    ann.range.start - line_start_pos
                } else {
                    0
                };
                let inline_marks = if ann.range.start < line_start_pos {
                    vec![DisplayMark {
                        mark_type: DisplayMarkType::AnnotationEnd,
                        annotation_type: ann.annotation_type.clone(),
                    }]
                } else {
                    vec![]
                };
                body.push(DisplayLine::Source {
                    lineno: None,
                    inline_marks,
                    line: DisplaySourceLine::Annotation {
                        annotation: Annotation {
                            annotation_type: ann.annotation_type.clone(),
                            id: None,
                            label: ann.label,
                        },
                        range: start..(ann.range.end - line_start_pos),
                    },
                });
            }
            line_start_pos += line_range.len();
            i += 1;
        }

        body.push(DisplayLine::Source {
            lineno: None,
            inline_marks: vec![],
            line: DisplaySourceLine::Empty,
        });

        DisplayList { body }
    }
}
