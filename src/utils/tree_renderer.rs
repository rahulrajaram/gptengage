//! Tree renderer for idea trees with Unicode box-drawing and color support

use crate::orchestrator::ideation::{IdeaNode, IdeationResult};
use crate::utils::colors::*;
use crossterm::style::Attribute;
use std::io::Write;

/// Default terminal width fallback
const DEFAULT_TERM_WIDTH: u16 = 80;

/// Renders idea trees to a writer with Unicode box-drawing and color
pub struct TreeRenderer {
    palette: IdeaTreePalette,
    color_mode: ColorMode,
    term_width: usize,
}

impl TreeRenderer {
    /// Create a new TreeRenderer for the given color mode
    pub fn new(color_mode: ColorMode) -> Self {
        let term_width = crossterm::terminal::size()
            .map(|(w, _)| w)
            .unwrap_or(DEFAULT_TERM_WIDTH) as usize;

        Self {
            palette: IdeaTreePalette::for_mode(color_mode),
            color_mode,
            term_width,
        }
    }

    /// Create a TreeRenderer with a specific terminal width (for testing)
    #[cfg(test)]
    pub fn with_width(color_mode: ColorMode, width: usize) -> Self {
        Self {
            palette: IdeaTreePalette::for_mode(color_mode),
            color_mode,
            term_width: width,
        }
    }

    /// Render the full idea tree to the writer
    pub fn render(&self, result: &IdeationResult, w: &mut dyn Write) -> std::io::Result<()> {
        self.render_seed_header(result, w)?;
        self.render_connector("│", w)?;
        writeln!(w)?;

        let count = result.ideas.len();
        for (i, idea) in result.ideas.iter().enumerate() {
            let is_last = i == count - 1;
            self.render_l1_node(idea, is_last, w)?;
        }

        // Reset all attributes at the end
        if self.color_mode != ColorMode::None {
            write!(w, "{}", crossterm::style::SetAttribute(Attribute::Reset))?;
        }

        Ok(())
    }

    /// Render the seed header line: `seed: "..." σ=1.0`
    fn render_seed_header(
        &self,
        result: &IdeationResult,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        let sigma_str = format!("  σ={}", result.sigma);

        if self.color_mode == ColorMode::None {
            writeln!(w, "seed: \"{}\"  sigma={}", result.seed, result.sigma)?;
        } else {
            write!(
                w,
                "{}",
                styled_header("seed:", self.palette.root_fg, None, self.palette.bold)
            )?;
            write!(
                w,
                " {}",
                styled_body(&format!("\"{}\"", result.seed), self.palette.body_fg)
            )?;
            writeln!(
                w,
                "{}",
                styled_dim(&sigma_str, self.palette.axis_fg, self.palette.dim)
            )?;
        }

        Ok(())
    }

    /// Render an L1 node with its children
    fn render_l1_node(
        &self,
        node: &IdeaNode,
        is_last: bool,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        let connector = if is_last { "└── " } else { "├── " };
        let continuation = if is_last { "    " } else { "│   " };

        // Header line: ├── A  Title A
        let header_text = format!("{}  {}", node.id, node.title);
        self.render_node_header(connector, &header_text, true, w)?;

        // Axis + description line
        let axis_desc = format!("{} · {}", node.axis, node.description);
        self.render_wrapped_line(continuation, &axis_desc, false, true, w)?;

        // Deviation rationale
        let deviation = format!("↳ {}", node.deviation_rationale);
        self.render_wrapped_line(continuation, &deviation, true, false, w)?;

        // Children (L2 nodes)
        if !node.children.is_empty() {
            writeln!(w)?;
            let child_count = node.children.len();
            for (j, child) in node.children.iter().enumerate() {
                let child_is_last = j == child_count - 1;
                self.render_l2_node(child, child_is_last, continuation, w)?;
            }
        }

        // Blank line after each L1 node
        writeln!(w)?;

        Ok(())
    }

    /// Render an L2 node
    fn render_l2_node(
        &self,
        node: &IdeaNode,
        is_last: bool,
        parent_prefix: &str,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        let connector = if is_last { "└── " } else { "├── " };
        let continuation = if is_last { "    " } else { "│   " };

        // Build the full prefix for this L2 node
        let prefix = format!("{}{}", parent_prefix, connector);
        let cont_prefix = format!("{}{}", parent_prefix, continuation);

        // Header line
        let header_text = format!("{}  {}", node.id, node.title);
        self.render_node_header(&prefix, &header_text, false, w)?;

        // Axis + description
        let axis_desc = format!("{} · {}", node.axis, node.description);
        self.render_wrapped_line(&cont_prefix, &axis_desc, false, true, w)?;

        // Deviation rationale
        let deviation = format!("↳ {}", node.deviation_rationale);
        self.render_wrapped_line(&cont_prefix, &deviation, true, false, w)?;

        // Blank line between L2 nodes (except the last one)
        if !is_last {
            self.render_connector_line(&cont_prefix, w)?;
        }

        Ok(())
    }

