import pandas as pd
import asyncio
import aiohttp
from datetime import datetime
from math import sqrt
from typing import List, Dict
import certifi
import ssl
import geopy
from geopy.geocoders import Nominatim
ctx = ssl.create_default_context(cafile=certifi.where())
geopy.geocoders.options.default_ssl_context = ctx

class Person:
    def __init__(self, **kwargs):
        self.id = kwargs.get("id", "")
        self.name = kwargs.get("name", "")
        self.preference_1 = kwargs.get("preference_1", 0.0)
        self.preference_2 = kwargs.get("preference_2", 0.0)
        self.preference_3 = kwargs.get("preference_3", 0.0)
        self.pronouns = kwargs.get("pronouns", "")
        self.pronouns_id = 0.0
        self.residential_college = kwargs.get("residential_college", "")
        self.difficulty = kwargs.get("difficulty", "")
        self.difficulty_id = 0.0
        self.days = kwargs.get("days", "")
        self.days_id = 0.0
        self.arts = kwargs.get("arts", "")
        self.arts_id = 0.0
        self.food = kwargs.get("food", "")
        self.food_id = 0.0
        self.location = kwargs.get("location", "")
        self.location_id = {"lat": 0.0, "lon": 0.0}
        self.school = kwargs.get("school", "")

def assign_difficulty(person):
    mapping = {
        "Easy: a mellow trip, though still some challenges!": 1.0,
        "Moderate: a few ups and downs, some rough terrain": 2.0,
        "Strenuous: some ups and downs, some rough terrain": 3.0,
        "Very strenuous: lots of hiking up and down, rough terrain": 4.0 
    }
    person.difficulty_id = mapping.get(person.difficulty, 4.0)

def assign_pronouns(person):
    mapping = {
        "he/him": 1.0,
        "he/they": 1.0,
        "she/her": 2.0,
        "she/they": 2.0,
    }
    person.pronouns_id = mapping.get(person.pronouns, 3.0)

def assign_days(person):
    mapping = {
        "Yes, I am interested in day hikes only": 1.0,
        "I am NOT interested in day hikes.": 2.0,
    }
    person.days_id = mapping.get(person.days, 1.8)

def assign_arts(person):
    mapping = {
        "Yes, I am interested in the arts-focused trips only": 1.0,
        "I am NOT interested in the arts-focused trips.": 2.0,
    }
    person.arts_id = mapping.get(person.arts, 1.8)

def assign_food(person):
    if "nut" in str(person.food).lower():
        person.food_id = 2.0
    else:
        person.food_id = 1.0

def assign_location(person):
    geolocator = Nominatim(user_agent="footie-grouper-rust/1.0 landon.hellman@yale.edu")
    
    try:
    # Geocode the city name
        location = geolocator.geocode(person.location)
        
        if location:
            # Return the latitude and longitude as a tuple
            person.location_id = {"lat": location.latitude, "lon": location.longitude}
        else:
            return None
    
    except Exception as e:
        print(f"Error: {e}")
        return None

def group_people(people: List[Person], group_size: int) -> List[List[Person]]:
    people.sort(key=lambda p: sum([
        p.preference_1 ** 2, p.preference_2 ** 2, p.preference_3 ** 2, p.pronouns_id ** 2
    ]))
    groups = [people[i:i + group_size] for i in range(0, len(people), group_size)]
    return groups

def write_groups_to_csv(groups: List[List[Person]], file_path: str):
    rows = []
    group_number = 1
    for group in groups:
        for person in group:
            rows.append({
                "Group": group_number,
                "ID": person.id,
                "Name": person.name,
                "Pronouns": person.pronouns,
                "Pronouns ID": person.pronouns_id,
                "Residential College": person.residential_college,
                "Difficulty": person.difficulty,
                "Days": person.days,
                "Arts": person.arts,
                "Food": person.food,
                "Location": person.location,
                "LocationID": person.location_id,
                "School": person.school,
                "Preference 1": person.preference_1,
                "Preference 2": person.preference_2,
                "Preference 3": person.preference_3,
            })
        group_number += 1
    pd.DataFrame(rows).to_csv(file_path, index=False)

def main(input_file: str, output_file: str):
    df = pd.read_csv(input_file)
    people = [Person(**row) for _, row in df.iterrows()]

    tasks = []
    totalPeople = 0
    for person in people:
        totalPeople += 1
        print("Assigning Person " + str(totalPeople))
        assign_difficulty(person)
        assign_pronouns(person)
        assign_days(person)
        assign_arts(person)
        assign_food(person)
        tasks.append(assign_location(person))

    print("Assigning Groups...")
    groups = group_people(people, 8)
    write_groups_to_csv(groups, output_file)

if __name__ == "__main__":
    input_file = "examples/example.csv"
    timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    output_file = f"outputs/exampleOutput_{timestamp}.csv"
    main(input_file, output_file)
