use rocket::time::OffsetDateTime;

pub fn timestamp_to_date(timestamp:u64) -> String{

    let date =
        OffsetDateTime::from_unix_timestamp(timestamp as i64).unwrap();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year(),date.month() as u32,date.day(),
        date.hour(),date.minute(),date.second()
    )
}