    /// Render a node header with connector and colored text
    fn render_node_header(
        &self,
        prefix: &str,
        text: &str,
        is_l1: bool,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        if self.color_mode == ColorMode::None {
            writeln!(w, "{}{}", prefix, text)?;
        } else {
            let (fg, bg) = if is_l1 {
                (self.palette.l1_fg, self.palette.l1_bg)
            } else {
                (self.palette.l2_fg, self.palette.l2_bg)
            };
            write!(
                w,
                "{}{}",
                styled_connector(prefix, self.palette.connector_fg, self.palette.dim),
                styled_header(text, fg, bg, self.palette.bold)
            )?;
            // Reset background after header
            if bg.is_some() {
                write!(w, "{}", crossterm::style::SetAttribute(Attribute::Reset))?;
            }
            writeln!(w)?;
        }

        Ok(())
    }

    /// Render a wrapped text line with prefix, handling word-wrap
    fn render_wrapped_line(
        &self,
        prefix: &str,
        text: &str,
        is_dim: bool,
        has_axis: bool,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        // Pad prefix to align with text start (after "X  " in header)
        let text_indent = format!("{}     ", prefix);
        let available_width = self.term_width.saturating_sub(text_indent.len());

        if available_width < 10 {
            // Terminal too narrow for wrapping, just output as-is
            if self.color_mode == ColorMode::None {
                writeln!(w, "{}{}", text_indent, text)?;
            } else {
                self.write_styled_line(&text_indent, text, is_dim, has_axis, w)?;
            }
            return Ok(());
        }

        let lines = wrap_text(text, available_width);
        for (i, line) in lines.iter().enumerate() {
            let line_prefix = &text_indent;
            if self.color_mode == ColorMode::None {
                writeln!(w, "{}{}", line_prefix, line)?;
            } else {
                self.write_styled_line(line_prefix, line, is_dim, has_axis && i == 0, w)?;
            }
        }

        Ok(())
    }

    /// Write a single styled line (axis label colored differently from body)
    fn write_styled_line(
        &self,
        prefix: &str,
        text: &str,
        is_dim: bool,
        has_axis: bool,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        write!(
            w,
            "{}",
            styled_connector(prefix, self.palette.connector_fg, self.palette.dim)
        )?;

        if is_dim {
            writeln!(
                w,
                "{}",
                styled_dim(text, self.palette.axis_fg, self.palette.dim)
            )?;
        } else if has_axis {
            // Split at " · " to color axis label separately
            if let Some(dot_pos) = text.find(" · ") {
                let axis_part = &text[..dot_pos];
                let desc_part = &text[dot_pos + 5..]; // skip " · "
                write!(w, "{}", styled_axis(axis_part, self.palette.axis_fg))?;
                write!(w, "{}", styled_axis(" · ", self.palette.axis_fg))?;
                writeln!(w, "{}", styled_body(desc_part, self.palette.body_fg))?;
            } else {
                writeln!(w, "{}", styled_body(text, self.palette.body_fg))?;
            }
        } else {
            writeln!(w, "{}", styled_body(text, self.palette.body_fg))?;
        }

        Ok(())
    }

    /// Render a bare connector character
    fn render_connector(&self, ch: &str, w: &mut dyn Write) -> std::io::Result<()> {
        if self.color_mode == ColorMode::None {
            write!(w, "{}", ch)?;
        } else {
            write!(
                w,
                "{}",
                styled_connector(ch, self.palette.connector_fg, self.palette.dim)
            )?;
        }
        Ok(())
    }

    /// Render a blank connector line (just the vertical bar at the right indentation)
    fn render_connector_line(&self, prefix: &str, w: &mut dyn Write) -> std::io::Result<()> {
        if self.color_mode == ColorMode::None {
            writeln!(w, "{}", prefix.trim_end())?;
        } else {
            writeln!(
                w,
                "{}",
                styled_connector(
                    prefix.trim_end(),
                    self.palette.connector_fg,
                    self.palette.dim
                )
            )?;
        }
        Ok(())
    }
}

/// Render a full idea tree to a writer
pub fn render_idea_tree(
    result: &IdeationResult,
    color_mode: ColorMode,
    w: &mut dyn Write,
) -> std::io::Result<()> {
    let renderer = TreeRenderer::new(color_mode);
    renderer.render(result, w)
}

