*Godview* is a library that allows you to efficiently and cost effectively 
search for places based on a configurable and extensible set of criteria,
using the Google Places API.

Examples:

* Find a place by type
```rust
// For this example we load the long and lat 
// bounds fo the polygon from a text file
// but they can be obtained either way

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

let result = PolyFillSearch::new()
    .with_type(PlaceTypes::Hospital)
    .with_polygon(polygon)
    .execute()
    .await;

println!("Found {} places", result.len());

let unique: HashSet<String> = HashSet::from_iter(result);

println!("Found {} unique places", unique.len());
```




Implemented strategies:

* PolyFill

Planned strategies:

* Convex Hull Dynamic BFS 
* Branch Pruning Divide et Impera 
* 