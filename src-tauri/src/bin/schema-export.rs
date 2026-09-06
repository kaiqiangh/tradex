fn main() {
    let schema = schemars::schema_for!(tradex::protocol::IpcSchema);
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("schema is serializable")
    );
}
