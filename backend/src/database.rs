// Library to manage the database store
use rusqlite::Connection;
use std::{fmt::Display, path::PathBuf};
use dirs;

#[allow(unused)]
pub fn get_database_location() -> PathBuf {
    dirs::data_local_dir().unwrap().join("zentrox").join("database.db")
}

#[allow(unused)]
pub fn connect_to_db() -> Result<Connection, String> {
    match Connection::open(get_database_location()) {
        Ok(c) => Ok(c),
        Err(e) => {
            dbg!(&e);
            Err(e.to_string())
        },
    }
}

#[allow(unused)]
pub fn setup_database() -> () {
    let conn = connect_to_db().unwrap();
    let _ = conn.execute_batch(include_str!("../setup.sql"));
}

#[allow(unused)]
pub fn read_kv<T: Into<String> + Display>(table: T, key: T) -> Result<String, String> {
    let conn = connect_to_db().expect("Failed to connect to database.");
    let command = conn.prepare(format!("SELECT value FROM \"{table}\" WHERE name=\"{key}\"").as_str());
    
    match command {
        Ok(mut v) => {
            let mut r = v.query([]);
            if r.is_err() { return Err("Failed to read rows from database".to_string()) }
            let mut vec = Vec::new();
            while let Some(row) = r.as_mut().unwrap().next().unwrap() {
                vec.push(row.get_unwrap::<usize, String>(0))
            }
            if vec.len() < 1 {
                return Err("No values found".to_string())
            }
            return Ok(vec[0].clone())
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[allow(unused)]
pub fn read_cols<T: Into<String> + Display>(table: T, cols: &[T]) -> Result<Vec<Vec<String>>, String> {
    let conn = connect_to_db().expect("Failed to connect to database.");
    let command = conn.prepare(format!("SELECT {} FROM \"{table}\"", {
        let mut s = String::new();
        for ele in cols {
            s = format!("{s}\"{ele}\",");
        }
        s.pop();
        s
    }).as_str());

    match command {
        Ok(mut v) => {
            let mut r = v.query([]);
            if r.is_err() { return Err("Failed to read rows from database".to_string()) }
            let mut vec = Vec::new();
            while let Some(row) = r.as_mut().unwrap().next().unwrap() {
                let mut i = 0;
                let mut parsed_row: Vec<String> = Vec::new();
                while i < cols.len() {
                    parsed_row.push(row.get_unwrap::<usize, String>(i));
                    i = i + 1;
                }
                vec.push(parsed_row)
            }
            if vec.len() < 1 {
                return Err("No values found".to_string())
            }
            return Ok(vec)
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[allow(unused)]
pub fn insert<T: Into<String> + Display>(table: T, cols: &[T], values: &[T]) -> Result<usize, String> {
    let conn = connect_to_db().expect("Failed to connect to database.");
    let command = conn.execute(format!("INSERT INTO \"{table}\" ({}) VALUES ({}) ", {
        let mut s = String::new();
        for ele in cols {
            s = format!("{s}\"{ele}\",");
        }
        s.pop();
        s
    }, {
        let mut s = String::new();
        for ele in values {
            s = format!("{s}\"{ele}\",");
        }
        s.pop();
        s
    }).as_str(), ());
    
    match command {
        Ok(v) => {
            Ok(v)
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[allow(unused)]
pub fn update_where<T: Into<String> + Display>(table: T, cols: &[T], values: &[T], where_left: T, where_right: T) -> Result<usize, String> {
    let conn = connect_to_db().expect("Failed to connect to database.");

    if values.len() != cols.len() {
        return Err(format!("Not as many values ({}) as collumns ({}).", values.len(), cols.len()).to_string())
    }

    dbg!(format!("UPDATE \"{table}\" SET {} WHERE \"{where_left}\"=\"{where_right}\"", {
        let mut s = String::new();
        let mut i = 0;
        while i < cols.len() {
            s = format!("{s}\"{}\"=\"{}\",", cols[i], values[i]);
            i+=1
        }
        s.pop();
        s
    }));

    let command = conn.execute(format!("UPDATE \"{table}\" SET {} WHERE \"{where_left}\"=\"{where_right}\"", {
        let mut s = String::new();
        let mut i = 0;
        while i < cols.len() {
            s = format!("{s}\"{}\"=\"{}\",", cols[i], values[i]);
            i+=1
        }
        s.pop();
        s
    }).as_str(), ());
    
    match command {
        Ok(v) => {
            Ok(v)
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[allow(unused)]
pub fn exists<T: Into<String> + Display>(table: T, identifier: T, key: T) -> Result<bool, String> {
    let conn = connect_to_db().expect("Failed to connect to database.");
    let command = conn.prepare(format!("SELECT * FROM {table} WHERE \"{identifier}\"=\"{key}\"").as_str());

    match command {
        Ok(mut v) => {
            let mut r = v.query([]);
            if r.is_err() { return Err("Failed to read rows from database".to_string()) }
            let mut vec = Vec::new();
            while let Some(row) = r.as_mut().unwrap().next().unwrap() {
                vec.push(row.get_unwrap::<usize, String>(0))
            }
            return Ok(vec.len() >= 1)
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

