use crate::template::CalculatedValue;
use crate::{Render, RenderContext, VariableManager};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Debug)]
pub struct Include {
    pub template_name: CalculatedValue,
}

impl Render for Include {
    fn render<VM: VariableManager>(
        &self,
        context: &mut RenderContext<VM>,
        buf: &mut String,
    ) -> crate::error::Result<()> {
        let mut hasher = DefaultHasher::default();
        self.template_name.calc(context)?.hash(&mut hasher);
        let hash = hasher.finish();
        let template = context
            .templates
            .get(&hash)
            .ok_or(crate::error::Error::UnknownTemplate)?;
        template.render(context, buf)
    }
}
