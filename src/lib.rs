use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::io::BufReader;
use std::collections::HashMap;
use std::fmt;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate memmap;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref TAGS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        for tags in RAW_TAGS.lines() {
            let vec: Vec<_> = tags.split(",").collect();
            m.insert(vec[0], vec[1]);
        }
        m
    };
}

static RAW_TAGS: &'static str = include_str!("tags.csv");

pub struct Config {
    pub filename: String,
    pub output_file: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments: no filename specified");
        }
        let filename = args[1].clone();
        let output_file = match args.len() { //TODO nice argument parsing syntax and logic
            3 => args[2].clone(),
            _ => String::from("output.json"),
        };
        Ok(Config {
            filename,
            output_file,
        })
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Country {
    tag: String,
    provinces: Vec<Province>,
    total_rural_pop: i32,
    total_urban_pop: i32,
    total_wealth_growth: f32,
    total_urban_wealth_growth: f32
}

impl Country {
    fn new(tag: String) -> Country {
        Country {
            tag,
            provinces: Vec::new(),
            total_rural_pop: 0,
            total_urban_pop: 0,
            total_wealth_growth: 0f32,
            total_urban_wealth_growth: 0f32,
        }
    }

    fn add_province(&mut self, prov: Province) {
        self.total_rural_pop += prov.rural_pop;
        self.total_urban_pop += prov.urban_pop;
        self.total_wealth_growth += prov.wealth_total_growth;
        self.total_urban_wealth_growth += prov.wealth_urban_growth;
        self.provinces.push(prov);
    }
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Country {}", TAGS.get::<str>(&self.tag).unwrap())
    }
}

///Wealth as growth is split 60% / 40% between the variable called "growth" and the "to_farm"
/// variable. wealth_from_mines, wealth_urban_from_pop, wealth_urban_from_trade, and
/// wealth_urban_from_production, + the two percentage modifiers can roughly approximate total
/// growth of urban wealth. Actual wealth growth is also influenced by country-wide modifiers, e.g.,
/// war exhaustion, so when these variables are in a "non-default" state the approximation will be
/// farther off. I believe rural wealth comes only from wealth_rural_from_pop and
/// wealth_rural_from_production, and a single percentage modifier, usually 1.00 (?).
#[derive(Serialize, Deserialize, Debug)]
pub struct Province {
    name: String,
    rural_pop: i32,
    urban_pop: i32,
    wealth_total_growth: f32,
    wealth_urban_growth: f32,
    owner: String,
}

impl Province {
    fn new() -> Province {
        Province {
            name: String::from(""),
            rural_pop: 0,
            urban_pop: 0,
            wealth_total_growth: 0f32,
            wealth_urban_growth: 0f32,
            owner: String::from("NIL"),
        }
    }
    fn adjust_rural_pop(num_str: &str, prov: &mut Province) -> bool {
        let n: f32 = num_str.parse().expect("Number expected");
        let n = (n * 10000.0) as i32;
        prov.rural_pop += n; //pop is measured in units of 10k.
        true //Keep parsing
    }

    fn adjust_urban_pop(num_str: &str, prov: &mut Province) -> bool {
        let n: f32 = num_str.parse().expect("Number expected");
        let n = (n * 10000.0) as i32;
        prov.urban_pop += n; //pop is measured in units of 10k
        true //Keep parsing
    }

    fn adjust_wealth_growth(num_str: &str, prov: &mut Province) -> bool {
        let n: f32 = num_str.parse().expect("Number expected");
        prov.wealth_total_growth += n;
        true //Keep parsing
    }
}

impl fmt::Display for Province { //TODO better display
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Prov {} has rural pop {} and urban pop {}", self.name, self.rural_pop,
        self.urban_pop)
    }
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let map = parse_save(&config).unwrap();
    let path= Path::new(&config.output_file);
    let mut file: File = match File::create(&path) {
        Err(_) => panic!("Error creating out file"),
        Ok(file) => file
    };
    serde_json::to_writer(&mut file, &map);
//    let mut country_vec: Vec<&Country> = map.values().collect();
//    let urban_pop_sort = |a: &&Country, b: &&Country| -> std::cmp::Ordering {
//        (*b).total_urban_pop.cmp(&(*a).total_urban_pop)
//    };
//    let urbanization = |a: &Country| -> f64 {
//        (a.total_urban_pop as f64) / ((a.total_urban_pop + a.total_rural_pop) as f64)
//    };
//    let urbanization_sort = |a: &&Country, b: &&Country| -> std::cmp::Ordering {
//        urbanization(b).approx_cmp(&urbanization(a), 2)
//    };
//    let wealth_growth_sort = |a: &&Country, b: &&Country| -> std::cmp::Ordering {
//        (*b).total_wealth_growth.approx_cmp(&(*a).total_wealth_growth, 2)
//    };
//    let wealth_growth_per = |a: &Country| -> f32 {
//        a.total_wealth_growth / (a.total_urban_pop + a.total_rural_pop) as f32
//    };
//    let sort_wealth_growth_per = |a: &&Country, b: &&Country| -> std::cmp::Ordering {
//        wealth_growth_per(b).approx_cmp(&wealth_growth_per(a), 2)
//    };
//    country_vec.sort_by(sort_wealth_growth_per);
//    for ref country in country_vec {
//        if country.total_urban_pop + country.total_rural_pop > 100000 {
//            let pop = (country.total_rural_pop + country.total_urban_pop) as f32;
//            println!("{} with wealth per thousand {}", country, 1000f32 * country.total_wealth_growth / pop);
//        }
//    }
    Ok(())
}

