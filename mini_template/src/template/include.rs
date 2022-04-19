use crate::template::CalculatedValue;
use crate::{Render, RenderContext};

#[derive(PartialEq, Debug)]
pub struct Include {
    pub template_name: CalculatedValue,
}

impl Render for Include {
    fn render<'a>(
        &self,
        context: &mut RenderContext,
        buf: &mut String,
    ) -> crate::error::Result<()> {
        let key: String = match self.template_name.calc(context)?.try_into() {
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
