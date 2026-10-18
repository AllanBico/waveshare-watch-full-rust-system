//! Deterministic layout snapshots for watch and desk sizes.

use super::*;

#[test]
fn representative_layout_snapshot_is_deterministic() {
    assert_eq!(
        snapshot(Region::new(0, 0, 410, 502)),
        "0 0 0 410 502 0 0 410 502\n\
         1 4 4 402 32 4 4 402 32\n\
         2 4 4 86 32 4 4 86 32\n\
         3 98 4 308 32 98 4 308 32\n\
         4 4 44 402 46 4 44 402 46\n\
         5 4 44 20 20 4 44 20 20\n\
         6 209 44 20 20 209 44 20 20\n\
         7 4 70 20 20 4 70 20 20\n\
         8 209 70 20 20 209 70 20 20\n\
         9 4 98 402 30 4 98 402 30\n\
         10 4 92 402 12 4 98 402 6\n\
         11 4 104 402 12 4 104 402 12\n\
         12 4 116 402 12 4 116 402 12\n"
    );
    assert_eq!(
        snapshot(Region::new(0, 0, 224, 160)),
        "0 0 0 224 160 0 0 224 160\n\
         1 4 4 216 32 4 4 216 32\n\
         2 4 4 86 32 4 4 86 32\n\
         3 98 4 122 32 98 4 122 32\n\
         4 4 44 216 46 4 44 216 46\n\
         5 4 44 20 20 4 44 20 20\n\
         6 116 44 20 20 116 44 20 20\n\
         7 4 70 20 20 4 70 20 20\n\
         8 116 70 20 20 116 70 20 20\n\
         9 4 98 216 30 4 98 216 30\n\
         10 4 92 216 12 4 98 216 6\n\
         11 4 104 216 12 4 104 216 12\n\
         12 4 116 216 12 4 116 216 12\n"
    );
}

fn snapshot(viewport: Region) -> String {
    let root = representative_root();
    let mut entries = [EMPTY; 32];
    let count = run(root, viewport, Insets::ZERO, &mut entries);
    let mut out = String::new();
    for entry in &entries[..count] {
        out.push_str(&format!(
            "{} {} {} {} {} {} {} {} {}\n",
            entry.id,
            entry.rect.x,
            entry.rect.y,
            entry.rect.width,
            entry.rect.height,
            entry.clip.x,
            entry.clip.y,
            entry.clip.width,
            entry.clip.height,
        ));
    }
    out
}

fn representative_root() -> &'static Node<'static> {
    let children = root_children();
    Box::leak(Box::new(
        Node::container(
            0,
            Length::Fill(1),
            Length::Fill(1),
            Kind::Column {
                spacing: 8,
                cross_align: Align::Start,
            },
            children,
        )
        .with_padding(Insets::all(4)),
    ))
}

fn root_children() -> &'static [Node<'static>] {
    Box::leak(Box::new([
        Node::container(
            1,
            Length::Fill(1),
            Length::Fixed(32),
            Kind::Row {
                spacing: 8,
                cross_align: Align::Center,
            },
            header_nodes(),
        ),
        Node::container(
            4,
            Length::Fill(1),
            Length::Fit,
            Kind::Grid {
                columns: 2,
                column_spacing: 8,
                row_spacing: 6,
            },
            grid_nodes(),
        ),
        Node::container(
            9,
            Length::Fill(1),
            Length::Fixed(30),
            Kind::List {
                row_height: 12,
                spacing: 0,
                offset: 6,
            },
            list_nodes(),
        ),
    ]))
}

fn header_nodes() -> &'static [Node<'static>] {
    let style = Theme::WATCH.type_scale.title;
    Box::leak(Box::new([
        Node::text(2, Length::Fit, Length::Fit, "WV2", style),
        Node::spacer(3, Length::Fill(1), Length::Fill(1)),
    ]))
}

fn grid_nodes() -> &'static [Node<'static>] {
    Box::leak(Box::new([
        Node::spacer(5, Length::Fixed(20), Length::Fixed(20)),
        Node::spacer(6, Length::Fixed(20), Length::Fixed(20)),
        Node::spacer(7, Length::Fixed(20), Length::Fixed(20)),
        Node::spacer(8, Length::Fixed(20), Length::Fixed(20)),
    ]))
}

fn list_nodes() -> &'static [Node<'static>] {
    let nodes: Vec<Node> = (10..15)
        .map(|id| Node::spacer(id, Length::Fill(1), Length::Fill(1)))
        .collect();
    Box::leak(nodes.into_boxed_slice())
}
