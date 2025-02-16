use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use google_places_api::GooglePlacesAPI;
use google_places_api::types::constants::place::Location;
use google_places_api::types::constants::{LocationBias, PlaceSearchPlace, PlaceSearchPlaceFields, PlaceTypes};
use google_places_api::types::NearbySearchResult;
use google_places_api::endpoints::find_place::FindPlace;
use geo::{algorithm::contains::Contains, prelude::BoundingRect, Coord, Polygon, Rect, ToRadians};
use futures::{stream, StreamExt};

pub struct PolyFillSearch {
    query_text : Option<String>,
    query_type: Option<PlaceTypes>,
    api: GooglePlacesAPI,
    polygon: Option<Vec<Location>>
}

impl PolyFillSearch {
    pub fn new() -> Self {
        PolyFillSearch {
            query_text: None,
            query_type: None,
            polygon: None,
            api: GooglePlacesAPI::new()
        }
    }
    pub fn with_polygon(mut self, polygon: Vec<Location>) -> Self {
        self.polygon = Some(polygon);
        self
    }
    pub fn with_keywords(mut self, query_text: String) -> Self {
        self.query_text = Some(query_text);
        self
    }

    pub fn with_type(mut self, query_filters: PlaceTypes) -> Self {
        self.query_type = Some(query_filters);
        self
    }
    pub async fn execute(self) -> Vec<String> {
        let polygon = self.polygon.expect("Please provide a polygon");
        if polygon.len() < 3 {
            panic!("A valid polygon must have at least 3 points.");
        }

        let geo_polygon = Polygon::new(
            polygon.iter().map(|p| Coord { x: p.lon.unwrap(), y: p.lat.unwrap() }).collect(),
            vec![],
        );

        let bbox = geo_polygon.bounding_rect().expect("Failed to compute bounding box");
        let min_lat = bbox.min().y;
        let max_lat = bbox.max().y;
        let min_lng = bbox.min().x;
        let max_lng = bbox.max().x;

        let radius = (110.574 * fabs(min_lat - max_lat)) / 16.0;
        let earth_radius_km = 6371.0;
        let lat_spacing = radius / earth_radius_km * (180.0 / std::f64::consts::PI);
        let lng_spacing = lat_spacing / (min_lat.to_radians().cos());

        let mut grid_points = vec![];
        let mut lat = min_lat;
        while lat <= max_lat {
            let mut lng = min_lng;
            while lng <= max_lng {
                let point = Coord { x: lng, y: lat };
                if geo_polygon.contains(&point) {
                    grid_points.push(Location::new(lat, lng));
                }
                lng += lng_spacing;
            }
            lat += lat_spacing;
        }

        // println!("Running query for {} points, bounding box is {} {} {} {}", grid_points.len(),
        //          min_lat, max_lat, min_lng, max_lng);

        let api = Arc::new(self.api);

        let results = stream::iter(grid_points)
            .map(|loc| {
                let api = Arc::clone(&api);
                let query_type = self.query_type;
                let query_text = self.query_text.clone();
                async move {
                    let mut place_search = api.place_search();
                    let mut result = place_search.nearby_search();
                    let mut result = result.with_location(loc).with_radius( radius * 1000.0);

                    if let Some(ty) = query_type {
                        result = result.with_type(ty);
                    } else if let Some(text) = &query_text {
                        result = result.with_keyword(text);
                    } else {
                        panic!("Please provide at least a type or keyword.");
                    }
                    let r = result.execute(3).await;
                    Some(r.unwrap().get_result().places.clone())
                }
            })
            .buffer_unordered(16);

        let final_results = Arc::new(Mutex::new(Vec::new()));

        results.for_each( |b|async  {
            let final_results = Arc::clone(&final_results);
            if let Some(response) = b {
                let mut results = final_results.lock().unwrap();
                results.extend(response.iter().map(|p| p.id.clone()));
            } else {
                println!("Error fetching places.");
            }
        }).await;

        // Extract and return the final vector of place IDs
        let final_results = final_results.lock().unwrap().clone();
        final_results


    }
}

fn fabs(p0: f64) -> f64{
    if p0 < 0.0 {
        -p0
    } else {
        p0
    }
}

