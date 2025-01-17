// Library to manage the database store
use dirs;
use rusqlite::types::FromSql;
use rusqlite::{Connection, Row};
use std::{fmt::Display, path::PathBuf};

pub const ST_BOOL_TRUE: &str = "TRUE";
/// Some columns can only use text. If a boolean has to be
/// stored in one of these columns use and ST_BOOL
pub const ST_BOOL_FALSE: &str = "FALSE";
/// Some columns can only use text. If a boolean has to be
/// stored in one of these columns use and ST_BOOL

/// Enumaration used to categorize values for storage in SQL.
/// Use InsertValue::from to automatically convert most common types into an InsertValue.
/// Supported types are:
/// Int(i32)
/// UsingnedInt(u32)
/// Float(f64)
/// Text(String)
/// Bool(bool)
/// Null()
#[allow(unused)]
#[derive(Debug)]
pub enum InsertValue {
    Int32(i32),
    Int64(i64),
    Int128(i128),
    UnsignedInt32(u32),
    UnsignedInt64(u64),
    UnsignedInt128(u128),
    Float(f64),
    Text(String),
    Bool(bool),
    Null(),
}

impl From<i32> for InsertValue {
    fn from(value: i32) -> Self {
        InsertValue::Int32(value)
    }
}

impl From<i64> for InsertValue {
    fn from(value: i64) -> Self {
        InsertValue::Int64(value)
    }
}

impl From<i128> for InsertValue {
    fn from(value: i128) -> Self {
        InsertValue::Int128(value)
    }
}

impl From<u32> for InsertValue {
    fn from(value: u32) -> Self {
        InsertValue::UnsignedInt32(value)
    }
}

impl From<u64> for InsertValue {
    fn from(value: u64) -> Self {
        InsertValue::UnsignedInt64(value)
    }
}

impl From<u128> for InsertValue {
    fn from(value: u128) -> Self {
        InsertValue::UnsignedInt128(value)
    }
}

impl From<f64> for InsertValue {
    fn from(value: f64) -> Self {
        InsertValue::Float(value)
    }
}

trait ToSqlQuerySegment {
    fn to_sql_query_segment(&self) -> String;
}

impl ToSqlQuerySegment for InsertValue {
    fn to_sql_query_segment(&self) -> String {
        match self {
            InsertValue::Int32(value) => value.to_string(),
            InsertValue::Int64(value) => value.to_string(),
            InsertValue::Int128(value) => value.to_string(),
            InsertValue::UnsignedInt32(value) => value.to_string(),
            InsertValue::UnsignedInt64(value) => value.to_string(),
            InsertValue::UnsignedInt128(value) => value.to_string(),
            InsertValue::Float(value) => value.to_string(),
            InsertValue::Text(value) => format!(
                "'{}'",
                value
                    .chars()
                    .flat_map(|c| match c {
                        '\\' => "\\\\".chars().collect::<Vec<_>>(), // Escape backslash
                        '\'' => "''".chars().collect::<Vec<_>>(),   // Escape single quote
                        '\"' => "\"\"\"\"".chars().collect::<Vec<_>>(), // Escape double quote
                        '\0' => "\\0".chars().collect::<Vec<_>>(),  // Escape NULL character
                        _ => vec![c],
                    })
                    .collect::<String>()
            ), // Escape single quotes
            InsertValue::Bool(value) => value.to_string(),
            InsertValue::Null() => "NULL".to_string(),
        }
    }
}

impl From<String> for InsertValue {
    fn from(value: String) -> Self {
        InsertValue::Text(value)
    }
}

impl From<&String> for InsertValue {
    fn from(value: &String) -> Self {
        InsertValue::Text(value.to_string())
    }
}

impl From<&str> for InsertValue {
    fn from(value: &str) -> Self {
        InsertValue::Text(value.to_string())
    }
}

impl From<bool> for InsertValue {
    fn from(value: bool) -> Self {
        InsertValue::Bool(value)
    }
}

/// Enumaration used to classify different errors.
/// NoRow: No such row exists.
/// ExecutionError: An error during the execution of the SQL statement occured. More details found
/// in the String value.
/// CantConnect: Can not connect to the database.
/// InsufficientData: While inserting not as many columns as values are speciffied. Please specify
/// empty values using InsertValue::Null() or using default values.
#[derive(Debug)]
pub enum SQLError {
    NoRow(String),
    ExecutionError(String),
    CantConnect(String),
    InsufficientData(String),
}

impl ToString for SQLError {
    fn to_string(&self) -> String {
        match self {
            SQLError::NoRow(v) => v.clone(),
            SQLError::InsufficientData(v) => v.clone(),
            SQLError::ExecutionError(v) => v.clone(),
            SQLError::CantConnect(v) => v.clone(),
        }
        .to_string()
    }
}

