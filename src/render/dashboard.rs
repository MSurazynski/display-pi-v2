use chrono::Local;
use resvg::usvg::{self, fontdb};
use crate::render::dither::convert_pixmap_to_epaper_pallete;

struct SvgCanvas {
    width: u32,
    height: u32,
    background: String,
    components: Vec<String>,
    y_cursor: u32,
    space_y: u32,
    margin_x: u32,
}

impl SvgCanvas {
    fn new() -> Self {
        Self {
            width: 480,
            height: 800,
            background: String::from("#fbefc2"),
            components: Vec::new(),
            y_cursor: 0,
            space_y: 20,
            margin_x: 40,
        }
    }

    fn add_header(&mut self, title: &str) -> &mut Self {
        let header_height = 80;

        self.components.push(format!(
            r##"<text x="{}" y="{}" font-family="Yanone Kaffeesatz" font-size="35" font-weight="700" fill="#000000">{}</text>"##,
            self.margin_x, 
            header_height, title
        ));

        self.y_cursor += header_height + self.space_y;

        self
    }

    fn build(&self) -> String {
        let body = self.components.join("\n");
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
  <defs>
    <style>@import url('https://fonts.googleapis.com/css2?family=Yanone+Kaffeesatz:wght@700');</style>
  </defs>
  <rect width="{w}" height="{h}" fill="{bg}" />
  {body}
</svg>"##,
            w = self.width,
            h = self.height,
            bg = self.background,
        )
    }
}


pub fn render_dashboard_svg() {

    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let now = Local::now();
    let output_path = project_root.join("output").join("dashboard.png");
    let font_path = include_bytes!("../../fonts/YanoneKaffeesatz-VariableFont_wght.ttf");

    let title = now.format("%A, %d %B").to_string();

    let mut canvas = SvgCanvas::new();
    canvas.add_header(&title);

    let svg_string = canvas.build();

    // Load font
    let mut fontdb = fontdb::Database::new();
    fontdb.load_font_data(font_path.to_vec());

    // Parse svg
    let options = usvg::Options {
        fontdb: std::sync::Arc::new(fontdb),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&svg_string, &options).unwrap();

    // Render to png
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Convert to epaper pallete
    convert_pixmap_to_epaper_pallete(&mut pixmap);

    // Save
    pixmap.save_png(&output_path).unwrap();
}
