//! SVG loading and analysis operations

use crate::{ImageToolsError, SvgInfo, ViewBox};
use resvg::usvg::{Options, Tree};

/// Parse SVG string and return the parsed tree
pub fn load_svg(svg_data: &str) -> Result<Tree, ImageToolsError> {
    let options = Options::default();
    Tree::from_str(svg_data, &options)
        .map_err(|e| ImageToolsError::svg_error(format!("Failed to parse SVG: {}", e)))
}

/// Get information about an SVG
pub fn get_svg_info(svg_data: &str) -> Result<SvgInfo, ImageToolsError> {
    let tree = load_svg(svg_data)?;
    // In resvg 0.39, size and view_box are fields, not methods
    let size = tree.size;
    let vb = tree.view_box;

    // Get viewBox if different from size
    let view_box = if vb.rect.x() != 0.0
        || vb.rect.y() != 0.0
        || (vb.rect.width() - size.width()).abs() > 0.001
        || (vb.rect.height() - size.height()).abs() > 0.001
    {
        Some(ViewBox {
            x: vb.rect.x() as f64,
            y: vb.rect.y() as f64,
            width: vb.rect.width() as f64,
            height: vb.rect.height() as f64,
        })
    } else {
        None
    };

    Ok(SvgInfo {
        width: size.width() as f64,
        height: size.height() as f64,
        view_box,
    })
}
