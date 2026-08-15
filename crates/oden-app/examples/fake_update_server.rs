//! A local stand-in for the GitHub Releases API, used to exercise Oden's
//! in-app updater (check -> download -> "install" -> restart, plus the
//! post-update "what's new" changelog popup) without touching the real
//! release channel or the real running binary.
//!
//! Run it, then point a debug build of Oden at it:
//!
//! ```text
//! cargo run --example fake_update_server
//! # in another shell:
//! ODEN_FAKE_UPDATE_SERVER=http://127.0.0.1:4067 cargo run -p oden
//! ```
//!
//! It always reports a single fake release, `v99.99.99`, newer than any
//! real Oden version, with a small placeholder "binary" as its asset. The
//! updater still performs a real HTTP download and a real self-replace of
//! *something* ; but oden-app redirects that replace at a scratch file
//! under the OS temp dir whenever this fake server is in play, so nothing
//! about the actual running app is ever touched.

use std::io::{Cursor, Read};
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use zip::write::SimpleFileOptions;

const FAKE_VERSION: &str = "99.99.99";
const THROTTLE_TOTAL: Duration = Duration::from_secs(6);
const FAKE_ASSET_SIZE: usize = 4 * 1024 * 1024;
const FAKE_CHANGELOG: &str = "\
## Fake test release

This changelog came from the **local fake update server**, not GitHub. \
Nothing was actually downloaded from, or installed from, a real release.

- Exercises the check-for-updates flow end to end.
- Exercises the install flow: a real HTTP download happens, but it's \
  written to a scratch file, never the real Oden binary.
- Exercises this \"what's new\" popup, rendered with Oden's own markdown \
  renderer.

Unset `ODEN_FAKE_UPDATE_SERVER` to go back to talking to real GitHub \
releases.

The following is volontarily long Lorem Ipsum filler text to make the changelog scrollable and exercise the scrollable popup UI.

Maecenas dui. Aliquam volutpat auctor lorem.  Cras placerat est vitae lectus. Curabitur massa lectus, rutrum  euismod, dignissim ut, dapibus a, odio. Ut eros erat, vulputate ut,  interdum non, porta eu, erat. Cras fermentum, felis in porta congue,  velit leo facilisis odio, vitae consectetuer lorem quam vitae orci.  Sed ultrices, pede eu placerat auctor, ante ligula rutrum tellus,  vel posuere nibh lacus nec nibh. Maecenas laoreet dolor at enim.  Donec molestie dolor nec metus. Vestibulum libero. Sed quis erat.  Sed tristique. Duis pede leo, fermentum quis, consectetuer eget,  vulputate sit amet, erat.

Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos hymenaeos. Aenean nonummy turpis id odio. Integer euismod imperdiet turpis. Ut nec leo nec diam imperdiet lacinia. Etiam eget lacus eget mi ultricies posuere. In placerat tristique tortor. Sed porta vestibulum metus. Nulla iaculis sollicitudin pede. Fusce luctus tellus in dolor. Curabitur auctor velit a sem. Morbi sapien. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos hymenaeos. Donec adipiscing urna vehicula nunc. Sed ornare leo in leo. In rhoncus leo ut dui. Aenean dolor quam, volutpat nec, fringilla id, consectetuer vel, pede.

Vivamus sit amet pede. Duis interdum, nunc  eget rutrum dignissim, nisl diam luctus leo, et tincidunt velit nisl  id tellus. In lorem tellus, aliquet vitae, porta in, aliquet sed,  lectus. Phasellus sodales. Ut varius scelerisque erat. In vel nibh  eu eros imperdiet rutrum. Donec ac odio nec neque vulputate  suscipit. Nam nec magna. Pellentesque habitant morbi tristique  senectus et netus et malesuada fames ac turpis egestas. Nullam  porta, odio et sagittis iaculis, wisi neque fringilla sapien, vel  commodo lorem lorem id elit. Ut sem lectus, scelerisque eget,  placerat et, tincidunt scelerisque, ligula. Pellentesque non  orci.

Donec libero. Quisque vitae est quis dui  bibendum suscipit. Fusce leo felis, sagittis non, vehicula ac,  ultricies vitae, diam. Aenean congue libero et metus. Nulla  convallis libero a lacus. Donec hendrerit lorem sit amet leo. Mauris  libero. Pellentesque pulvinar molestie dolor. Proin nibh mauris,  ornare at, pretium sit amet, porttitor vel, mi. Pellentesque  habitant morbi tristique senectus et netus et malesuada fames ac  turpis egestas.

