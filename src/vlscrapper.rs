use scraper::{Html, Selector};
use chrono::{Date, Local, NaiveDate, TimeZone};
use regex::Regex;
use crate::{Tekma, TEKME_VL};

pub(crate) async fn pridobi_vl_tekme() -> Vec<Tekma> {
    let mut tekme: Vec<Tekma> = Vec::new();
    println!("{}", "Tekma VL scrapper started");
    // Pridobimo podatke iz spletne strani climbers.si
    let text = reqwest::get("https://climbers.si/").await
                .unwrap()
                .text()
                .await
                .unwrap();

    let document = Html::parse_document(&text);

    // Regex for competitions
    let tekma_class_regex = Regex::new(r"bgtekma\d").unwrap();


    // Selector for table elements
    let selector = Selector::parse(r#"table > tbody"#).unwrap();
    let b_sel = Selector::parse(r#"b"#).unwrap();
    let td_sel = Selector::parse(r#"td"#).unwrap();

    for title in document.select(&selector) {
        let inner = title.inner_html();

        if tekma_class_regex.is_match(inner.as_str()) {
            //urls.push(inner.to_string());
            // Parse data to Tekma struct
            let doc = Html::parse_document(&inner);

            let mut bolds = doc.select(&b_sel);

            // Datum TEKME
            let dt = bolds.next().unwrap().inner_html();
            let naive_date: NaiveDate = NaiveDate::parse_from_str(dt.as_str(), "%d.%m.%Y").unwrap();
            let date: Date<Local> = Local.from_local_date(&naive_date).unwrap();

            // Lokacija TEKME
            let lokacija: String = bolds.next().unwrap().inner_html();

            // Organizator TEKME
            // second td element
            let mut tds = doc.select(&td_sel);
            //let organizator: String = tds..unwrap().inner_html();

            /*let location: String =
            let organizer: String,
            let league: String,*/
            let tekma: Tekma = Tekma {
                date: date,
                location: lokacija,
                organizer: "unknown".to_string(),
                league: "Vzhodna Liga".to_string(),
            };
            tekme.push(tekma);
            //println!("Zadnja: {}", tekme_vl.last().unwrap().date);
        }

    }

    return tekme;
}
