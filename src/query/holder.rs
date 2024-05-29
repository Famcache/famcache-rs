#[derive(Debug, Clone)]
pub struct CacheQuery {
  pub(crate) id: String,
  pub(crate) value: Option<String>,
  pub(crate) is_error: bool,
}

impl CacheQuery {
  pub fn from_str(query: &str) -> Result<CacheQuery, &'static str> {
    let escaped = query.replace("\n", "");
    let parts: Vec<&str> = escaped.split(" ").collect();

    if parts.len() < 2 {
      return Err("Invalid query");
    }

    let id = parts[0];
    let status = parts[1];

    let value = if parts.len() == 3 {
      Some(parts[2])
    } else {
      None
    };

    let is_error = status == "ERROR";

    Ok(CacheQuery::new(id, value, is_error))
  }

  pub fn new(id: &str, value: Option<&str>, is_error: bool) -> CacheQuery {
    CacheQuery {
      id: id.to_string(),
      value: value.map(|v| v.to_string()),
      is_error
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from_str() {
    let query = CacheQuery::from_str("asd-asdasd-asdads OK 10").unwrap();
    assert_eq!(query.id, "asd-asdasd-asdads");
    assert_eq!(query.value, Some("10".to_string()));
    assert_eq!(query.is_error, false);

    let query = CacheQuery::from_str("asda-asdasd-asdads OK").unwrap();
    assert_eq!(query.id, "asda-asdasd-asdads");
    assert_eq!(query.value, None);
    assert_eq!(query.is_error, false);

    let query = CacheQuery::from_str("asdas-asdasdd-asdd ERROR").unwrap();
    assert_eq!(query.id, "asdas-asdasdd-asdd");
    assert_eq!(query.value, None);
    assert_eq!(query.is_error, true);
  }

  #[test]
  fn test_new() {
    let query = CacheQuery::new("asd-asdasd-asdads", Some("10"), false);
    assert_eq!(query.id, "asd-asdasd-asdads");
    assert_eq!(query.value, Some("10".to_string()));
    assert_eq!(query.is_error, false);

    let query = CacheQuery::new("asda-asdasd-asdads", None, false);
    assert_eq!(query.id, "asda-asdasd-asdads");
    assert_eq!(query.value, None);
    assert_eq!(query.is_error, false);

    let query = CacheQuery::new("asdas-asdasdd-asdd", None, true);
    assert_eq!(query.id, "asdas-asdasdd-asdd");
    assert_eq!(query.value, None);
    assert_eq!(query.is_error, true);
  }
}