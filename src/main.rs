use standing::{app, copy};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(copy::APP_NAME)
            .with_app_id("standing")
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([960.0, 640.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/standing.png"))
                    .expect("embedded icon is a valid PNG"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        copy::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(app::Standing::new(cc)))),
    )
}
