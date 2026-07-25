use anyhow::Result;

use crate::db::Convertible;

fn to_vec<T, U: Convertible<U> >(items: impl IntoIterator) -> Result<Vec<U>> {
    let mut result: Vec<U> = vec![];
    for item in items {
        let dto = U::from_model(item)?;
        result.push(dto);
    }

    Ok(result)
}
