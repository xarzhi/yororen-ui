use criterion::{Criterion, criterion_group, criterion_main};
use proc_macro2::Span;

fn bench_codegen(c: &mut Criterion) {
    let inputs = [
        ("simple_label", r#"<Label id="l" text="Hello" />"#),
        (
            "button_with_event",
            r#"<Button id="b" caption="Save" on_click={controller.save} />"#,
        ),
        (
            "nested_containers",
            r#"
<Column flex col gap="3" p="4">
    <Row flex row gap="2">
        <Label id="l1" text="One" />
        <Label id="l2" text="Two" />
        <Label id="l3" text="Three" />
    </Row>
    <Button id="b" caption="OK" variant="primary" />
</Column>
"#,
        ),
        (
            "for_loop",
            r#"
<Column flex col gap="2">
    <For each={items.clone()} let:item let:index={i} key={i}>
        <Label id={format!("l-{}", i)} text={item.clone()} />
    </For>
</Column>
"#,
        ),
        (
            "conditional",
            r#"
<Column flex col gap="2">
    <If condition={show_title}>
        <Heading id="h" level="H2" text="Title" />
    </If>
    <Label id="l" text="Body" />
</Column>
"#,
        ),
    ];

    for (name, xml) in inputs {
        c.bench_function(&format!("codegen/{name}"), |b| {
            b.iter(|| {
                yororen_ui_xml::codegen::codegen(xml, Span::call_site(), None, None, &[])
                    .expect("codegen succeeds")
            })
        });
    }

    // A benchmark that parses only, to separate parser cost
    // from codegen cost.
    let parse_input = r#"
<Column flex col gap="3" p="4">
    <Row flex row gap="2">
        <Label id="l1" text="One" />
        <Label id="l2" text="Two" />
    </Row>
    <Button id="b" caption="OK" />
</Column>
"#;
    c.bench_function("parser/nested", |b| {
        b.iter(|| {
            let line_starts = yororen_ui_xml::parser::line_starts(parse_input);
            let location = yororen_ui_xml::parser::LocationTracker {
                line_starts: &line_starts,
                xml: parse_input,
                outer_span: Span::call_site(),
            };
            yororen_ui_xml::parser::parse(parse_input, Span::call_site(), &location)
                .expect("parse succeeds")
        })
    });
}

criterion_group!(benches, bench_codegen);
criterion_main!(benches);