fn read_json(string: &str) -> HashMap<String, Country> {
    let path = Path::new(string);
    let mut file: File = File::open(path).expect("This is valid");
    let map: HashMap<String, Country> = serde_json::from_reader(file).expect("Valid");
    map
}

fn parse_save(config: &Config) -> Result<(HashMap<String, Country>), Box<Error>> {
    let var_map = build_function_map();
    let f = File::open(config.filename.clone())?;
    //Using mmap appears to have ~5-10% speedup on file reading / parsing
    let mmap = memmap::Mmap::open(&f, memmap::Protection::Read).unwrap();
    let bytes: &[u8] = unsafe {mmap.as_slice() };
    let f = BufReader::new(bytes);
//    let f = BufReader::new(f);
    let mut countries: HashMap<String, Country> = HashMap::new();
    let mut prov = Province::new();
    let mut variables_tag = false;
    for line in f.lines() {
        let l = line.unwrap_or_else(|_| String::from("   =  ")); //Hacky way to not crash
        if variables_tag {
            variables_tag = variable_match(&l[2..], &mut prov, &var_map);
        } else {
            if l.starts_with("-") { //Beginning of a new province
                if prov.owner != "NIL" {
                    let country = countries.entry(prov.owner.clone())
                        .or_insert(Country::new(prov.owner.clone()));
                    country.add_province(prov);
                }
                prov = Province::new();
                if l.starts_with("-5510") { //Last province
                    break;
                }
            } else if l.starts_with("		variables={") { //The part we care about
                variables_tag = true;
            }
        }
    }
    if prov.owner != "NIL" { //Handling the last province
        let country = countries.entry(prov.owner.clone()).or_insert(Country::new(prov.owner.clone()));
        country.add_province(prov);
    }
    Ok(countries)
}

fn variable_match(input: &str, province: &mut Province,
                  map: &HashMap<&str, fn(&str, &mut Province) -> bool>) -> bool {
    let split: Vec<&str> = input.split("=").collect();
    if let Some(f) = map.get(split[0]) {
        return f(split[1], province) //If it's in the map, it will contain an "="
    }
    return true //Keep parsing variables, in the event of no match
}


fn build_function_map() -> HashMap<&'static str, fn(&str, &mut Province) -> bool> {
    //O(1) matching on Strings
    let mut map: HashMap<&str, fn(&str, &mut Province) -> bool> = HashMap::new();
    map.insert("	rural_population", Province::adjust_rural_pop);
    map.insert("	rural_population_growing", Province::adjust_rural_pop);
    map.insert("	urban_population", Province::adjust_urban_pop);
    map.insert("	urban_population_growing", Province::adjust_urban_pop);
    map.insert("name", |string: &str, prov: &mut Province| {prov.name = string.replace("\"", "");
        true});
    map.insert("institutions", |_, _| false); //Usual quit
    map.insert("history", |_, _| false); //Fail safe quit
    map.insert("owner", |string: &str, prov: &mut Province| {prov.owner = string.replace("\"", "");
        true});
    map.insert("	wealth_total_growth", Province::adjust_wealth_growth);
    map.insert("	wealth_urban_growth", |num_str: &str, prov: &mut Province| {
        let n: f32 = num_str.parse().expect("Number expected"); prov.wealth_urban_growth += n; true
    });
    map
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sort() {
        let mut countries: HashMap<String, Country> = HashMap::new();
        let vec = vec![String::from("FRA"), String::from("SEC"), String::from("THR")];
        let mut start_pop = 20000;
        for tag in vec {
            let mut country = Country::new(tag.clone());
            country.total_urban_pop = start_pop;
            countries.insert(tag.clone(), country);
            start_pop += 20000;
        }
        let mut country_vec: Vec<_> = countries.values().collect();
        let cl = |a: &&Country, b: &&Country| -> std::cmp::Ordering {
            b.total_urban_pop.cmp(&a.total_urban_pop)
        };
        country_vec.sort_by(cl);
        let first = countries.get("FRA");
        let third = countries.get("THR");
        assert_eq!(*country_vec[2].tag, *first.unwrap().tag);
        assert_eq!(*country_vec[0].tag, *third.unwrap().tag);
        assert_eq!(*TAGS.get::<str>(&first.unwrap().tag).unwrap(), "France");
    }

    #[test]
    fn test_tag() {
        let tag1 = TAGS.get("FRA").unwrap();
        assert_eq!(*tag1, "France");
        let tag2 = TAGS.get("MNG").unwrap();
        assert_eq!(*tag2, "Ming");
    }

    #[test]
    fn test_json_read() {
        let map = read_json("example.json");
        let val = map.get("FRA").expect("Valid?");
        assert_eq!(val.tag, "FRA");
    }
}