Morbi congue congue metus. Aenean sed purus.  Nam pede magna, tristique nec, porta id, sollicitudin quis, sapien.  Vestibulum blandit. Suspendisse ut augue ac nibh ullamcorper  posuere. Integer euismod, neque at eleifend fringilla, augue elit  ornare dolor, vel tincidunt purus est id lacus. Vivamus lorem dui,  commodo quis, scelerisque eu, tincidunt non, magna. Cras sodales.  Quisque vestibulum pulvinar diam. Phasellus tincidunt, leo vitae  tristique facilisis, ipsum wisi interdum sem, dapibus semper nulla  velit vel lectus. Cras dapibus mauris et augue. Quisque cursus nulla  in libero. Suspendisse et lorem sit amet mauris malesuada mollis.  Nullam id justo. Maecenas venenatis. Donec lacus arcu, egestas ac,  fermentum consectetuer, tempus eu, metus. Proin sodales, sem in  pretium fermentum, arcu sapien commodo mauris, venenatis consequat  augue urna in wisi. Quisque sapien nunc, varius eget, condimentum  quis, lacinia in, est. Fusce facilisis. Praesent nec ipsum.

Curabitur tellus magna, porttitor a, commodo a, commodo in, tortor. Donec interdum. Praesent scelerisque. Maecenas posuere sodales odio. Vivamus metus lacus, varius quis,  imperdiet quis, rhoncus a, turpis. Etiam ligula arcu, elementum a, venenatis quis, sollicitudin sed, metus. Donec nunc pede, tincidunt in, venenatis vitae, faucibus vel, nibh. Pellentesque wisi. Nullam malesuada. Morbi ut tellus ut pede tincidunt porta. Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Etiam congue neque id dolor.

Vestibulum ante ipsum primis in faucibus orci  luctus et ultrices posuere cubilia Curae Aliquam interdum porttitor  tortor. Donec ultricies justo eget sapien. Proin ac est. Aliquam  erat volutpat. In tempus scelerisque ligula. Morbi scelerisque urna.  Duis ac nisl. Donec sed leo. Fusce posuere orci mollis nunc. Sed  arcu enim, pharetra nec, aliquam eu, consectetuer sit amet, eros.  Sed id enim. Etiam mattis est at elit. Pellentesque est risus,  pellentesque nec, dignissim vitae, egestas vitae, sapien. Maecenas  et eros non libero iaculis facilisis. Mauris porttitor tempor justo.  Sed sollicitudin neque nec libero.

Nam dui ligula, fringilla a, euismod sodales, sollicitudin vel, wisi. Morbi auctor lorem non justo. Nam lacus libero, pretium at, lobortis vitae, ultricies et, tellus. Donec aliquet, tortor sed accumsan bibendum, erat ligula aliquet magna, vitae ornare odio metus a mi. Morbi ac orci et nisl hendrerit mollis. Suspendisse ut massa. Cras nec ante. Pellentesque a nulla. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Aliquam tincidunt urna. Nulla ullamcorper vestibulum turpis. Pellentesque cursus luctus mauris.

Aenean tincidunt laoreet dui. Vestibulum ante  ipsum primis in faucibus orci luctus et ultrices posuere cubilia  Curae Integer ipsum lectus, fermentum ac, malesuada in, eleifend  ut, lorem. Vivamus ipsum turpis, elementum vel, hendrerit ut, semper  at, metus. Vivamus sapien tortor, eleifend id, dapibus in, egestas  et, pede. Pellentesque faucibus. Praesent lorem neque, dignissim in,  facilisis nec, hendrerit vel, odio. Nam at diam ac neque aliquet  viverra. Morbi dapibus ligula sagittis magna. In lobortis. Donec  aliquet ultricies libero. Nunc dictum vulputate purus. Morbi varius.  Lorem ipsum dolor sit amet, consectetuer adipiscing elit. In tempor.  Phasellus commodo porttitor magna. Curabitur vehicula odio vel  dolor.

Sed commodo posuere pede. Mauris ut est. Ut quis purus. Sed ac odio. Sed vehicula hendrerit sem. Duis non odio. Morbi ut dui. Sed accumsan risus eget odio. In hac habitasse platea dictumst. Pellentesque non elit. Fusce sed justo eu urna porta tincidunt. Mauris felis odio, sollicitudin sed, volutpat a, ornare ac, erat. Morbi quis dolor. Donec pellentesque, erat ac sagittis semper, nunc dui lobortis purus, quis congue purus metus ultricies tellus. Proin et quam. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos hymenaeos. Praesent sapien turpis, fermentum vel, eleifend faucibus, vehicula eu, lacus.
";

struct ThrottledReader {
    cursor: Cursor<Vec<u8>>,
    bytes_per_sec: f64,
}

