use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{
    parser::{parse, ParseContextBuilder},
    template::Template, CustomBlockParser, ParseError,
};

pub trait TemplateProvider {
    fn get_template(&self, key: &str) -> Result<Option<Arc<Template>>, ParseError>;
    fn insert_template(&self, key: String, tpl: Template) -> Arc<Template>;
}

pub struct DefaultTemplateProvider {
    templates: RwLock<HashMap<String, Arc<Template>>>,
}

impl TemplateProvider for DefaultTemplateProvider {
    fn get_template(&self, key: &str) -> Result<Option<Arc<Template>>, ParseError> {
        Ok(self.templates.read().unwrap().get(key).map(Arc::clone))
    }

    fn insert_template(&self, key: String, tpl: Template) -> Arc<Template> {
        self.templates.write().unwrap().insert(key.clone(), Arc::new(tpl));
        Arc::clone(self.templates.read().unwrap().get(key.as_str()).unwrap())
    }
}

impl Default for DefaultTemplateProvider {
    fn default() -> Self {
        Self {
            templates: RwLock::<HashMap<String, Arc<Template>>>::default(),
        }
    }
}

pub struct FileSystemTemplateProvider<Inner = DefaultTemplateProvider>
where
    Inner: TemplateProvider,
{
    base_dir: PathBuf,
    inner: Inner,
    custom_blocks: Arc<HashMap<String, Box<dyn CustomBlockParser>>>
}

impl FileSystemTemplateProvider<DefaultTemplateProvider> {
    pub fn new(base_dir: PathBuf, custom_blocks: Arc<HashMap<String, Box<dyn CustomBlockParser>>>) -> std::io::Result<Self> {
        Self::new_with_provider(base_dir, custom_blocks, DefaultTemplateProvider::default())
    }
}

impl<Inner> FileSystemTemplateProvider<Inner>
where
    Inner: TemplateProvider,
{
    pub fn new_with_provider(base_dir: PathBuf, custom_blocks: Arc<HashMap<String, Box<dyn CustomBlockParser>>>, inner: Inner) -> std::io::Result<Self> {
        Ok(Self {
            base_dir: std::fs::canonicalize(base_dir)?,
            inner,
            custom_blocks
        })
    }

    fn get_file_path(&self, key: &str) -> std::io::Result<PathBuf> {
        let file = std::fs::canonicalize(self.base_dir.join(key))?;
        if file.starts_with(&self.base_dir) {
            return Ok(file);
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Tried to access file outside of the template directory",
        ))
    }
}

impl<Inner> TemplateProvider for FileSystemTemplateProvider<Inner>
where
    Inner: TemplateProvider,
{
    fn get_template(&self, key: &str) -> Result<Option<Arc<Template>>, ParseError> {
        match self.inner.get_template(key) {
            Ok(Some(tpl)) => return Ok(Some(tpl)),
            Ok(None) => {},
            Err(e) => return Err(e)
        }
        let path = self.get_file_path(key).unwrap();
        let tpl = std::fs::read_to_string(path).unwrap();
        let tpl = parse(
            tpl,
            &ParseContextBuilder::default()
                .custom_blocks(&self.custom_blocks)
                .build(),
        )?;
        let tpl = self.insert_template(key.to_owned(), tpl);
        Ok(Some(tpl))
    }

    fn insert_template(&self, key: String, tpl: Template) -> Arc<Template> {
        self.inner.insert_template(key, tpl)
    }
}
