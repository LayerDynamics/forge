//! Image and badge generation utilities
//!
//! This module provides utilities for generating SVG badges and
//! simple diagrams for documentation.

use std::fmt::Write;

/// Badge style
#[derive(Debug, Clone, Copy, Default)]
pub enum BadgeStyle {
    /// Flat badge (shields.io style)
    #[default]
    Flat,
    /// Flat square badge
    FlatSquare,
    /// Plastic badge
    Plastic,
    /// For the badge style
    ForTheBadge,
}

/// Badge colors
#[derive(Debug, Clone)]
pub enum BadgeColor {
    /// Success green
    Green,
    /// Warning yellow
    Yellow,
    /// Error red
    Red,
    /// Info blue
    Blue,
    /// Gray (neutral)
    Gray,
    /// Custom hex color
    Custom(String),
}

impl BadgeColor {
    /// Get hex color code
    pub fn hex(&self) -> &str {
        match self {
            BadgeColor::Green => "#4c1",
            BadgeColor::Yellow => "#dfb317",
            BadgeColor::Red => "#e05d44",
            BadgeColor::Blue => "#007ec6",
            BadgeColor::Gray => "#555",
            BadgeColor::Custom(c) => c,
        }
    }
}

/// Generate an SVG badge
pub fn generate_badge(label: &str, value: &str, color: BadgeColor, style: BadgeStyle) -> String {
    let label_width = estimate_text_width(label) + 10;
    let value_width = estimate_text_width(value) + 10;
    let total_width = label_width + value_width;
    let color_hex = color.hex();

    let (rect_style, text_shadow) = match style {
        BadgeStyle::Flat => ("", ""),
        BadgeStyle::FlatSquare => ("rx=\"0\"", ""),
        BadgeStyle::Plastic => (
            "",
            "<linearGradient id=\"smooth\" x2=\"0\" y2=\"100%\"><stop offset=\"0\" stop-color=\"#bbb\" stop-opacity=\".1\"/><stop offset=\"1\" stop-opacity=\".1\"/></linearGradient>",
        ),
        BadgeStyle::ForTheBadge => ("rx=\"0\"", ""),
    };

    let height = match style {
        BadgeStyle::ForTheBadge => 28,
        _ => 20,
    };

    let font_size = match style {
        BadgeStyle::ForTheBadge => 10,
        _ => 11,
    };

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="{height}">
  {text_shadow}
  <rect width="{label_width}" height="{height}" fill="#555" {rect_style}/>
  <rect x="{label_width}" width="{value_width}" height="{height}" fill="{color_hex}" {rect_style}/>
  <g fill="#fff" text-anchor="middle" font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="{font_size}">
    <text x="{label_x}" y="{text_y}">{label}</text>
    <text x="{value_x}" y="{text_y}">{value}</text>
  </g>
</svg>"##,
        total_width = total_width,
        height = height,
        text_shadow = text_shadow,
        label_width = label_width,
        value_width = value_width,
        color_hex = color_hex,
        rect_style = rect_style,
        font_size = font_size,
        label_x = label_width / 2,
        value_x = label_width + value_width / 2,
        text_y = height / 2 + 4,
        label = escape_xml(label),
        value = escape_xml(value),
    )
}

