use std::{fs::File, path::{Path, PathBuf}};

use rocket::{Request, http::{Status, uri::Segments}, request::FromSegments, response::{DEFAULT_CHUNK_SIZE, Responder, Response}};
use rocket_contrib::templates::Template;

use crate::{models::{DirectoryModel, EntryModel}, users::LoginedUser};
#[derive(Debug)]
pub struct MyError;
pub struct PrivateFile(PathBuf);

impl<'a> FromSegments<'a> for PrivateFile {
    type Error = MyError;

    fn from_segments(segments: Segments<'a>) -> Result<PrivateFile, Self::Error> {
        if let Ok(buf) = segments.into_path_buf(false) {
            let joined = Path::new("storage/private/").join(buf);

            if joined.is_file() {
                Ok(PrivateFile(joined))
            } else {
                Err(MyError)
            }
        } else {
            Err(MyError)
        }
    }
}
pub struct PrivateDirectory(PathBuf);

impl<'a> FromSegments<'a> for PrivateDirectory {
    type Error = MyError;

    fn from_segments(segments: Segments<'a>) -> Result<PrivateDirectory, Self::Error> {
        if let Ok(buf) = segments.into_path_buf(false) {
            let joined = Path::new("storage/private/").join(buf);

            if joined.is_dir() {
                Ok(PrivateDirectory(joined))
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

#[get("/private")]
pub fn private(user: &LoginedUser) -> Option<Template> {
    let dir = PrivateDirectory(PathBuf::from("storage/private"));
    private_directory(user, dir)
}

#[get("/private/<file..>", rank = 1)]
pub fn private_file(_user: &LoginedUser, file: PrivateFile) -> Option<FileDownload> {    
    if let Ok(file) = File::open(file.0) {
        Some(FileDownload(file))
    } else {
        None
    }
}

#[get("/private/<dir..>", rank = 2)]
pub fn private_directory(_user: &LoginedUser, dir: PrivateDirectory) -> Option<Template> {
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
        
        if uri != "private" {
            entries.insert(0, EntryModel { name: "..".to_string(), base: uri.clone(), size: 0 } );
        }

        Some(Template::render("directory", DirectoryModel::new(&uri, entries)))
    } else {
        None
    }
}
