use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet},
};

fn main() {
    let snippet = Snippet {
        title: Some(Annotation {
            label: Some("mismatched types".to_string()),
            id: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices: vec![
            Slice {
                source: "Foo".to_string(),
                line_start: 51,
                origin: Some("src/format.rs".to_string()),
                fold: false,
                annotations: vec![],
            },
            Slice {
                source: "Faa".to_string(),
                line_start: 129,
                origin: Some("src/display.rs".to_string()),
                fold: false,
                annotations: vec![],
            },
        ],
        opt: FormatOptions {
            color: true,
            ..Default::default()
        },
    };

    let dl = DisplayList::from(snippet);
    println!("{}", dl);
}
