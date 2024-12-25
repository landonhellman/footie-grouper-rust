extern crate csv;
extern crate serde;
use csv::{ReaderBuilder, Writer};
use serde::Deserialize;
use std::collections::VecDeque;
use std::error::Error;

#[derive(Debug, Deserialize, Clone)]
struct Person {
    id: String,
    name: String,
    preference_1: f64,
    preference_2: f64,
    preference_3: f64,
}

fn read_csv(file_path: &str) -> Result<Vec<Person>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(file_path)?;
    let mut people: Vec<Person> = Vec::new();
    
    for result in rdr.deserialize() {
        let record: Person = result?;
        people.push(record);
    }
    
    Ok(people)
}

fn calculate_similarity(p1: &Person, p2: &Person) -> f64 {
    // For simplicity, we're using Euclidean distance between preferences.
    let diff_1 = p1.preference_1 - p2.preference_1;
    let diff_2 = p1.preference_2 - p2.preference_2;
    let diff_3 = p1.preference_3 - p2.preference_3;
    
    // Return the Euclidean distance as the similarity metric
    (diff_1.powi(2) + diff_2.powi(2) + diff_3.powi(2)).sqrt()
}

fn group_people(people: Vec<Person>, group_size: usize) -> Vec<Vec<Person>> {
    // Sort people based on the similarity (using Euclidean distance, so lower is more similar)
    let mut people = people;
    people.sort_by(|a, b| {
        let sim_a = a.preference_1.powi(2) + a.preference_2.powi(2) + a.preference_3.powi(2);
        let sim_b = b.preference_1.powi(2) + b.preference_2.powi(2) + b.preference_3.powi(2);
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

fn write_groups_to_csv(groups: Vec<Vec<Person>>, file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(file_path)?;

    // Assuming each person in a group should be written to a row with the group number
    let mut group_number = 1;
    for group in groups {
        for person in group {
            wtr.serialize((
                group_number,
                person.id,
                person.name,
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
    let input_file = "/Users/landonhellman/Documents/rust-blockchain-example/examples/example.csv"; // CSV input file with people
    let output_file = "/Users/landonhellman/Documents/rust-blockchain-example/outputs/exampleOutput.csv"; // CSV output file with grouped people
    
    // Step 1: Read the CSV file
    let people = read_csv(input_file)?;

    // Step 2: Group people based on similarity in preferences (group of 7-8 people)
    let groups = group_people(people, 8);

    // Step 3: Write the grouped people back to a CSV
    write_groups_to_csv(groups, output_file)?;

    println!("Grouping complete and saved to {}", output_file);

    Ok(())
}
