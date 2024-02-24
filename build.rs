use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    tonic_build::configure().build_server(false).compile(
        &[format!("{manifest_dir}/proto/inx.proto")],
        &[format!("{manifest_dir}/proto")],
    )?;
    Ok(())
}