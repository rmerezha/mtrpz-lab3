use tokio_pg_mapper::{Error, FromTokioPostgresRow};
use tokio_postgres::Row;

pub fn from_rows<T>(rows: Vec<Row>) -> Result<Vec<T>, Error>
where T: FromTokioPostgresRow {
    let mut res: Vec<T> = Vec::with_capacity(rows.len());

    for row in rows {
        res.push(T::from_row(row)?)
    }

    Ok(res)
}
