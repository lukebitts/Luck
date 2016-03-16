//! TODO: Fill the documentation

pub mod loader_obj;
mod resources;
pub mod loader;

use self::loader_obj::ObjResourceLoader;
pub use self::resources::*;
pub use self::loader::*;

use std::collections::HashMap;
use std::rc::Rc;
use std::{path, mem};

/// A type that centralizes loading and parsing external files into resources which can be used by
/// the engine.
pub struct Resources {
    queue: Vec<(path::PathBuf, String)>,
    data: HashMap<String, Rc<Resource>>,
    loaders: Vec<Box<ResourceLoader>>,
}

/// When queueing a file to be loaded, several checks are made to avoid panicking later.
#[derive(Debug)]
pub enum ResourceQueueError {
    /// Path should only have utf-8 characters.
    PathInvalidCharacters,
    /// Name should only have utf-8 characters. This would only be a problem when a name is not
    /// provided and the path is used, since the path can have non-utf-8 characters.
    NameInvalidCharacters,
    /// Every file should have an extension, otherwise we can't pick the correct parser.
    PathWithoutExtension,
    /// The path should be a file, not a folder.
    PathIsNotFile,
}

fn load_binary_file(_: &path::Path) -> Result<Box<&[u8]>, ResourceLoadError> {
    Ok(Box::new(b"")) // TODO
}

fn load_text_file(path: &path::Path) -> Result<String, ResourceLoadError> {
    use std::fs::File;
    use std::io::Read;

    let mut data = String::new();
    let mut f = try!(File::open(path));
    try!(f.read_to_string(&mut data));

    Ok(data)
}

impl Resources {
    /// Returns a new instance of `Resources` with every loader already inside it.
    pub fn new() -> Self {
        let mut ret = Resources {
            queue: Vec::new(),
            data: HashMap::new(),
            loaders: Vec::new(),
        };
        ret.loaders.push(Box::new(ObjResourceLoader));

        ret
    }

    /// Queues a file to be loaded when `load_all` is called. While the function receives a
    /// path only, the file name internally will be the path converted to string. If you queue
    /// the same file twice, the file will be reloaded.
    pub fn queue(&mut self, path: &path::Path) -> Result<(), ResourceQueueError> {
        if let Some(path_name) = path.to_str() {
            self.queue_with_name(path, path_name)
        } else {
            Err(ResourceQueueError::PathInvalidCharacters)
        }
    }

    /// Queues a file to be loaded when `load_all` is called. The file name will be used internally
    /// to identify the resource. If you queue two files with the same name, the engine will load
    /// both but the second one will overwrite the first.
    pub fn queue_with_name(&mut self,
                           path: &path::Path,
                           name: &str)
                           -> Result<(), ResourceQueueError> {
        if path.is_file() {
            match (path.to_str(), path.extension()) {
                (Some(_), Some(_)) => {
                    self.queue.push((path.to_path_buf(), name.to_string()));
                    Ok(())
                }
                (Some(_), None) => Err(ResourceQueueError::PathWithoutExtension),
                _ => Err(ResourceQueueError::NameInvalidCharacters),
            }
        } else {
            Err(ResourceQueueError::PathIsNotFile)
        }
    }

    /// Loads every queued file. Successful files are added to the list and the callback `success`
    /// is called. Failures are ignored and the `failure` callback is called.
    pub fn load_all<S: FnMut(&str), F: FnMut(&str, ResourceLoadError)>(&mut self,
                                                                       mut success: S,
                                                                       mut failure: F) {
        let queue = mem::replace(&mut self.queue, Vec::new());

        for queued_resource in queue {
            for loader in &self.loaders {
                if !loader.extensions()
                         .contains(&&queued_resource.0
                                                   .extension()
                                                   .unwrap()
                                                   .to_string_lossy()
                                                   .into_owned()[..]) { continue; }
                match loader.file_type() {
                    FileType::Binary => {
                        match load_binary_file(queued_resource.0.as_path()) {
                            Ok(raw_file) => {
                                match loader.load_from_binary(&raw_file[..]) {
                                    Ok(file) => {
                                        self.data.insert(queued_resource.1.clone(), file);
                                        success(&queued_resource.1[..]);
                                    }
                                    Err(e) => failure(&queued_resource.1[..], e),
                                }
                            }
                            Err(e) => failure(&queued_resource.1[..], e),
                        }
                    }
                    FileType::Text => {
                        match load_text_file(queued_resource.0.as_path()) {
                            Ok(raw_file) => {
                                match loader.load_from_text(&raw_file[..]) {
                                    Ok(file) => {
                                        self.data.insert(queued_resource.1.clone(), file);
                                        success(&queued_resource.1[..]);
                                    }
                                    Err(e) => failure(&queued_resource.1[..], e),
                                }
                            }
                            Err(e) => failure(&queued_resource.1[..], e),
                        }
                    }
                }
            }
        }
    }

    /// Returns a loaded resource with type T. If the name doesn't exist or T is not the correct
    /// type, None is returned.
    pub fn get<T: Resource>(&self, name: String) -> Option<&T> {
        // TODO this should return an RC?
        self.data.get(&name).and_then(|res| res.downcast_ref::<T>())
    }
}

#[cfg(test)]
mod test {
    use super::{Resources, ResourceQueueError, ResourceLoadError};
    use std::path::Path;

    #[test]
    fn resource_loading() {
        let mut resources = Resources::new();

        match resources.queue(Path::new("/inexistent/path")) {
            Err(ResourceQueueError::PathIsNotFile) => (),
            Err(e) => panic!(e),
            _ => panic!(),
        };

        match resources.queue(Path::new("static_assets/mesh/cube.obj")) {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        };

        assert_eq!(resources.queue.len(), 1);
        assert_eq!(resources.queue[0],
                   (Path::new("static_assets/mesh/cube.obj").to_path_buf(),
                    "static_assets/mesh/cube.obj".to_string()));

        match resources.queue_with_name(Path::new("static_assets/mesh/sphere.obj"), "sphere") {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        };

        assert_eq!(resources.queue.len(), 2);
        assert_eq!(resources.queue[1],
                   (Path::new("static_assets/mesh/sphere.obj").to_path_buf(),
                    "sphere".to_string()));

        let mut successes = 0;
        {
            let success = |_: &str| {
                successes = successes + 1;
            };

            fn failure(_: &str, _: ResourceLoadError) {
                panic!();
            }
            resources.load_all(success, failure);
        }

        assert_eq!(successes, 2);

    }
}