impl ThrottledReader {
    fn new(data: Vec<u8>) -> Self {
        let bytes_per_sec = data.len() as f64 / THROTTLE_TOTAL.as_secs_f64();
        Self {
            cursor: Cursor::new(data),
            bytes_per_sec,
        }
    }
}

impl Read for ThrottledReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.cursor.read(buf)?;
        if n > 0 {
            std::thread::sleep(Duration::from_secs_f64(n as f64 / self.bytes_per_sec));
        }
        Ok(n)
    }
}

fn platform_asset_hint() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "oden-win-x64";
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return "oden-win-arm64";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "oden-macos-arm64";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "oden-linux-x64";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "oden-linux-arm64";
}

fn placeholder_asset_bytes(target: &str) -> Vec<u8> {
    let bin_name = if target.contains("windows") {
        "oden.exe"
    } else {
        "oden"
    };

    let mut content = Vec::with_capacity(FAKE_ASSET_SIZE);
    content.extend_from_slice(
        b"this is a fake test build produced by fake_update_server, not a real oden binary\n",
    );
    while content.len() < FAKE_ASSET_SIZE {
        content.extend_from_slice(b"padding so this download has enough bytes to show real incremental progress instead of arriving in one burst\n");
    }
    content.truncate(FAKE_ASSET_SIZE);

    let mut buf = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(Cursor::new(&mut buf));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        writer.start_file(bin_name, options).expect("start_file");
        std::io::Write::write_all(&mut writer, &content).expect("write placeholder asset");
        writer.finish().expect("finish zip");
    }
    buf
}

fn release_json(port: u16, asset_name: &str) -> serde_json::Value {
    serde_json::json!({
        "tag_name": format!("v{FAKE_VERSION}"),
        "name": "Fake Update (test server)",
        "created_at": Utc::now().to_rfc3339(),
        "body": FAKE_CHANGELOG,
        "assets": [
            {
                "name": asset_name,
                "url": format!("http://127.0.0.1:{port}/download/{asset_name}"),
            }
        ],
    })
}

fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4067);
    let target = self_update::get_target();
    let asset_name = format!("{}_{FAKE_VERSION}.zip", platform_asset_hint());
    let asset_bytes = placeholder_asset_bytes(target);

    let server =
        Server::http((Ipv4Addr::LOCALHOST, port)).expect("failed to bind fake update server");

    println!("fake update server listening on http://127.0.0.1:{port}");
    println!("target: {target}");
    println!("fake release: v{FAKE_VERSION} (asset: {asset_name})");
    println!();
    println!("run Oden against it with:");
    println!("  ODEN_FAKE_UPDATE_SERVER=http://127.0.0.1:{port} cargo run -p oden");
    println!();
    println!("the update flow performs a real download from this server, but the");
    println!("\"install\" step is redirected to a scratch file, never the real oden binary.");

    let asset_name = Arc::new(asset_name);
    let asset_bytes = Arc::new(asset_bytes);

    for request in server.incoming_requests() {
        let asset_name = Arc::clone(&asset_name);
        let asset_bytes = Arc::clone(&asset_bytes);
        std::thread::spawn(move || handle_request(request, port, &asset_name, &asset_bytes));
    }
}

fn handle_request(request: Request, port: u16, asset_name: &str, asset_bytes: &[u8]) {
    let method = request.method().clone();
    let path = request.url().split('?').next().unwrap_or("").to_string();
    println!("{method:?} {path}");

    if method != Method::Get {
        let _ = request.respond(Response::empty(405));
        return;
    }

    if path.ends_with("/releases/latest") && path.starts_with("/repos/") {
        let body = release_json(port, asset_name).to_string();
        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
        let _ = request.respond(Response::from_string(body).with_header(header));
        return;
    }

    if path.ends_with("/releases") && path.starts_with("/repos/") {
        let body = serde_json::Value::Array(vec![release_json(port, asset_name)]).to_string();
        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
        let _ = request.respond(Response::from_string(body).with_header(header));
        return;
    }

    if path == format!("/download/{asset_name}") {
        println!(
            "  serving download over ~{}s so there's something to watch...",
            THROTTLE_TOTAL.as_secs()
        );
        let header =
            Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..]).unwrap();
        let len = asset_bytes.len();
        let reader = ThrottledReader::new(asset_bytes.to_vec());
        let response = Response::new(StatusCode(200), vec![header], reader, Some(len), None)
            .with_chunked_threshold(usize::MAX);
        let _ = request.respond(response);
        return;
    }

    let _ = request.respond(Response::from_string("not found").with_status_code(404));
}
