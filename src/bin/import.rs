use diesel::prelude::*;
use enum_dispatch::enum_dispatch;

use track_pc_usage_rs as trbtt;
use trbtt::prelude::*;

fn main() -> anyhow::Result<()> {
    let opt = ImportArgs::from_args();
    let data = opt.import()?;
    println!("inserting...");
    use track_pc_usage_rs::schema::activity;
    let db = track_pc_usage_rs::database::connect()?;

    let updated = diesel::insert_into(activity::table)
        .values(&data)
        .execute(&db)?;

    println!("successfully inserted {}/{} entries", updated, data.len());
    Ok(())
}
