use eframe::egui;
use lr0_parser_rs::AstNode;

pub(super) const NODE_R: f32 = 14.0;
pub(super) const H_GAP: f32 = 48.0;
pub(super) const V_GAP: f32 = 52.0;

pub(super) struct LayoutNode {
    pub label: String,
    pub is_terminal: bool,
    pub x: f32,
    pub y: f32,
    pub subtree_width: f32,
    pub children: Vec<LayoutNode>,
}

pub(super) fn layout_ast(node: &AstNode) -> LayoutNode {
    layout_rec(node, 0)
}

fn layout_rec(node: &AstNode, depth: usize) -> LayoutNode {
    let y = depth as f32 * V_GAP;
    match node {
        AstNode::Terminal(c) => LayoutNode {
            label: c.to_string(),
            is_terminal: true,
            x: H_GAP / 2.0,
            y,
            subtree_width: H_GAP,
            children: vec![],
        },
        AstNode::NonTerminal(name, children) => {
            if children.is_empty() {
                return LayoutNode {
                    label: name.to_string(),
                    is_terminal: false,
                    x: H_GAP / 2.0,
                    y,
                    subtree_width: H_GAP,
                    children: vec![],
                };
            }
            let mut child_layouts: Vec<LayoutNode> =
                children.iter().map(|c| layout_rec(c, depth + 1)).collect();
            let mut x_cursor = 0.0;
            for child in &mut child_layouts {
                shift_x(child, x_cursor);
                x_cursor += child.subtree_width;
            }
            let total_width = x_cursor;
            let center_x = total_width / 2.0;
            LayoutNode {
                label: name.to_string(),
                is_terminal: false,
                x: center_x,
                y,
                subtree_width: total_width,
                children: child_layouts,
            }
        }
    }
}

pub(super) fn shift_x(node: &mut LayoutNode, dx: f32) {
    node.x += dx;
    for child in &mut node.children {
        shift_x(child, dx);
    }
}

pub(super) fn tree_pixel_height(node: &LayoutNode) -> f32 {
    if node.children.is_empty() {
        node.y
    } else {
        node.children
            .iter()
            .map(tree_pixel_height)
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

pub(super) fn draw_tree(painter: &egui::Painter, origin: egui::Pos2, node: &LayoutNode) {
    let center = origin + egui::Vec2::new(node.x, node.y);

    for child in &node.children {
        let child_center = origin + egui::Vec2::new(child.x, child.y);
        painter.line_segment(
            [center, child_center],
            egui::Stroke::new(1.5, egui::Color32::from_gray(110)),
        );
        draw_tree(painter, origin, child);
    }

    let (fill, rim) = if node.is_terminal {
        (
            egui::Color32::from_rgb(60, 45, 10),
            egui::Color32::from_rgb(240, 180, 60),
        )
    } else {
        (
            egui::Color32::from_rgb(20, 55, 30),
            egui::Color32::from_rgb(100, 200, 110),
        )
    };
    painter.circle(center, NODE_R, fill, egui::Stroke::new(1.5, rim));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        &node.label,
        egui::FontId::monospace(11.0),
        rim,
    );
}
