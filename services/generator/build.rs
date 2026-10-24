fn main() {
    let mut config = prost_build::Config::new();
    std::fs::create_dir_all("src/proto").unwrap();
    config.out_dir("src/proto");
    config.compile_protos(&["../../proto/quote.proto"], &["../../proto"]).unwrap();
}
