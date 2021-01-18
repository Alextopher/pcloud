use std::{fs::File, path::{Path, PathBuf}};

use rocket::{Request, http::{Status, uri::Segments}, request::FromSegments, response::{DEFAULT_CHUNK_SIZE, Responder, Response}};
use rocket_contrib::templates::Template;
use serde::Serialize;
#[derive(Debug)]
pub struct MyError;
pub struct PubFilePathBuf(PathBuf);

impl<'a> FromSegments<'a> for PubFilePathBuf {
    type Error = MyError;

    fn from_segments(segments: Segments<'a>) -> Result<PubFilePathBuf, Self::Error> {
        if let Ok(buf) = segments.into_path_buf(false) {
            let joined = Path::new("storage/public/").join(buf);

            if joined.is_file() {
                Ok(PubFilePathBuf(joined))
            } else {
                Err(MyError)
            }
        } else {
            Err(MyError)
        }
    }
}
pub struct PubDirectoryPathBuf(PathBuf);

impl<'a> FromSegments<'a> for PubDirectoryPathBuf {
    type Error = MyError;

    fn from_segments(segments: Segments<'a>) -> Result<PubDirectoryPathBuf, Self::Error> {
        if let Ok(buf) = segments.into_path_buf(false) {
            let joined = Path::new("storage/public/").join(buf);

            if joined.is_dir() {
                Ok(PubDirectoryPathBuf(joined))
            } else {
                Err(MyError)
            }
        } else {
            Err(MyError)
        }
    }
}

pub struct FileDownload(File);

impl<'r> Responder<'r> for FileDownload {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .chunked_body(self.0, DEFAULT_CHUNK_SIZE)
            .raw_header("Content-Disposition", "attachment")
            .ok()
    }
}

#[get("/public/<file..>", rank = 1)]
pub fn public_file(file: PubFilePathBuf) -> Option<FileDownload> {    
    if let Ok(file) = File::open(file.0) {
        Some(FileDownload(file))
    } else {
        None
    }
}

#[derive(Serialize)]
struct EntryModel {
    name: String,
    size: u64,
    base: String,
}
#[derive(Serialize)]
struct DirectoryModel<'a> {
    uri: &'a str,
    entries: Vec<EntryModel>
}

impl<'a> DirectoryModel<'a> {
    fn new(uri: &'a str, entries: Vec<EntryModel>) -> Self { Self { uri, entries } }
}

#[get("/public")]
pub fn public() -> Option<Template> {
    let dir = PubDirectoryPathBuf(PathBuf::from("storage/public"));
    public_directory(dir)
}

#[get("/public/<dir..>", rank = 2)]
pub fn public_directory(dir: PubDirectoryPathBuf) -> Option<Template> {
    let contents = dir.0.read_dir();
    let uri = dir.0.strip_prefix("storage/").unwrap().to_str().unwrap().to_string();

    if let Ok(contents) = contents {
        let mut entries: Vec<EntryModel> =contents
            .filter_map(Result::ok)
            .map(|entry| {
                let metadata = entry.metadata().unwrap();
                let mut name = entry.file_name().to_str().unwrap().to_string();
                let base = uri.clone();
                if metadata.is_dir() {
                    name.push('/');
                }
                let size = metadata.len();

                EntryModel { name: name, size: size, base: base }
            })
            .collect();
        
        if uri != "public" {
            entries.insert(0, EntryModel { name: "..".to_string(), base: uri.clone(), size: 0 } );
        }

        Some(Template::render("directory", DirectoryModel::new(&uri, entries)))
    } else {
        None
    }
}
