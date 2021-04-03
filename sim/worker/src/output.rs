//! Sending stuff outside (DB)
use anyhow::Context;
use slog::{debug, info, Logger};

pub async fn send_schema<'a>(
    logger: Logger,
    connection: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
    queen_tag: &'a str,
) -> anyhow::Result<()> {
    debug!(logger, "Sending schema");
    let schema = caolo_sim::scripting_api::make_import();
    let imports = schema.imports();

    let basic_descs = cao_lang::compiler::card_description::get_instruction_descriptions();

    #[derive(serde::Serialize)]
    struct Card<'a> {
        name: &'a str,
        description: &'a str,
        ty: &'a str,
        input: &'a [&'a str],
        output: &'a [&'a str],
        constants: &'a [&'a str],
    }

    let msg = imports
        .iter()
        .map(|import| Card {
            name: import.desc.name,
            description: import.desc.description,
            constants: &*import.desc.constants,
            input: &*import.desc.input,
            output: &*import.desc.output,
            ty: import.desc.ty.as_str(),
        })
        .chain(basic_descs.iter().map(|card| Card {
            name: card.name,
            description: card.description,
            input: &*card.input,
            output: &*card.output,
            constants: &*card.constants,
            ty: card.ty.as_str(),
        }))
        .collect::<Vec<_>>();

    let payload = serde_json::to_value(&msg)?;

    sqlx::query!(
        r#"
    INSERT INTO scripting_schema (queen_tag, payload)
    VALUES ($1, $2)
    ON CONFLICT (queen_tag)
    DO UPDATE SET
    payload=$2
        "#,
        queen_tag,
        payload
    )
    .execute(connection)
    .await
    .with_context(|| "Failed to send schema")?;

    debug!(logger, "Sending schema done");
    Ok(())
}

async fn output_to_db<'a>(
    time: i64,
    payload: &'a serde_json::Value,
    connection: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
    queen_tag: &'a str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO world_output (queen_tag, world_time, payload)
        VALUES ($1, $2, $3);
        "#,
        queen_tag,
        time,
        payload
    )
    .execute(connection)
    .await
    .with_context(|| "Failed to insert current world state into DB")?;
    Ok(())
}

pub async fn send_world<'a>(
    logger: Logger,
    time: i64,
    payload: &'a serde_json::Value,
    queen_tag: &'a str,
    db: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
) -> anyhow::Result<()> {
    info!(logger, "Sending world");

    output_to_db(time, payload, db, queen_tag)
        .await
        .with_context(|| "Failed to send to db")?;

    info!(logger, "Sending world done");
    Ok(())
}
