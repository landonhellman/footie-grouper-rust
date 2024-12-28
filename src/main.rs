extern crate chrono;
extern crate csv;
extern crate serde;
extern crate tokio;
extern crate reqwest;

use csv::{ReaderBuilder, Writer};
use serde::Deserialize;
use std::collections::VecDeque;
use std::error::Error;
use chrono::Local;

#[derive(Debug, Deserialize, Default, Clone, serde::Serialize)]
struct NominatimResponse {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize, Clone)]
struct Person {
    id: String,
    name: String,
    preference_1: f64,
    preference_2: f64,
    preference_3: f64,

    pronouns: String, // 1 for he/him, 2 for she/her, 3 for anything else
    #[serde(default)] pronouns_id: f64,

    residential_college: String,

    difficulty: String, // 1, 2, 3, and 4 for difficulty
    #[serde(default)] difficulty_id: f64,

    days: String,
    #[serde(default)] days_id: f64, // 1 for no day hikes, 2 for day hikes, 1.1 for indifferent

    arts: String,
    #[serde(default)] arts_id: f64, // same as day hikes

    food: String,
    #[serde(default)] food_id: f64, // basically true or false

    location: String,
    #[serde(default)] location_id: NominatimResponse, // weights ASSIGNED based off of geographic location.

    school: String,
    #[serde(default)] school_id: f64, // 1 for public and magnet, 2 for private
}

fn assign_difficulty(footie: &mut Person) {
    let difficulty_string = &footie.difficulty;

    if difficulty_string == "Easy: a mellow trip, though still some challenges!" {
        footie.difficulty_id = 1.0;
    } else if difficulty_string == "Moderate: a few ups and downs, some rough terrain" {
        footie.difficulty_id = 2.0;
    } else if difficulty_string == "Strenuous: some ups and downs, some rough terrain" {
        footie.difficulty_id = 3.0;
    } else {
        footie.difficulty_id = 4.0;
    }
}

fn assign_pronouns(footie: &mut Person) {
    let pronoun_string = &footie.pronouns;

    if pronoun_string == "he/him" || pronoun_string == "he/they" {
        footie.pronouns_id = 1.0;
    } else if pronoun_string == "she/her" || pronoun_string == "she/they" {
        footie.pronouns_id = 2.0;
    } else {
        footie.pronouns_id = 3.0;
    }
}

fn assign_days(footie: &mut Person) {
    let day_string = &footie.days;

    if day_string == "Yes, I am interested in day hikes only" {
        footie.days_id = 1.0;
    } else if day_string == "I am NOT interested in day hikes." {
        footie.days_id = 2.0;
    } else {
        footie.days_id = 1.8;
    }
}

fn assign_arts(footie: &mut Person) {
    let art_string = &footie.arts;

    if art_string == "Yes, I am interested in the arts-focused trips only" {
        footie.arts_id = 1.0;
    } else if art_string == "I am NOT interested in the arts-focused trips." {
        footie.arts_id = 2.0;
    } else {
        footie.arts_id = 1.8;
    }
}

fn assign_food(footie: &mut Person) {
    let food_string = &footie.food;

    if food_string == "I have no dietary restrictions or preferences" || food_string == "Other requirement" {
        footie.food_id = 1.0;
    } else if food_string.contains("Allergic to peanuts") || food_string.contains("Allergic to tree nuts")
        || food_string.contains("nut") { // NUT ALLERGY!
        footie.food_id = 2.0;
    } else {
        footie.food_id = 1.0;
    }
}

async fn assign_location(footie: &mut Person) {
    let town_name = &footie.location;
    match get_coordinates(town_name).await {
        Ok(coordinates) => {
            footie.location_id = coordinates;
        }
        Err(_) => {
            footie.location_id = NominatimResponse { lat: 0.0, lon: 0.0 };
        }
    }
}

async fn get_coordinates(location: &str) -> Result<NominatimResponse, Box<dyn Error>> {
    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&addressdetails=1",
        location
    );

    let response: Vec<NominatimResponse> = reqwest::get(&url)
        .await?
        .json()
        .await?;

    if response.is_empty() {
        eprintln!("No coordinates found for location: {}", location);
        Err(Box::new(GeocodingError::NotFound) as Box<dyn Error>)
    } else {
        Ok(response[0].clone()) 
    }
}

#[derive(Debug)]
pub enum GeocodingError {
    NotFound,
}

impl std::fmt::Display for GeocodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GeocodingError::NotFound => write!(f, "Location not found"),
        }
    }
}

impl std::error::Error for GeocodingError {}

async fn read_csv(file_path: &str) -> Result<Vec<Person>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(file_path)?;
    let mut people: Vec<Person> = Vec::new();

    for result in rdr.deserialize() {
        let mut record: Person = result?;
        assign_pronouns(&mut record);
        assign_difficulty(&mut record);
        assign_days(&mut record);
        assign_arts(&mut record);
        assign_food(&mut record);
        assign_location(&mut record).await;
        people.push(record);
    }
    Ok(people)
}

fn calculate_similarity(p1: &Person, p2: &Person) -> f64 {
    let diff_1 = p1.preference_1 - p2.preference_1;
    let diff_2 = p1.preference_2 - p2.preference_2;
    let diff_3 = p1.preference_3 - p2.preference_3;
    let diff_4 = p1.pronouns_id - p2.pronouns_id;

    (diff_1.powi(2) + diff_2.powi(2) + diff_3.powi(2) + diff_4.powi(2)).sqrt()
}

fn group_people(people: Vec<Person>, group_size: usize) -> Vec<Vec<Person>> {
    let mut people = people;
    people.sort_by(|a, b| {
        let sim_a = a.preference_1.powi(2) + a.preference_2.powi(2) + a.preference_3.powi(2) + a.pronouns_id.powi(2);
        let sim_b = b.preference_1.powi(2) + b.preference_2.powi(2) + b.preference_3.powi(2) + b.pronouns_id.powi(2);
        sim_a.partial_cmp(&sim_b).unwrap()
    });

    let mut groups: Vec<Vec<Person>> = Vec::new();
    let mut group: VecDeque<Person> = VecDeque::new();

    for person in people {
        group.push_back(person);
        if group.len() == group_size {
            groups.push(group.iter().cloned().collect());
            group.clear();
        }
    }

    if !group.is_empty() {
        groups.push(group.iter().cloned().collect());
    }

    groups
}

// Write the grouped people to a CSV file
fn write_groups_to_csv(groups: Vec<Vec<Person>>, file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(file_path)?;

    let mut group_number = 1;
    for group in groups {
        for person in group {
            wtr.serialize((
                group_number,
                person.id,
                person.name,
                person.pronouns,
                person.pronouns_id,
                person.residential_college,
                person.difficulty,
                person.days,
                person.arts,
                person.food,
                person.location,
                person.school,
                person.preference_1,
                person.preference_2,
                person.preference_3,
            ))?;
        }
        group_number += 1;
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Runtime::new()?;

    let input_file = "/Users/landonhellman/Documents/footie-grouper-rust/examples/example.csv";
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let output_file_name = format!("/Users/landonhellman/Documents/footie-grouper-rust/outputs/exampleOutput_{}.csv", timestamp);
    let output_file: &str = &output_file_name;

    runtime.block_on(async {
        let people = read_csv(input_file).await?;
        let groups = group_people(people, 8);
        write_groups_to_csv(groups, output_file)?;
        println!("Grouping complete and saved to {}", output_file);
        Ok(())
    })
}
