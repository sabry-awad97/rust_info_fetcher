use std::error::Error;

use swiss_info_clinic_scraper::{write_to_csv, Scraper};

const MAX_PAGES: i32 = 100;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let scraper = Scraper::new(
        "https://www.local.ch/en/q".to_owned(),
        "/Switzerland/clinique".to_owned(),
        MAX_PAGES,
    );

    // let clinics = scraper.scrape_pages().await?;
    let clinics = scraper.scrape_pages_parallel().await?;

    if let Err(err) = write_to_csv(clinics) {
        println!("Failed to write to csv. Error: {}", err);
    }

    Ok(())
}
