use std::fs;
use std::path::Path;
use crate::frontmatter::Frontmatter;
use headless_chrome::{Browser, protocol::page::ScreenshotFormat, LaunchOptionsBuilder};

pub(crate) fn create_images_from_template(frontmatter_list: Vec<Frontmatter>, chrome_path: &Path, port: u16) {
    let url = &format!("http://localhost:{}/", port);
    let options = LaunchOptionsBuilder::default()
        .path(Some(chrome_path.to_path_buf()))
        .build().expect("Failed to build LaunchOptionsBuilder");
    let browser = Browser::new(options).expect("Failed to create browser");

    // Wait for tab to open
    let tab = browser.wait_for_initial_tab().expect("Failed to wait for initial tab");

    for frontmatter in frontmatter_list {
        println!("Saving {}", frontmatter.image_path.display());

        //Wait for template to open
        match tab.navigate_to(url) {
            Err(e) => {
                eprintln!("Error while navigating to url: {}", e);
                continue;
            }
            _ => {}
        }

        match tab.wait_until_navigated() {
            Err(e) => {
                eprintln!("Error while waiting for navigation: {}", e);
                continue;
            }
            _ => {}
        }

        //Update text in template
        let title = &frontmatter.title;
        let subtitle = &frontmatter.description;
        let func = format!(r"
            let text = {{
                title: '{}',
                subtitle: '{}'
            }};
            setText(text);
            fitText();
        ", title.replace("'", "\\'"), subtitle.replace("'", "\\'"));

        match tab.evaluate(&func, true){
            Err(e) => {
                eprintln!("Error while evaluating js: {}", e);
                continue;
            }
            _ => {}
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
        match fs::write(&frontmatter.image_path, &png_data) {
            Err(e) => eprintln!("Error while capturing screenshot: {}", e),
            _ => {}
        }
    }
}