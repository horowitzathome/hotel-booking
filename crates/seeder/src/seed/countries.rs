use anyhow::Result;
use sqlx::PgPool;

const COUNTRIES: &[(&str, &str)] = &[
    ("Germany", "DE"),
    ("Spain", "ES"),
    ("France", "FR"),
    ("Italy", "IT"),
    ("Portugal", "PT"),
    ("Netherlands", "NL"),
    ("Belgium", "BE"),
    ("Luxembourg", "LU"),
    ("Austria", "AT"),
    ("Switzerland", "CH"),
    ("Denmark", "DK"),
    ("Sweden", "SE"),
    ("Norway", "NO"),
    ("Finland", "FI"),
    ("Iceland", "IS"),
    ("Ireland", "IE"),
    ("United Kingdom", "GB"),
    ("Poland", "PL"),
    ("Czechia", "CZ"),
    ("Slovakia", "SK"),
    ("Hungary", "HU"),
    ("Slovenia", "SI"),
    ("Croatia", "HR"),
    ("Greece", "GR"),
    ("Cyprus", "CY"),
    ("Malta", "MT"),
    ("Estonia", "EE"),
    ("Latvia", "LV"),
    ("Lithuania", "LT"),
    ("Bulgaria", "BG"),
    ("Romania", "RO"),
    ("United States", "US"),
    ("Canada", "CA"),
    ("Mexico", "MX"),
    ("Brazil", "BR"),
    ("Argentina", "AR"),
    ("Chile", "CL"),
    ("Australia", "AU"),
    ("New Zealand", "NZ"),
    ("Japan", "JP"),
    ("South Korea", "KR"),
    ("Singapore", "SG"),
    ("Thailand", "TH"),
    ("India", "IN"),
    ("South Africa", "ZA"),
    ("Morocco", "MA"),
    ("Egypt", "EG"),
    ("Turkey", "TR"),
    ("Israel", "IL"),
    ("United Arab Emirates", "AE"),
];

pub async fn seed(pool: &PgPool) -> Result<Vec<i64>> {
    let names: Vec<String> = COUNTRIES.iter().map(|(n, _)| (*n).to_string()).collect();
    let codes: Vec<String> = COUNTRIES.iter().map(|(_, c)| (*c).to_string()).collect();

    let ids: Vec<i64> = sqlx::query_scalar(
        "INSERT INTO countries (name, iso_code)
         SELECT * FROM UNNEST($1::text[], $2::text[])
         RETURNING id",
    )
    .bind(&names)
    .bind(&codes)
    .fetch_all(pool)
    .await?;

    Ok(ids)
}
