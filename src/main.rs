use warp::Filter;
use headless_chrome::{Browser, Tab};
use std::fs;
use std::sync::{Arc, Mutex, MutexGuard};
use tera::{Tera, Context};
use pulldown_cmark::{Parser, html};

const TAB_POOL_SIZE: usize = 5;

struct ChromeRenderer {
    chrome_renderer_tab: Arc<Tab>,
}
impl ChromeRenderer {
    pub fn with_chrome_tab(tab: &Arc<Tab>) -> Self {
        tab.navigate_to("about:blank").unwrap().wait_until_navigated().unwrap();

        Self {
            chrome_renderer_tab: tab.clone(),
        }
    }
    pub fn clear(&self) {
        self.chrome_renderer_tab.evaluate("document.body.innerHTML = '';", false).unwrap();
    }
    pub fn render_html(&self, html: &str) -> Vec<u8> {
        // Clear the document body
            
        // Set the combined HTML content using JavaScript
        self.chrome_renderer_tab.evaluate(&format!("document.write(`{}`);", html), false).unwrap();
        
        // Generate PDF from the rendered HTML
        let pdf_data = self.chrome_renderer_tab.print_to_pdf(None).unwrap();
        // fs::write("./output.pdf", pdf_data.clone()).unwrap();
        // println!("PDF successfully created from Markdown content.");
        pdf_data
    }
}

struct RendererPool {
    pool: Mutex<Vec<ChromeRenderer>>
}
impl RendererPool {
    pub fn from_pool(pool: Vec<ChromeRenderer>) -> Self {
        Self {
            pool: Mutex::new(pool)
        }
    }
    pub fn with_browser(browser: Arc<Browser>) -> Self {
        let tab_pool: Vec<_> = (0..TAB_POOL_SIZE)
        .map(|_| {
            let tab = browser.new_tab().unwrap();
            ChromeRenderer::with_chrome_tab(&tab)
        })
        .collect();
        Self::from_pool(tab_pool)
    }

    pub fn get_next_renderer(&self) -> Result<ChromeRenderer, ()> {
        let pool = self.pool.lock().unwrap();
        
    }

}
#[tokio::main]
async fn main() {
    // Create a shared browser instance
    let browser = Arc::new(Browser::default().unwrap());
    
    let renderer_pool = Arc::new(RendererPool::with_browser(browser));
    
    // Initialize Tera template engine
    let tera = Tera::new("assets/*.html").unwrap();
    
    let hello = warp::path!("hello" / String)
        .map(move |name| {
            let render_pool = renderer_pool.clone();
            let renderer = get_next_available_tab(&tab_pool).unwrap();
            
            // Load CSS file
            let css_content = fs::read_to_string("assets/styles.css").unwrap();
            
            // Convert Markdown to HTML
            let parser = Parser::new(format!("Hello {}", name).as_str());
            let mut html_content = String::new();
            html::push_html(&mut html_content, parser);
            
            // Render HTML template with Tera
            let mut context = Context::new();
            context.insert("content", &html_content);
            let html_template = tera.render("template.html", &context).unwrap();
            
            // Combine HTML and CSS
            let combined_html = format!(
                "<!DOCTYPE html>
                <html>
                <head>
                <style>{}</style>
                </head>
                <body>
                {}
                </body>
                </html>",
                css_content, html_template
            );
            
            let pdf_data = renderer.render_html(html_template);
            
            pdf_data.clone()
        });

    println!("listening on http://localhost:3030");

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
