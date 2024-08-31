use std::process::{Child, Command};

use crate::helper::say;

use serenity::{client::Context, model::prelude::Message};


use thirtyfour::{prelude::*, ChromeCapabilities};
use scraper::{Html, Selector};


const FULL: &str = "https://fortnite.gg/shop?game=br";
const NEW_ONLY: &str = "https://fortnite.gg/shop?game=br&different";
const IMG_PREPEND: &str = "https://fortnite.gg";
const DELIMITER: &str = "^^^";


#[derive(Debug, Clone)]
struct FnShop<'a> {
    url: &'a str,
    name: String,
    time: String,
    price: String,
}


pub async fn fortniteWrapper(msg: &Message, ctx: &Context) {
    
    match runFortnite(&msg, &ctx).await {
        Ok(_) => println!("Successfully accessed Fortnite Shop info"),
        Err(e) =>  println!("Failure: {}", e),
    }
}

async fn runFortnite(msg: &Message, ctx: &Context) -> Result<(), thirtyfour::error::WebDriverError> {

    let mut caps: ChromeCapabilities = DesiredCapabilities::chrome();

    let mut chromedriver: Child = startDriver();

    // TODO: url mode full new
    let _mode: String = String::default();

    caps.add_arg("--headless")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--log-level=1")?;
    caps.add_arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")?;

    let driver: WebDriver = WebDriver::new("http://localhost:4444", caps).await?;

    driver.get(NEW_ONLY).await?;

    let page_source: String = driver.source().await?;

    let form: Option<String> = processDocument(&page_source);

    match form {
        Some(F) => {
            for part in splitMessage(F, DELIMITER).iter() {
                say(msg, ctx, part.to_string()).await
            }         
        },
        None => println!("Err"),
    }

    driver.quit().await?;

    chromedriver.kill().expect("Unable to kill Chromedriver");
    chromedriver.wait().expect("Failed to wait on chromedriver process");

    Ok(())
}

fn processDocument(page_source: &String) -> Option<String> {

    let document: Html = Html::parse_document(page_source);

    let item_selector: Selector = Selector::parse("div.fn-item").unwrap();
    let img_selector: Selector = Selector::parse("img.img").unwrap();
    let name_selector: Selector = Selector::parse("h3.fn-item-name").unwrap();
    let price_selector: Selector = Selector::parse("div.fn-item-price").unwrap();
    let time_selector: Selector = Selector::parse("span.js-bundle-countdown").unwrap();

    let mut item_list: Vec<FnShop> = vec![];
    item_list.reserve(50);

    for item_div in document.select(&item_selector) {
        // Get image source
        let img_src = item_div
            .select(&img_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .unwrap_or("No Image");

        // Get item name
        let name = item_div
            .select(&name_selector)
            .next()
            .map(|n| n.text().collect::<Vec<_>>().join(""))
            .unwrap_or_else(|| "No Name".to_string());

        // Get item price
        let price = item_div
            .select(&price_selector)
            .next()
            .map(|p| p.text().collect::<Vec<_>>().join(""))
            .unwrap_or_else(|| "No Price".to_string());

        let time = item_div
            .select(&time_selector)
            .next()
            .map(|t| t.text().collect::<Vec<_>>().join(""))
            .unwrap_or_else(|| "Unknown Time Remaining".to_string());

        
        item_list.push(FnShop { 
            url: img_src,
            name: name,
            time: time,
            price: price 
        });        
    }

    formatItems(item_list)
}



fn formatItems(items: Vec<FnShop>) -> Option<String> {
    let mut form = String::new();

    for item in items {
        let formatted_item = format!(
            "{}\n\n**Name:** {}\n**Price:** {}\n**Time Remaining:** {}\n{}\n",
            format!("{}/{}", IMG_PREPEND, item.url),
            item.name,
            item.price,
            item.time,
            DELIMITER
        );

        form.push_str(&formatted_item);
    }

    if form.len() != 0 {
        Some(form)
    } else {
        None
    }


}

fn startDriver() -> Child {
    Command::new("chromedriver")
        .arg("--port=4444")
        .spawn()
        .expect("Failed to start chromedrive")
}

fn splitMessage(message: String, delimiter: &str) -> Vec<String> {
    let sections: Vec<&str> = message.split(delimiter).collect();

    // Create a new Vec to hold the split messages
    let mut messages = Vec::new();

    for section in sections {
        if !section.trim().is_empty() {
            messages.push(section.trim().to_string());
        }
    }


    messages
}








