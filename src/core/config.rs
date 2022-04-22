use toml_edit::Document;

struct Config {
    core_path: std::Path::Pathbuf,
    document: Document,
}