use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_image_tools", "runtime:image_tools")
        .ts_path("ts/init.ts")
        .ops(&[
            // PNG operations
            "op_image_png_info",
            "op_image_png_load",
            "op_image_png_save",
            "op_image_png_optimize",
            // SVG operations
            "op_image_svg_info",
            "op_image_svg_load",
            // WebP operations (for app asset optimization)
            "op_image_webp_encode",
            "op_image_webp_decode",
            "op_image_webp_info",
            // Convert operations
            "op_image_svg_to_png",
            "op_image_png_to_ico",
            "op_image_ico_extract",
            "op_image_favicon_create",
            "op_image_png_to_webp",
            // Transform operations
            "op_image_resize",
            "op_image_scale",
            "op_image_crop",
            "op_image_rotate",
            "op_image_flip",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build runtime_image_tools extension");
}
