use regex::Regex;
use rusqlite::{Connection, Result as SqliteResult};

pub fn validate_identifier(name: &str) -> Result<(), String> {
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    if !re.is_match(name) {
        return Err(format!(
            "Invalid identifier '{}': must match ^[a-zA-Z_][a-zA-Z0-9_]*$",
            name
        ));
    }
    let upper = name.to_uppercase();
    let reserved = ["SQLITE_MASTER", "SQLITE_SEQUENCE", "SQLITE_TEMP_MASTER"];
    if reserved.contains(&upper.as_str()) {
        return Err(format!("'{}' is a reserved name", name));
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TableInfo {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnInfo {
    pub cid: i32,
    pub name: String,
    pub col_type: String,
    pub notnull: bool,
    pub default_value: Option<String>,
    pub pk: bool,
}

pub fn list_tables(conn: &Connection) -> SqliteResult<Vec<TableInfo>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    )?;
    let tables = stmt
        .query_map([], |row| Ok(TableInfo { name: row.get(0)? }))?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(tables)
}

pub fn list_columns(conn: &Connection, table_name: &str) -> Result<Vec<ColumnInfo>, String> {
    validate_identifier(table_name)?;
    let query = format!("PRAGMA table_info(\"{}\")", table_name);
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let columns = stmt
        .query_map([], |row| {
            Ok(ColumnInfo {
                cid: row.get(0)?,
                name: row.get(1)?,
                col_type: row.get(2)?,
                notnull: row.get::<_, i32>(3)? != 0,
                default_value: row.get(4)?,
                pk: row.get::<_, i32>(5)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(columns)
}

pub fn create_table(conn: &Connection, table_name: &str) -> Result<(), String> {
    validate_identifier(table_name)?;
    let sql = format!(
        "CREATE TABLE \"{}\" (id INTEGER PRIMARY KEY AUTOINCREMENT)",
        table_name
    );
    conn.execute(&sql, []).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn drop_table(conn: &Connection, table_name: &str) -> Result<(), String> {
    validate_identifier(table_name)?;
    let sql = format!("DROP TABLE IF EXISTS \"{}\"", table_name);
    conn.execute(&sql, []).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    column_type: &str,
) -> Result<(), String> {
    validate_identifier(table_name)?;
    validate_identifier(column_name)?;
    let allowed_types = ["TEXT", "INTEGER", "REAL", "BLOB", "NUMERIC"];
    let col_type_upper = column_type.to_uppercase();
    if !allowed_types.contains(&col_type_upper.as_str()) {
        return Err(format!(
            "Invalid column type '{}': must be one of {:?}",
            column_type, allowed_types
        ));
    }
    let sql = format!(
        "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {}",
        table_name, column_name, col_type_upper
    );
    conn.execute(&sql, []).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn drop_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<(), String> {
    validate_identifier(table_name)?;
    validate_identifier(column_name)?;
    let sql = format!(
        "ALTER TABLE \"{}\" DROP COLUMN \"{}\"",
        table_name, column_name
    );
    conn.execute(&sql, []).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        conn
    }

    #[test]
    fn test_validate_identifier_valid() {
        assert!(validate_identifier("users").is_ok());
        assert!(validate_identifier("_private").is_ok());
        assert!(validate_identifier("Table123").is_ok());
    }

    #[test]
    fn test_validate_identifier_invalid() {
        assert!(validate_identifier("123abc").is_err());
        assert!(validate_identifier("user-name").is_err());
        assert!(validate_identifier("DROP TABLE;").is_err());
        assert!(validate_identifier("").is_err());
        assert!(validate_identifier("a b").is_err());
    }

    #[test]
    fn test_create_and_list_tables() {
        let conn = setup_db();
        create_table(&conn, "users").unwrap();
        create_table(&conn, "posts").unwrap();
        let tables = list_tables(&conn).unwrap();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "posts");
        assert_eq!(tables[1].name, "users");
    }

    #[test]
    fn test_drop_table() {
        let conn = setup_db();
        create_table(&conn, "temp_table").unwrap();
        assert_eq!(list_tables(&conn).unwrap().len(), 1);
        drop_table(&conn, "temp_table").unwrap();
        assert_eq!(list_tables(&conn).unwrap().len(), 0);
    }

    #[test]
    fn test_add_and_list_columns() {
        let conn = setup_db();
        create_table(&conn, "users").unwrap();
        add_column(&conn, "users", "name", "TEXT").unwrap();
        add_column(&conn, "users", "age", "INTEGER").unwrap();
        let cols = list_columns(&conn, "users").unwrap();
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].name, "id");
        assert_eq!(cols[1].name, "name");
        assert_eq!(cols[1].col_type, "TEXT");
        assert_eq!(cols[2].name, "age");
    }

    #[test]
    fn test_drop_column() {
        let conn = setup_db();
        create_table(&conn, "users").unwrap();
        add_column(&conn, "users", "name", "TEXT").unwrap();
        add_column(&conn, "users", "temp_col", "TEXT").unwrap();
        drop_column(&conn, "users", "temp_col").unwrap();
        let cols = list_columns(&conn, "users").unwrap();
        assert_eq!(cols.len(), 2);
    }

    #[test]
    fn test_add_column_invalid_type() {
        let conn = setup_db();
        create_table(&conn, "users").unwrap();
        let result = add_column(&conn, "users", "name", "VARCHAR(255)");
        assert!(result.is_err());
    }

    #[test]
    fn test_sql_injection_prevented() {
        let conn = setup_db();
        let result = create_table(&conn, "users; DROP TABLE users;--");
        assert!(result.is_err());
    }
}
