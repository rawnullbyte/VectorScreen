use slint_build::EmbedResourcesKind;

fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
