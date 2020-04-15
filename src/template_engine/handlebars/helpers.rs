use handlebars::{Helper, HelperResult, JsonRender, Output, RenderError};
use serde_json::Value;

#[macro_export]
macro_rules! helper {
    ($handle:expr, $name:expr, $func:expr) => {
        $handle.register_helper(
            $name,
            Box::new(
                |h: &Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult { $func(h, out) },
            ),
        );
    };
}

pub fn json_str(h: &Helper, out: &mut dyn Output) -> HelperResult {
    let val = param(h, 0)?;
    out.write(val.to_string().as_str())?;
    Ok(())
}

pub fn concat(h: &Helper, out: &mut dyn Output) -> HelperResult {
    out.write(
        h.params()
            .iter()
            .map(|val| val.value())
            .fold(String::new(), |mut acc, next| {
                acc.push_str(next.render().as_str());
                acc
            })
            .as_str(),
    )?;
    Ok(())
}

pub fn ternary(h: &Helper, out: &mut dyn Output) -> HelperResult {
    let cond = param(h, 0)?;
    let truthy_val = param(h, 1)?;
    let falsy_val = param(h, 2)?;

    let cond = cond
        .as_bool()
        .or_else(|| cond.as_str().map(|s| !s.is_empty()))
        .or_else(|| cond.as_f64().map(|n| n != 0_f64))
        .or_else(|| cond.as_array().map(|a| !a.is_empty()))
        .or_else(|| cond.as_null().map(|val| false))
        .unwrap_or(false);

    if cond {
        out.write(truthy_val.render().as_str())?;
    } else {
        out.write(falsy_val.render().as_str())?;
    }

    Ok(())
}

fn param<'a>(h: &'a Helper, idx: usize) -> Result<&'a Value, RenderError> {
    h.param(idx)
        .map(|v| v.value())
        .ok_or_else(|| RenderError::new(format!("The {} param not provided", idx)))
}
