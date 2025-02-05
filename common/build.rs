fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::Config::new()
        .type_attribute("ClientEnvelope", "#[derive(bon::Builder)]")
        .type_attribute("ServerEnvelope", "#[derive(bon::Builder)]")
        .type_attribute("ClientMessage", "#[derive(bon::Builder)]")
        .type_attribute("ServerMessage", "#[derive(bon::Builder)]")
        .include_file("_includes.rs")
        .compile_protos(&["proto/Envelope.proto"], &["proto"])?;

    Ok(())
}
