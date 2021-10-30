#[macro_use]
extern crate rocket;

use chrono::{NaiveDateTime, Utc};
use rocket::response::status;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Mutex;
use uuid::Uuid;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                create_entry,
                list_entries,
                visit,
                list_visits,
                list_entry_visits
            ],
        )
        .manage(Store::new())
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
struct NewEntry {
    code: String,
    url: String,
}

#[post("/entries", data = "<data>")]
fn create_entry(
    data: Json<NewEntry>,
    store: &State<Store>,
) -> Result<Json<Entry>, status::BadRequest<String>> {
    let mut entries = store.entries.lock().unwrap();
    let entry = Entry::new(data.code.clone(), data.url.clone());
    // Err(status::BadRequest(Some(String::from("Invalid entry"))))
    entries.push(entry.clone());
    Ok(Json(entry))
}

#[get("/entries")]
fn list_entries(store: &State<Store>) -> Json<Vec<Entry>> {
    let entries = store.entries.lock().unwrap();
    Json(entries.clone())
}

#[get("/<code>")]
fn visit(
    code: &str,
    store: &State<Store>,
    ip_addr: IpAddr,
) -> Result<Redirect, status::NotFound<String>> {
    let entries = store.entries.lock().unwrap();
    let entry = entries.iter().find(|e| e.code == code);

    if let Some(entry) = entry {
        let visit = Visit::new(entry.id, ip_addr);
        let mut visits = store.visits.lock().unwrap();
        visits.push(visit.clone());
        // TODO: make sure query params are forwarded (and override if they exist on .url)
        Ok(Redirect::to(entry.url.clone()))
    } else {
        Err(status::NotFound(String::from("Not found")))
    }
}

#[get("/visits")]
fn list_visits(store: &State<Store>) -> Json<Vec<Visit>> {
    let visits = store.visits.lock().unwrap();
    Json(visits.clone())
}

#[get("/entries/<entry_id>/visits")]
fn list_entry_visits(
    entry_id: Uuid,
    store: &State<Store>,
) -> Result<Json<Vec<Visit>>, status::NotFound<String>> {
    let entries = store.entries.lock().unwrap();
    let entry = entries.iter().find(|e| e.id == entry_id);
    if entry.is_none() {
        return Err(status::NotFound(String::from("Entry not found by that ID")));
    }

    let visits = store.visits.lock().unwrap();
    let visits = visits
        .to_owned()
        .into_iter()
        .filter(|v| v.entry_id == entry_id)
        .collect();
    Ok(Json(visits))
}

struct Store {
    entries: Mutex<Vec<Entry>>,
    visits: Mutex<Vec<Visit>>,
}

impl Store {
    fn new() -> Self {
        Store {
            entries: Mutex::new(Vec::new()),
            visits: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Serialize, Clone)]
struct Entry {
    id: Uuid,
    code: String,
    url: String,
}

impl Entry {
    fn new(code: String, url: String) -> Self {
        Entry {
            id: Uuid::new_v4(),
            code,
            url,
        }
    }
}

#[derive(Serialize, Clone)]
struct Visit {
    id: Uuid,
    entry_id: Uuid,
    ip: IpAddr,
    timestamp: NaiveDateTime,
}

impl Visit {
    fn new(entry_id: Uuid, ip: IpAddr) -> Self {
        Visit {
            id: Uuid::new_v4(),
            entry_id,
            ip,
            timestamp: Utc::now().naive_utc(),
        }
    }
}