/// Estimate text width in pixels (rough approximation)
fn estimate_text_width(text: &str) -> usize {
    // Approximate character width of ~7px for the default font
    text.len() * 7
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generate a status badge for API coverage
pub fn coverage_badge(covered: usize, total: usize) -> String {
    let percentage = if total > 0 {
        (covered as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let color = if percentage >= 80.0 {
        BadgeColor::Green
    } else if percentage >= 60.0 {
        BadgeColor::Yellow
    } else {
        BadgeColor::Red
    };

    generate_badge(
        "coverage",
        &format!("{:.0}%", percentage),
        color,
        BadgeStyle::Flat,
    )
}

/// Generate a version badge
pub fn version_badge(version: &str) -> String {
    generate_badge("version", version, BadgeColor::Blue, BadgeStyle::Flat)
}

/// Generate a deprecated badge
pub fn deprecated_badge() -> String {
    generate_badge("status", "deprecated", BadgeColor::Red, BadgeStyle::Flat)
}

/// Generate an experimental badge
pub fn experimental_badge() -> String {
    generate_badge(
        "status",
        "experimental",
        BadgeColor::Yellow,
        BadgeStyle::Flat,
    )
}

/// Generate a simple SVG icon
pub fn generate_icon(icon_type: IconType, size: usize, color: &str) -> String {
    let path = match icon_type {
        IconType::Function => "M3 5h18v2H3V5zm0 6h18v2H3v-2zm0 6h18v2H3v-2z",
        IconType::Class => "M4 4h16v4H4V4zm0 8h16v8H4v-8z",
        IconType::Interface => "M4 4h16v16H4V4zm2 2v12h12V6H6z",
        IconType::Enum => "M4 4h16v3H4V4zm0 5h16v3H4V9zm0 5h16v3H4v-3z",
        IconType::Type => "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
        IconType::Variable => "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z",
        IconType::Async => "M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46A7.93 7.93 0 0020 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74A7.93 7.93 0 004 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z",
        IconType::Deprecated => "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z",
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="{color}">
  <path d="{path}"/>
</svg>"#,
        size = size,
        color = color,
        path = path,
    )
}

/// Icon types
#[derive(Debug, Clone, Copy)]
pub enum IconType {
    Function,
    Class,
    Interface,
    Enum,
    Type,
    Variable,
    Async,
    Deprecated,
}

/// Generate a simple diagram showing module structure
pub fn module_diagram(name: &str, items: &[(&str, &str)]) -> String {
    let mut svg = String::new();

    let item_height = 30;
    let padding = 10;
    let width = 200;
    let header_height = 40;
    let total_height = header_height + (items.len() * item_height) + padding * 2;

    writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
        width, total_height
    )
    .ok();

    // Background
    writeln!(
        svg,
        r##"  <rect width="{}" height="{}" fill="#f8f9fa" stroke="#dee2e6" rx="4"/>"##,
        width, total_height
    )
    .ok();

    // Header
    writeln!(
        svg,
        r##"  <rect width="{}" height="{}" fill="#007ec6" rx="4"/>"##,
        width, header_height
    )
    .ok();
    writeln!(
        svg,
        r##"  <text x="{}" y="{}" fill="white" font-family="sans-serif" font-size="14" text-anchor="middle">{}</text>"##,
        width / 2,
        header_height / 2 + 5,
        escape_xml(name)
    )
    .ok();

    // Items
    for (i, (item_name, kind)) in items.iter().enumerate() {
        let y = header_height + padding + (i * item_height);
        let icon_color = match *kind {
            "function" | "op" => "#61afef",
            "class" => "#e5c07b",
            "interface" | "struct" => "#98c379",
            "enum" => "#c678dd",
            "type" => "#56b6c2",
            _ => "#abb2bf",
        };

        writeln!(
            svg,
            r#"  <circle cx="{}" cy="{}" r="6" fill="{}"/>"#,
            padding + 10,
            y + item_height / 2,
            icon_color
        )
        .ok();
        writeln!(
            svg,
            r##"  <text x="{}" y="{}" fill="#333" font-family="sans-serif" font-size="12">{}</text>"##,
            padding + 25,
            y + item_height / 2 + 4,
            escape_xml(item_name)
        )
        .ok();
    }

    writeln!(svg, "</svg>").ok();

    svg
}

/// Generate an inline badge HTML
pub fn inline_badge_html(text: &str, color: &str) -> String {
    format!(
        r#"<span style="display:inline-block;padding:2px 6px;border-radius:3px;background:{};color:white;font-size:12px;">{}</span>"#,
        color,
        escape_xml(text)
    )
}

/// Generate markdown-compatible badge (using shields.io URL format)
pub fn shields_io_badge_url(label: &str, message: &str, color: &str) -> String {
    let label_encoded = urlencoding::encode(label);
    let message_encoded = urlencoding::encode(message);
    format!(
        "https://img.shields.io/badge/{}-{}-{}",
        label_encoded, message_encoded, color
    )
}

/// Generate markdown for a shields.io badge
pub fn shields_io_badge_markdown(label: &str, message: &str, color: &str, alt: &str) -> String {
    let url = shields_io_badge_url(label, message, color);
    format!("![{}]({})", alt, url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_badge() {
        let badge = generate_badge("version", "1.0.0", BadgeColor::Blue, BadgeStyle::Flat);
        assert!(badge.contains("<svg"));
        assert!(badge.contains("1.0.0"));
    }

    #[test]
    fn test_coverage_badge() {
        let badge = coverage_badge(80, 100);
        assert!(badge.contains("80%"));
        assert!(badge.contains("#4c1")); // Green color
    }

    #[test]
    fn test_generate_icon() {
        let icon = generate_icon(IconType::Function, 24, "#333");
        assert!(icon.contains("<svg"));
        assert!(icon.contains("width=\"24\""));
    }

    #[test]
    fn test_shields_io_badge_url() {
        let url = shields_io_badge_url("version", "1.0.0", "blue");
        assert!(url.contains("shields.io"));
        assert!(url.contains("version"));
    }
}
