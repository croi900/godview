mod polyfill;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::polyfill::PolyFillSearch;
    use std::collections::HashSet;
    use google_places_api::types::constants::{PlaceSearchPlaceFields, PlaceTypes};
    use google_places_api::types::constants::place::Location;

    #[tokio::test]
    async fn test_types() {

        let mut polygon: Vec<Location> = Vec::new();
        for line in std::fs::read_to_string("Cluj-Napoca.txt").unwrap().lines() {
            if line.len() == 0 {
                continue;
            }
            let mut coords = line.split_whitespace();
            polygon.push(Location { // coords are stored long lat
                lon: Some(coords.next().unwrap().parse::<f64>().unwrap()),
                lat: Some(coords.next().unwrap().parse::<f64>().unwrap()),
            });
        }
        println!("{:?}",polygon);
        //
        let result = PolyFillSearch::new()
            .with_type(PlaceTypes::Hospital)
            .with_polygon(polygon)
            .execute()
            .await;

        println!("Found {} places", result.len());

        let unique: HashSet<String> = HashSet::from_iter(result);

        println!("Found {} unique places", unique.len());
    }
}

