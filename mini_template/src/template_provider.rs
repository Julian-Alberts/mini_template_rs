use std::{sync::{RwLock, Arc}, collections::HashMap};

use crate::template::Template;

pub trait TemplateProvider {

    fn get_template(&self, key: &str) -> Option<Arc<Template>>;
    fn insert_template(&self, key: String, tpl: Template);

}

pub struct DefaultTemplateProvider {
    templates: RwLock<HashMap<String, Arc<Template>>>,
}

impl TemplateProvider for DefaultTemplateProvider {
    fn get_template(&self, key: &str) -> Option<Arc<Template>> {
        self.templates.read().unwrap().get(key).map(Arc::clone)
    }

    fn insert_template(&self, key: String, tpl: Template) {
        self.templates.write().unwrap().insert(key, Arc::new(tpl));
    }
}

impl Default for DefaultTemplateProvider {

    fn default() -> Self {
        Self {
            templates: RwLock::<HashMap<String, Arc<Template>>>::default()
        }
    }

}