/// Get the absolute path for the database by joining ~/.local/share with zentrox/database.db.
#[allow(unused)]
pub fn get_database_location() -> PathBuf {
    dirs::data_local_dir()
        .unwrap()
        .join("zentrox")
        .join("database.db")
}

/// Open a connection to the database using the get_database_location to retrieve the database file
/// location. If the connection fails an error CantConnect is returned.
#[allow(unused)]
pub fn connect_to_db() -> Result<Connection, SQLError> {
    match Connection::open(get_database_location()) {
        Ok(c) => Ok(c),
        Err(e) => Err(SQLError::CantConnect(e.to_string())),
    }
}

/// Runs all SQL statements of the file setup.sql.
#[allow(unused)]
pub fn setup_database() -> Result<(), String> {
    let conn = connect_to_db().unwrap();
    let s = conn.execute_batch(include_str!("../setup.sql"));

    match s {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Reads the value column of a row in a key-value table.
/// This is not a general database function, but only works on tables that use the column name
/// "name" for the columns that holds the unique key and "value" for the value of this row.
#[allow(unused)]
pub fn read_kv<T: Into<String> + Display>(table: T, key: T) -> Result<String, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");

    let query = format!("SELECT value FROM '{table}' WHERE name='{key}'");

    let command = conn.prepare(query.as_str());

    match command {
        Ok(mut v) => {
            let mut r = v.query([]);
            if r.is_err() {
                return Err(SQLError::ExecutionError(
                    "Failed to read rows from database".to_string(),
                ));
            }
            let mut vec = Vec::new();
            while let Some(row) = r.as_mut().unwrap().next().unwrap() {
                vec.push(row.get_unwrap::<usize, String>(0))
            }
            if vec.len() < 1 {
                return Err(SQLError::NoRow("No such row was found.".to_string()));
            }
            return Ok(vec[0].clone());
        }
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}

// Automatically decided whether to insert or to update a row in order to add or change the
// existing value. This function is a shorthand for use on key-value tables only.
// In a kv table the K collumn is named "name" and the V collumn in named "value".
#[allow(unused)]
pub fn write_kv<T: Into<String> + Display + ToString>(
    table: T,
    key: T,
    value: InsertValue,
) -> Result<usize, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");
    let execution: Result<usize, rusqlite::Error>;

    if exists(table.to_string(), "name".to_string(), &key).unwrap_or(false) {
        let query = format!("UPDATE '{table}' SET {} WHERE name='{key}'", {
            let mut s = String::new();
            s = format!("value={},", { value.to_sql_query_segment() });
            s.pop();
            s
        });
        execution = conn.execute(&query.as_str(), ());
    } else {
        let query = format!("INSERT INTO '{table}' VALUES ('{key}', {})", {
            value.to_sql_query_segment()
        });
        execution = conn.execute(&query.as_str(), ());
    }
    match execution {
        Ok(v) => Ok(v),
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}

pub trait FromRow {
    fn from_row(row: &rusqlite::Row) -> Self;
}

impl<T1> FromRow for (T1,)
where
    T1: FromSql,
{
    fn from_row(row: &Row) -> Self {
        (row.get(0).unwrap(),)
    }
}

impl<T1, T2> FromRow for (T1, T2)
where
    T1: FromSql,
    T2: FromSql,
{
    fn from_row(row: &Row) -> Self {
        (row.get(0).unwrap(), row.get(1).unwrap())
    }
}

impl<T1, T2, T3> FromRow for (T1, T2, T3)
where
    T1: FromSql,
    T2: FromSql,
    T3: FromSql,
{
    fn from_row(row: &Row) -> Self {
        (
            row.get(0).unwrap(),
            row.get(1).unwrap(),
            row.get(2).unwrap(),
        )
    }
}

impl<T1, T2, T3, T4> FromRow for (T1, T2, T3, T4)
where
    T1: FromSql,
    T2: FromSql,
    T3: FromSql,
    T4: FromSql,
{
    fn from_row(row: &Row) -> Self {
        (
            row.get(0).unwrap(),
            row.get(1).unwrap(),
            row.get(2).unwrap(),
            row.get(3).unwrap(),
        )
    }
}

impl<T1, T2, T3, T4, T5> FromRow for (T1, T2, T3, T4, T5)
where
    T1: FromSql,
    T2: FromSql,
    T3: FromSql,
    T4: FromSql,
    T5: FromSql,
{
    fn from_row(row: &Row) -> Self {
        (
            row.get(0).unwrap(),
            row.get(1).unwrap(),
            row.get(2).unwrap(),
            row.get(3).unwrap(),
            row.get(4).unwrap(),
        )
    }
}

/// Read all the specified columns from the specified table.
#[allow(unused)]
pub fn read_cols<T: Into<String> + Display, R: FromRow>(
    table: T,
    cols: &[T],
) -> Result<Vec<R>, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");
    let query = format!("SELECT {} FROM '{table}'", {
        let mut s = String::new();
        for ele in cols {
            s = format!("{s}{ele},");
        }
        s.pop();
        s
    });
    let command = conn.prepare(query.as_str());

    match command {
        Ok(mut v) => {
            let mut r = v.query([]);
            if r.is_err() {
                return Err(SQLError::ExecutionError(
                    "Failed to read rows from database".to_string(),
                ));
            }
            let mut vec = Vec::new();
            while let Some(row) = r.as_mut().unwrap().next().unwrap() {
                let parsed = R::from_row(row);
                vec.push(parsed)
            }
            return Ok(vec);
        }
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}

/// Insert all the values for all the specified columns.
/// The values have to be InsertValues. There have have to be the same amount of values as columns.
#[allow(unused)]
pub fn insert<T: Into<String> + Display>(
    table: T,
    cols: &[T],
    values: &[InsertValue],
) -> Result<usize, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");
    let query = format!(
        "INSERT INTO '{table}' ({}) VALUES ({}) ",
        {
            let mut s = String::new();
            for ele in cols {
                s = format!("{s}\"{ele}\",");
            }
            s.pop();
            s
        },
        {
            let mut s = String::new();
            for ele in values {
                s = format!("{s}{},", { ele.to_sql_query_segment() });
            }
            s.pop();
            s
        }
    );

    let command = conn.execute(query.as_str(), ());

    match command {
        Ok(v) => Ok(v),
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}

/// Update every specified column to the specified values in the specified table where the
/// condition is met. The condition is where_left = where_right.
#[allow(unused)]
pub fn update_where<T: Into<String> + Display>(
    table: T,
    cols: &[T],
    values: &[InsertValue],
    where_left: T,
    where_right: T,
) -> Result<usize, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");

    if values.len() != cols.len() {
        return Err(SQLError::InsufficientData(
            format!(
                "Not as many values ({}) as collumns ({}).",
                values.len(),
                cols.len()
            )
            .to_string(),
        ));
    }

    let command = conn.execute(
        format!(
            "UPDATE \"{table}\" SET {} WHERE \"{where_left}\"=\"{where_right}\"",
            {
                let mut s = String::new();
                let mut i = 0;
                for ele in values {
                    s = format!("{s}{}={},", cols[i], { ele.to_sql_query_segment() });

                    i += 1;
                }
                s.pop();
                s
            }
        )
        .as_str(),
        (),
    );

    match command {
        Ok(v) => Ok(v),
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}

/// Automatically decides whether to use update or insert a row.
pub fn write<T: Into<String> + Display + Clone>(
    table: T,
    cols: &[T],
    values: &[InsertValue],
    unique_column: T,
    unique_key: T,
) -> Result<usize, SQLError> {
    println!("The functions not dead");
    if exists(table.clone(), unique_column.clone(), unique_key.clone()).unwrap() {
        println!("We got so far");
        let uw = update_where(table, cols, values, unique_column, unique_key);
        println!("Update where also lives");
        uw
    } else {
        println!("Well hope inserting isnt fucked");
        let i = insert(table, cols, values);
        println!("It isnt");
        i
    }
}

/// Checks if a row with a certain key in the table exists.
#[allow(unused)]
pub fn exists<T: Into<String> + Display, K: ToString>(
    table: T,
    unique_column: T,
    key: K,
) -> Result<bool, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");

    let e_command = conn.prepare(
        format!(
            "SELECT * FROM {table} WHERE {unique_column}='{}'",
            key.to_string()
                .chars()
                .flat_map(|c| match c {
                    '\\' => "\\\\".chars().collect::<Vec<_>>(), // Escape backslash
                    '\'' => "''".chars().collect::<Vec<_>>(),   // Escape single quote
                    '\"' => "\"\"\"\"".chars().collect::<Vec<_>>(), // Escape double quote
                    '\0' => "\\0".chars().collect::<Vec<_>>(),  // Escape NULL character
                    _ => vec![c],
                })
                .collect::<String>()
        )
        .as_str(),
    );

    let mut x = e_command.unwrap();

    match x.exists(()) {
        Ok(b) => return Ok(b),
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}

pub fn delete_row<T: Into<String> + Display>(
    table: T,
    unqiue_column: T,
    key: T,
) -> Result<usize, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");

    let statement = format!("DELETE FROM {table} WHERE {unqiue_column}='{key}'");

    let x = conn.execute(statement.as_str(), ());

    match x {
        Ok(u) => Ok(u),
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}

pub fn truncate_table<T: Into<String> + Display>(table: T) -> Result<usize, SQLError> {
    let conn = connect_to_db().expect("Failed to connect to database.");

    let statement = format!("DELETE FROM '{table}'");

    let x = conn.execute(statement.as_str(), ());

    match x {
        Ok(u) => Ok(u),
        Err(e) => Err(SQLError::ExecutionError(e.to_string())),
    }
}
