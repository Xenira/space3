use fs_extra::dir::CopyOptions;

fn main() {
    std::fs::remove_dir_all("assets/generated");
    std::fs::create_dir_all("assets/generated").unwrap();
    fs_extra::copy_items(
        &[
            "../protocol/target/assets/gods",
            "../protocol/target/assets/characters",
        ],
        "assets/generated",
        &CopyOptions::new().overwrite(true),
    )
    .unwrap();
}
