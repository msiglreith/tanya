extern crate cc;

fn main() {
    let mut build = cc::Build::new();
    build.cpp(true);
    build
        .define("VMA_STATIC_VULKAN_FUNCTIONS", "0")
        .file("vma/src/VmaUsage.cpp")
        .include("vulkan/include")
        .compile("vma");
}
