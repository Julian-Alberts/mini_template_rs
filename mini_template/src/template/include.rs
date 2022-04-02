use crate::template::CalculatedValue;
use crate::{Render, RenderContext, TemplateKey};

#[derive(PartialEq, Debug)]
pub struct Include {
    pub template_name: CalculatedValue,
}

impl Render for Include {
    fn render<'a, TK>(
        &self,
        context: &mut RenderContext<TK>,
        buf: &mut String,
    ) -> crate::error::Result<()>
    where
        TK: TemplateKey,
    {
        let key = match self.template_name.calc(context)?.try_into() {
            Ok(key) => key,
            Err(e) => return Err(crate::error::Error::Include(e)),
        };
        let template = context
            .templates
            .get(&key)
            .ok_or(crate::error::Error::UnknownTemplate)?;
        template.render(context, buf)
    }
}