/// Word-wrap text to fit within the given width
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            // First word on the line — always add it even if it exceeds width
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::ideation::{IdeaNode, IdeationResult};

    fn make_test_result() -> IdeationResult {
        IdeationResult {
            seed: "Build a social app for pet owners".to_string(),
            sigma: 1.0,
            ideas: vec![
                IdeaNode {
                    id: "A".to_string(),
                    title: "PetConnect Pro".to_string(),
                    description: "A premium networking platform for pet professionals.".to_string(),
                    axis: "audience".to_string(),
                    deviation_rationale: "Shifts from casual owners to professionals.".to_string(),
                    children: vec![
                        IdeaNode {
                            id: "A.1".to_string(),
                            title: "VetLink".to_string(),
                            description: "Veterinary-focused collaboration tool.".to_string(),
                            axis: "mechanism".to_string(),
                            deviation_rationale: "Narrows to veterinary domain.".to_string(),
                            children: vec![],
                        },
                        IdeaNode {
                            id: "A.2".to_string(),
                            title: "PetEdu".to_string(),
                            description: "Educational platform for pet care.".to_string(),
                            axis: "domain".to_string(),
                            deviation_rationale: "Pivots from social to education.".to_string(),
                            children: vec![],
                        },
                        IdeaNode {
                            id: "A.3".to_string(),
                            title: "PawScale".to_string(),
                            description: "Enterprise pet services management.".to_string(),
                            axis: "scale".to_string(),
                            deviation_rationale: "Scales up to enterprise level.".to_string(),
                            children: vec![],
                        },
                    ],
                },
                IdeaNode {
                    id: "B".to_string(),
                    title: "PawPrint Maps".to_string(),
                    description: "Location-based discovery for pet-friendly places.".to_string(),
                    axis: "mechanism".to_string(),
                    deviation_rationale: "Changes from social to location-based.".to_string(),
                    children: vec![],
                },
                IdeaNode {
                    id: "C".to_string(),
                    title: "FurFuture".to_string(),
                    description: "AI-powered pet health prediction platform.".to_string(),
                    axis: "domain".to_string(),
                    deviation_rationale: "Pivots from social to health tech.".to_string(),
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_render_none_mode_has_seed() {
        let result = make_test_result();
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::None, 80);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("seed:"));
        assert!(output.contains("Build a social app for pet owners"));
        assert!(output.contains("sigma=1"));
    }

    #[test]
    fn test_render_none_mode_has_connectors() {
        let result = make_test_result();
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::None, 80);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("├── "));
        assert!(output.contains("└── "));
        assert!(output.contains("│"));
    }

    #[test]
    fn test_render_none_mode_has_all_ids() {
        let result = make_test_result();
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::None, 80);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("A  PetConnect Pro"));
        assert!(output.contains("B  PawPrint Maps"));
        assert!(output.contains("C  FurFuture"));
        assert!(output.contains("A.1  VetLink"));
        assert!(output.contains("A.2  PetEdu"));
        assert!(output.contains("A.3  PawScale"));
    }

    #[test]
    fn test_render_none_mode_has_deviation() {
        let result = make_test_result();
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::None, 80);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("↳ Shifts from casual owners to professionals."));
        assert!(output.contains("↳ Changes from social to location-based."));
    }

    #[test]
    fn test_render_none_mode_has_axis_descriptions() {
        let result = make_test_result();
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::None, 80);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("audience · A premium networking platform"));
        assert!(output.contains("mechanism · Location-based discovery"));
    }

    #[test]
    fn test_render_truecolor_mode_produces_output() {
        let result = make_test_result();
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::Truecolor, 100);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should still contain the text content (with ANSI escape codes around it)
        assert!(output.contains("PetConnect Pro"));
        assert!(output.contains("PawPrint Maps"));
        assert!(output.contains("FurFuture"));
    }

    #[test]
    fn test_render_ansi256_mode_produces_output() {
        let result = make_test_result();
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::Ansi256, 100);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("PetConnect Pro"));
    }

    #[test]
    fn test_wrap_text_basic() {
        let lines = wrap_text("hello world foo bar", 12);
        assert_eq!(lines, vec!["hello world", "foo bar"]);
    }

    #[test]
    fn test_wrap_text_single_long_word() {
        let lines = wrap_text("superlongword", 5);
        assert_eq!(lines, vec!["superlongword"]);
    }

    #[test]
    fn test_wrap_text_empty() {
        let lines = wrap_text("", 80);
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn test_wrap_text_fits_in_one_line() {
        let lines = wrap_text("short", 80);
        assert_eq!(lines, vec!["short"]);
    }

    #[test]
    fn test_render_empty_ideas() {
        let result = IdeationResult {
            seed: "Test".to_string(),
            sigma: 0.5,
            ideas: vec![],
        };
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::None, 80);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("seed:"));
    }

    #[test]
    fn test_l1_last_uses_corner_connector() {
        let result = IdeationResult {
            seed: "Test".to_string(),
            sigma: 1.0,
            ideas: vec![IdeaNode {
                id: "A".to_string(),
                title: "Only".to_string(),
                description: "Desc".to_string(),
                axis: "axis".to_string(),
                deviation_rationale: "Rationale".to_string(),
                children: vec![],
            }],
        };
        let mut buf = Vec::new();
        let renderer = TreeRenderer::with_width(ColorMode::None, 80);
        renderer.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("└── A  Only"));
    }

    #[test]
    fn test_render_function_shortcut() {
        let result = make_test_result();
        let mut buf = Vec::new();
        render_idea_tree(&result, ColorMode::None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("seed:"));
        assert!(output.contains("PetConnect Pro"));
    }
}
