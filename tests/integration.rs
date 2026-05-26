/// Helper to run the CLI and capture output
fn cli_command() -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("pdf-slim").unwrap();
    cmd.env("RUST_LOG", "off");
    cmd
}

/// Create a simple test PDF at the given path.
/// Includes uncompressed streams so compression actually reduces size.
fn create_test_pdf(path: &std::path::Path) {
    use lopdf::{Document, Object, Stream, Dictionary};

    let mut doc = Document::new();
    doc.version = "1.7".to_string();

    // Create root catalog
    let mut root = Dictionary::new();
    root.set(b"Type", Object::Name(b"Catalog".to_vec()));
    let root_id = doc.add_object(root);
    doc.trailer.set(b"Root", Object::Reference(root_id));

    // Create a page
    let mut page_dict = Dictionary::new();
    page_dict.set(b"Type", Object::Name(b"Page".to_vec()));
    page_dict.set(b"MediaBox", Object::Array(vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Integer(612),
        Object::Integer(792),
    ]));

    // Add content stream (uncompressed, repetitive data to ensure compression helps)
    let content_dict = Dictionary::new();
    let padding = " ".repeat(4096);
    let content_data = format!("BT /F1 12 Tf 100 700 Td (Hello World) Tj ET{}", padding);
    let content_stream = Stream::new(content_dict, content_data.into_bytes());
    let content_id = doc.add_object(content_stream);
    page_dict.set(b"Contents", Object::Reference(content_id));

    let page_id = doc.add_object(page_dict);

    // Pages tree
    let mut pages_dict = Dictionary::new();
    pages_dict.set(b"Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set(b"Count", Object::Integer(1));
    pages_dict.set(b"Kids", Object::Array(vec![Object::Reference(page_id)]));
    let pages_id = doc.add_object(pages_dict);

    // Set root pages reference
    let root_obj = doc.get_object_mut(root_id).unwrap();
    let root_dict = root_obj.as_dict_mut().unwrap();
    root_dict.set(b"Pages", Object::Reference(pages_id));

    std::fs::create_dir_all(path.parent().unwrap()).ok();
    doc.save(path).unwrap();
}

