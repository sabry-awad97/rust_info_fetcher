use csv::Writer;
use reqwest::Client;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use std::error::Error;
use std::fs::File;

#[derive(Debug)]
pub struct Clinic {
    name: String,
    address: String,
    postcode: Option<String>,
    city: Option<String>,
    phone: Option<String>,
    website: Option<String>,
}

pub struct Scraper {
    base_url: String,
    query: String,
    max_pages: i32,
}

impl Scraper {
    pub fn new(base_url: String, query: String, max_pages: i32) -> Self {
        Self {
            base_url,
            query,
            max_pages,
        }
    }

    pub async fn scrape_pages(&self) -> Result<Vec<Clinic>, Box<dyn Error>> {
        let mut clinics = Vec::new();
        let client = Client::new();

        let pages = (1..=self.max_pages).collect::<Vec<_>>();

        for page_num in pages {
            let results = self.scrape_page(page_num, &client).await?;
            clinics.extend(results);
        }

        Ok(clinics)
    }

    pub async fn scrape_page(
        &self,
        page_num: i32,
        client: &Client,
    ) -> Result<Vec<Clinic>, Box<dyn Error>> {
        println!("Scraping page {}.", page_num);

        let page_url = format!("{}{}?page={}", self.base_url, self.query, page_num);
        let res = client.get(&page_url).send().await?;

        if !res.status().is_success() {
            println!(
                "Failed to fetch page {}. Response status: {:?}",
                page_num,
                res.status()
            );
            return Ok(vec![]);
        }

        let body = res.text().await.unwrap();

        let results = Document::from(&body[..])
            .find(Class("js-entry-card-container"))
            .map(|result| {
                let name = result
                    .find(Name("h2").and(Class("card-info-title")))
                    .next()
                    .unwrap();

                let address = result.find(Class("card-info-address")).next().unwrap();

                let address_text = address.text().trim().to_owned();
                let postcode = address_text
                    .split_whitespace()
                    .find(|w| w.chars().all(|c| c.is_numeric()));

                let city = address_text.trim().split_whitespace().last();

                let phone = result
                    .find(Name("a"))
                    .filter_map(|n| n.attr("href"))
                    .find(|href| href.starts_with("tel:"));

                let website = result
                    .find(Name("a"))
                    .filter_map(|n| n.attr("href"))
                    .find(|href| href.starts_with("http"));

                Clinic {
                    name: name.text().trim().to_owned(),
                    address: address_text.to_owned(),
                    postcode: postcode.map(|p| p.to_owned()),
                    city: city.map(|c| c.to_owned()),
                    phone: phone.map(|p| p.to_owned()),
                    website: website.map(|w| w.to_owned()),
                }
            })
            .collect::<Vec<_>>();

        if results.len() == 0 {
            println!("No results found for page {}.", page_num);
        }

        Ok(results)
    }
}

pub fn write_to_csv(clinics: Vec<Clinic>) -> Result<(), Box<dyn Error>> {
    let file = File::create("clinics.csv")?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&["Name", "Address", "Postcode", "City", "Phone", "Website"])?;

    for clinic in clinics {
        writer.write_record(&[
            &clinic.name,
            &clinic.address,
            &clinic.postcode.unwrap_or_default(),
            &clinic.city.unwrap_or_default(),
            &clinic.phone.unwrap_or_default(),
            &clinic.website.unwrap_or_default(),
        ])?;
    }

    writer.flush()?;

    Ok(())
}
