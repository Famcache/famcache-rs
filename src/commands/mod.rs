pub fn set(id: &str, key: &str, value: &str, ttl: Option<u64>) -> String {
  if ttl.is_none() {
    return format!("{} SET {} {}\n", id, key, value)
  }

  return format!("{} SET {} {} {}\n", id, key, value, ttl.unwrap())
}

pub fn get(id: &str, key: &str) -> String {
  return format!("{} GET {}\n", id, key)
}

pub fn del(id: &str, key: &str) -> String {
  return format!("{} DELETE {}\n", id, key)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_set() {
    assert_eq!(set("1", "key", "value", None), "1 SET key value");
    assert_eq!(set("1", "key", "value", Some(10)), "1 SET key value 10");
  }

  #[test]
  fn test_get() {
    assert_eq!(get("1", "key"), "1 GET key");
  }

  #[test]
  fn test_del() {
    assert_eq!(del("1", "key"), "1 DEL key");
  }
}