/// Create a test PDF with an embedded image
fn create_test_pdf_with_image(path: &std::path::Path) {
    use lopdf::{Document, Object, Stream, Dictionary};

    let mut doc = Document::new();
    doc.version = "1.7".to_string();

    // Create root catalog
    let mut root = Dictionary::new();
    root.set(b"Type", Object::Name(b"Catalog".to_vec()));
    let root_id = doc.add_object(root);
    doc.trailer.set(b"Root", Object::Reference(root_id));

    // Create a fake image (2x2 RGB pixels = 12 bytes)
    let image_data: Vec<u8> = vec![
        255, 0, 0, 0, 255, 0,
        0, 0, 255, 255, 255, 255,
    ];
    let mut img_dict = Dictionary::new();
    img_dict.set(b"Type", Object::Name(b"XObject".to_vec()));
    img_dict.set(b"Subtype", Object::Name(b"Image".to_vec()));
    img_dict.set(b"Width", Object::Integer(2));
    img_dict.set(b"Height", Object::Integer(2));
    img_dict.set(b"ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    img_dict.set(b"BitsPerComponent", Object::Integer(8));
    img_dict.set(b"Length", Object::Integer(image_data.len() as i64));
    let image_stream = Stream::new(img_dict, image_data);
    let image_id = doc.add_object(image_stream);

    // Create a page with the image
    let mut page_dict = Dictionary::new();
    page_dict.set(b"Type", Object::Name(b"Page".to_vec()));
    page_dict.set(b"MediaBox", Object::Array(vec![
        Object::Integer(0), Object::Integer(0),
        Object::Integer(612), Object::Integer(792),
    ]));

    // Resources with XObject
    let mut resources = Dictionary::new();
    let mut xobjects = Dictionary::new();
    xobjects.set(b"img1", Object::Reference(image_id));
    resources.set(b"XObject", Object::Dictionary(xobjects));
    let resources_id = doc.add_object(resources);
    page_dict.set(b"Resources", Object::Reference(resources_id));

    // Content stream
    let mut content_dict = Dictionary::new();
    content_dict.set(b"Length", Object::Integer(30));
    let content_stream = Stream::new(content_dict, b"q /img1 Do Q".to_vec());
    let content_id = doc.add_object(content_stream);
    page_dict.set(b"Contents", Object::Reference(content_id));

    let page_id = doc.add_object(page_dict);

    // Pages tree
    let mut pages_dict = Dictionary::new();
    pages_dict.set(b"Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set(b"Count", Object::Integer(1));
    pages_dict.set(b"Kids", Object::Array(vec![Object::Reference(page_id)]));
    let pages_id = doc.add_object(pages_dict);

    let root_obj = doc.get_object_mut(root_id).unwrap();
    let root_dict = root_obj.as_dict_mut().unwrap();
    root_dict.set(b"Pages", Object::Reference(pages_id));

    std::fs::create_dir_all(path.parent().unwrap()).ok();
    doc.save(path).unwrap();
}

#[test]
fn test_help_flag() {
    let mut cmd = cli_command();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn test_version_flag() {
    let mut cmd = cli_command();
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn test_no_args_shows_help() {
    let mut cmd = cli_command();
    cmd.assert().failure();
}

#[test]
fn test_info_mode() {
    let dir = std::env::temp_dir().join("pdf-slim-test-info");
    let pdf_path = dir.join("test.pdf");
    create_test_pdf(&pdf_path);

    let mut cmd = cli_command();
    cmd.arg("--info").arg(&pdf_path);
    cmd.assert().success();

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_compression_creates_slim_file() {
    let dir = std::env::temp_dir().join("pdf-slim-test-compress");
    let pdf_path = dir.join("test.pdf");
    let slim_path = dir.join("test.slim.pdf");
    create_test_pdf(&pdf_path);

    let mut cmd = cli_command();
    cmd.arg(&pdf_path);
    cmd.assert().success();

    assert!(slim_path.exists(), "slim.pdf should be created");

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_file(&slim_path).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_json_output() {
    let dir = std::env::temp_dir().join("pdf-slim-test-json");
    let pdf_path = dir.join("test.pdf");
    create_test_pdf(&pdf_path);

    let mut cmd = cli_command();
    cmd.arg("--json").arg(&pdf_path);
    let output = cmd.assert().success().get_output().clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert!(report.get("version").is_some());
    assert!(report.get("timestamp").is_some());
    assert!(report.get("summary").is_some());
    assert!(report.get("files").is_some());

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_json_file_output() {
    let dir = std::env::temp_dir().join("pdf-slim-test-json-file");
    let pdf_path = dir.join("test.pdf");
    let json_path = dir.join("report.json");
    create_test_pdf(&pdf_path);

    let mut cmd = cli_command();
    cmd.arg("--json-file")
        .arg(&json_path)
        .arg(&pdf_path);
    cmd.assert().success();

    assert!(json_path.exists(), "JSON report file should be created");

    let content = std::fs::read_to_string(&json_path).unwrap();
    let report: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(report.get("summary").is_some());

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_file(&json_path).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_preset_options() {
    let dir = std::env::temp_dir().join("pdf-slim-test-preset");
    let pdf_path = dir.join("test.pdf");
    create_test_pdf(&pdf_path);

    for preset in &["gentle", "balanced", "aggressive"] {
        let mut cmd = cli_command();
        cmd.arg("--preset").arg(preset).arg(&pdf_path);
        cmd.assert().success();
        std::fs::remove_file(dir.join("test.slim.pdf")).ok();
    }

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_force_creates_backup() {
    let dir = std::env::temp_dir().join("pdf-slim-test-force");
    let pdf_path = dir.join("test.pdf");
    let bak_path = dir.join("test.pdf-slim-bak");
    create_test_pdf(&pdf_path);

    let mut cmd = cli_command();
    cmd.arg("--force").arg(&pdf_path);
    cmd.assert().success();

    assert!(bak_path.exists(), "backup file should be created");

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_file(&bak_path).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_image_compression() {
    let dir = std::env::temp_dir().join("pdf-slim-test-img");
    let pdf_path = dir.join("test.pdf");
    create_test_pdf_with_image(&pdf_path);

    let mut cmd = cli_command();
    cmd.arg("--json").arg(&pdf_path);
    let output = cmd.assert().success().get_output().clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _report: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_file(dir.join("test.slim.pdf")).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_batch_processing() {
    let dir = std::env::temp_dir().join("pdf-slim-test-batch");
    let pdf1 = dir.join("test1.pdf");
    let pdf2 = dir.join("test2.pdf");
    create_test_pdf(&pdf1);
    create_test_pdf(&pdf2);

    let mut cmd = cli_command();
    cmd.arg("--json")
        .arg(&pdf1)
        .arg(&pdf2);
    let output = cmd.assert().success().get_output().clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let files = report["files"].as_array().unwrap();
    assert_eq!(files.len(), 2, "should have 2 file entries");

    std::fs::remove_file(&pdf1).ok();
    std::fs::remove_file(&pdf2).ok();
    std::fs::remove_file(dir.join("test1.slim.pdf")).ok();
    std::fs::remove_file(dir.join("test2.slim.pdf")).ok();
    std::fs::remove_dir(&dir).ok();
}
