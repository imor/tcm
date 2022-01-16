use std::fs;
use std::path::Path;
use crate::frontmatter::Frontmatter;
use headless_chrome::{Browser, protocol::page::ScreenshotFormat, LaunchOptionsBuilder};

pub(crate) fn create_images_from_template(frontmatter_list: Vec<Frontmatter>, chrome_path: &Path, port: u16) {
    let url = &format!("http://localhost:{}/", port);
    let options = LaunchOptionsBuilder::default()
        .path(Some(chrome_path.to_path_buf()))
        .build().expect("Failed to build LaunchOptions");
    let browser = match Browser::new(options) {
        Ok(browser) => browser,
        Err(e) => {
            eprintln!("Failed to create browser: {}", e);
            return;
        }
    };

    // Wait for tab to open
    let tab = match browser.wait_for_initial_tab() {
        Ok(tab) => tab,
        Err(e) => {
            eprintln!("Failed to wait for initial tab: {}", e);
            return;
        }
    };

    for frontmatter in frontmatter_list {
        println!("Saving {}", frontmatter.image_path.display());

        //Wait for template to open
        if let Err(e) = tab.navigate_to(url) {
            eprintln!("Error while navigating to url: {}", e);
            continue;
        }

        if let Err(e) = tab.wait_until_navigated() {
            eprintln!("Error while waiting for navigation: {}", e);
            continue;
        }

        //Update text in template
        let title = &frontmatter.title;
        let subtitle = &frontmatter.description;

        let title = title.replace("'", "\\'");
        let subtitle = subtitle.replace("'", "\\'");

        let js_expr = format!(r"
            setText('{}', '{}');
            fitText();
        ", title, subtitle);

        if let Err(e) = tab.evaluate(&js_expr, true){
            eprintln!("Error while evaluating js: {}", e);
            continue;
        }

        //Render and save
        let container = match tab.find_element("#container") {
            Ok(container) => container,
            Err(e) => {
                eprintln!("Error while finding #container element: {}", e);
                continue;
            }
        };

        let viewport = match container.get_box_model() {
            Ok(box_model) => box_model.border_viewport(),
            Err(e) => {
                eprintln!("Error while getting #container box model: {}", e);
                continue;
            }
        };

        let png_data = match tab.capture_screenshot(ScreenshotFormat::PNG, Some(viewport), true) {
            Ok(png_data) => png_data,
            Err(e) => {
                eprintln!("Error while capturing screenshot: {}", e);
                continue;
            }
        };
        if let Err(e) = fs::write(&frontmatter.image_path, &png_data) {
            eprintln!("Error while capturing screenshot: {}", e);
        }
    }
}